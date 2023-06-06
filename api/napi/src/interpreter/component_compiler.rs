use std::path::PathBuf;

use super::JsComponentDefinition;
use super::JsDiagnostic;
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

    // todo: set_file_loader

    #[napi]
    pub fn diagnostics(&self) -> Vec<JsDiagnostic> {
        self.internal.diagnostics().iter().map(|d| JsDiagnostic::from(d.clone())).collect()
    }

    #[napi]
    pub fn build_from_path(&mut self, path: String) -> Option<JsComponentDefinition> {
        spin_on::spin_on(self.internal.build_from_path(PathBuf::from(path))).map(|d| d.into())
    }

    #[napi]
    pub fn build_from_source(
        &mut self,
        source_code: String,
        path: String,
    ) -> Option<JsComponentDefinition> {
        spin_on::spin_on(self.internal.build_from_source(source_code, PathBuf::from(path)))
            .map(|d| d.into())
    }
}
