use slint_interpreter::ComponentDefinition;

#[napi(js_name = "ComponentDefinition")]
pub struct JsComponentDefinition {
    internal: ComponentDefinition,
}

impl From<ComponentDefinition> for JsComponentDefinition {
    fn from(definition: ComponentDefinition) -> Self {
        Self {
            internal: definition
        }
    }
}

#[napi]
impl JsComponentDefinition {
}