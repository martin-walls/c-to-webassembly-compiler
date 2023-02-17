pub const MAIN_FUNCTION_EXPORT_NAME: &str = "main";

pub const MEMORY_IMPORT_MODULE_NAME: &str = "runtime";
pub const MEMORY_IMPORT_FIELD_NAME: &str = "memory";

pub fn get_imported_function_names() -> Vec<String> {
    vec![
        "printf".to_owned(),
        "strtoul".to_owned(),
        "strtol".to_owned(),
        "strlen".to_owned(),
        "strstr".to_owned(),
        "log_stack_ptr".to_owned(),
    ]
}
