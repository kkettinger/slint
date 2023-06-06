use std::path::PathBuf;

use slint_interpreter::ComponentCompiler;

#[napi(js_name = "ComponentCompiler")]
pub struct JsComponentCompiler {
    internal: ComponentCompiler,
}

#[napi]
impl JsComponentCompiler {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self { internal: ComponentCompiler::default() }
    }

    #[napi]
    pub fn set_include_paths(&mut self, include_paths: Vec<String>) {
        self.internal.set_include_paths(include_paths.iter().map(|p| PathBuf::from(p)).collect());
    }

    #[napi]
    pub fn include_paths(&self) -> Vec<String> {
        self.internal
            .include_paths()
            .iter()
            .map(|p| p.to_str().unwrap_or_default().to_string())
            .collect()
    }

    #[napi]
    pub fn set_style(&mut self, style: String) {
        self.internal.set_style(style);
    }

    #[napi]
    pub fn style(&self) -> Option<String> {
        self.internal.style().cloned()
    }
}
