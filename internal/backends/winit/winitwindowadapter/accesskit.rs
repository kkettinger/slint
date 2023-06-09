// Copyright © SixtyFPS GmbH <info@slint-ui.com>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-commercial

use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
    sync::{Arc, Condvar, Mutex},
};

use accesskit::{Node, NodeBuilder, NodeId, Role, Tree, TreeUpdate};
use i_slint_core::{
    component::ComponentVTable,
    items::{ItemRc, WindowItem},
    window::WindowAdapter,
};

use super::WinitWindowAdapter;

#[derive(derive_more::Deref)]
pub struct AccessKitAdapter {
    #[deref]
    inner: accesskit_winit::Adapter,

    node_classes: RefCell<accesskit::NodeClassSet>,
    tree_generation: Cell<u16>,
}

impl AccessKitAdapter {
    pub fn new(
        window_adapter_weak: Weak<WinitWindowAdapter>,
        winit_window: &winit::window::Window,
    ) -> Self {
        let wrapped_window_adapter_weak = send_wrapper::SendWrapper::new(window_adapter_weak);
        Self {
            inner: accesskit_winit::Adapter::new(
                &winit_window,
                move || Self::global_build_tree_update(wrapped_window_adapter_weak.clone()),
                crate::event_loop::with_window_target(|event_loop_interface| {
                    event_loop_interface.event_loop_proxy().clone()
                }),
            ),
            node_classes: RefCell::new(accesskit::NodeClassSet::new()),
            tree_generation: Cell::new(0),
        }
    }

    pub fn handle_focus_change(&self, new: Option<ItemRc>) {
        self.inner.update_if_active(|| TreeUpdate {
            nodes: vec![],
            tree: None,
            focus: new.map(|item| item.node_id(self.tree_generation.get())),
        })
    }

    fn build_tree(&self, item: ItemRc, nodes: &mut Vec<(NodeId, Node)>) -> NodeId {
        let (role, label) = if let Some(window_item) = item.downcast::<WindowItem>() {
            (Role::Window, window_item.as_pin_ref().title().to_string())
        } else {
            (
                match item.accessible_role() {
                    i_slint_core::items::AccessibleRole::None => Role::Unknown,
                    i_slint_core::items::AccessibleRole::Button => Role::Button,
                    i_slint_core::items::AccessibleRole::Checkbox => Role::CheckBox,
                    i_slint_core::items::AccessibleRole::Combobox => Role::ComboBoxGrouping,
                    i_slint_core::items::AccessibleRole::Slider => Role::Slider,
                    i_slint_core::items::AccessibleRole::Spinbox => Role::SpinButton,
                    i_slint_core::items::AccessibleRole::Tab => Role::Tab,
                    i_slint_core::items::AccessibleRole::Text => Role::TextField,
                },
                item.accessible_string_property(
                    i_slint_core::accessibility::AccessibleStringProperty::Label,
                )
                .to_string(),
            )
        };

        let mut builder = NodeBuilder::new(role);

        builder.set_name(label);

        let mut descendents = Vec::new();
        i_slint_core::accessibility::accessible_descendents(&item, &mut descendents);

        builder.set_children(
            descendents
                .into_iter()
                .map(|child| self.build_tree(child, nodes))
                .collect::<Vec<NodeId>>(),
        );

        let id = item.node_id(self.tree_generation.get());

        let node = builder.build(&mut self.node_classes.borrow_mut());

        nodes.push((id, node));

        id
    }

    fn build_tree_update(&self, window_adapter: &Rc<WinitWindowAdapter>) -> TreeUpdate {
        let window_inner = i_slint_core::window::WindowInner::from_pub(window_adapter.window());

        let tree_generation = self.tree_generation.get() + 1;
        self.tree_generation.set(tree_generation);

        let root_item = ItemRc::new(window_inner.component(), 0);

        let mut nodes = Vec::new();
        let root_id = self.build_tree(root_item, &mut nodes);

        let update = TreeUpdate {
            nodes,
            tree: Some(Tree::new(root_id)),
            focus: window_inner
                .focus_item
                .borrow()
                .upgrade()
                .map(|item| item.node_id(tree_generation)),
        };

        update
    }

    fn global_build_tree_update(
        wrapped_window_adapter_weak: send_wrapper::SendWrapper<Weak<WinitWindowAdapter>>,
    ) -> TreeUpdate {
        if !wrapped_window_adapter_weak.valid() {
            let update_from_main_thread =
                Arc::new((Mutex::new(RefCell::new(None)), Condvar::new()));

            let e = crate::SlintUserEvent::CustomEvent {
                event: crate::event_loop::CustomEvent::UserEvent(Box::new({
                    let update_from_main_thread = update_from_main_thread.clone();
                    move || {
                        let (lock, wait_condition) = &*update_from_main_thread;
                        let update = lock.lock().unwrap();

                        *update.borrow_mut() =
                            Some(Self::global_build_tree_update(wrapped_window_adapter_weak));

                        wait_condition.notify_one();
                    }
                })),
            };
            if let Err(_) = crate::send_event_via_global_event_loop_proxy(e) {
                return Default::default();
            }

            let (lock, wait_condition) = &*update_from_main_thread;
            let mut update = lock.lock().unwrap();
            while update.borrow().is_none() {
                update = wait_condition.wait(update).unwrap();
            }

            return update.borrow_mut().take().unwrap();
        }
        let Some(window_adapter) = wrapped_window_adapter_weak.take().upgrade() else {
            return Default::default();
        };

        window_adapter.accesskit_adapter.build_tree_update(&window_adapter)
    }
}

trait AccessibilityMapping {
    fn node_id(&self, tree_generation: u16) -> NodeId;
    fn from_node_id(id: NodeId, expected_generation: u16) -> Option<Self>
    where
        Self: Sized;
}

impl AccessibilityMapping for i_slint_core::items::ItemRc {
    fn node_id(&self, tree_generation: u16) -> NodeId {
        let mut item = self.clone();
        while !item.is_accessible() {
            if let Some(parent) = item.parent_item() {
                item = parent;
            } else {
                break;
            }
        }

        let component_ptr = unsafe { item.component().as_raw() };
        let component_usize: u128 = component_ptr as usize as u128;
        let item_index: u128 = item.index() as u32 as u128;

        let encoded_id = std::num::NonZeroU128::new(
            (component_usize << 64) | ((tree_generation as u128) << 48) | item_index,
        )
        .unwrap();
        NodeId(encoded_id)
    }

    fn from_node_id(id: NodeId, expected_generation: u16) -> Option<Self> {
        let encoded_component = id.0.get() >> 64;
        let encoded_generation = ((id.0.get() >> 48) & 0xffff) as u16;
        let encoded_item_index = (id.0.get() & 0xffffffff) as u32;

        if encoded_generation != expected_generation {
            return None;
        }

        let component_rc = unsafe {
            vtable::VRc::<ComponentVTable, vtable::Dyn>::from_raw(encoded_component as *const _)
        };

        Some(ItemRc::new(component_rc, encoded_item_index as usize))
    }
}
