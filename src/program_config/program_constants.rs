/// The name of the `main` function in the C source code.
pub const MAIN_FUNCTION_SOURCE_NAME: &str = "main";
/// The name of the function exported from the Wasm module that the JavaScript
/// runtime will call to run the program. Must match the name of the function called
/// in `runtime/run.mjs`
pub const MAIN_FUNCTION_EXPORT_NAME: &str = "main";

/// The module name of the memory import to the Wasm module. Must match the corresponding
// /// import in `runtime/run.mjs`.
pub const MEMORY_IMPORT_MODULE_NAME: &str = "runtime";
/// The field name of the memory import to the Wasm module. Must match the corresponding
/// import in `runtime/run.mjs`.
pub const MEMORY_IMPORT_FIELD_NAME: &str = "memory";

pub const LOG_STACK_PTR_IMPORT_NAME: &str = "log_stack_ptr";

/// A list of the standard library functions that I've implemented in the JavaScript
/// runtime, that will get imported. Must match the corresponding import names in `runtime/run.mjs`.
pub fn get_imported_function_names() -> Vec<String> {
    vec![
        "printf".to_owned(),
        "strtoul".to_owned(),
        "strtol".to_owned(),
        "strlen".to_owned(),
        "strstr".to_owned(),
        LOG_STACK_PTR_IMPORT_NAME.to_owned(),
    ]
}
