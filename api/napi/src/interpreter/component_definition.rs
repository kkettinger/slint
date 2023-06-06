use slint_interpreter::ComponentDefinition;

use super::JsComponentInstance;

#[napi(js_name = "ComponentDefinition")]
pub struct JsComponentDefinition {
    internal: ComponentDefinition,
}

impl From<ComponentDefinition> for JsComponentDefinition {
    fn from(definition: ComponentDefinition) -> Self {
        Self { internal: definition }
    }
}

#[napi]
impl JsComponentDefinition {
    #[napi(constructor)]
    pub fn new() -> Self {
        unreachable!("ComponentDefinition can only be created by using ComponentCompiler.")
    }

    #[napi(getter)]
    pub fn name(&self) -> String {
        self.internal.name().into()
    }

    #[napi]
    pub fn create(&self) -> Option<JsComponentInstance> {
        if let Ok(instance) = self.internal.create() {
            return Some(instance.into());
        }

        None
    }
}
