use crate::backend::wasm_indices::TypeIdx;
use crate::backend::wasm_instructions::WasmExpression;
use crate::backend::wasm_module::code_section::CodeSection;
use crate::backend::wasm_module::data_section::DataSection;
use crate::backend::wasm_module::element_section::ElementSection;
use crate::backend::wasm_module::exports_section::ExportsSection;
use crate::backend::wasm_module::functions_section::FunctionsSection;
use crate::backend::wasm_module::globals_section::GlobalsSection;
use crate::backend::wasm_module::imports_section::ImportsSection;
use crate::backend::wasm_module::memory_section::MemorySection;
use crate::backend::wasm_module::start_section::StartSection;
use crate::backend::wasm_module::tables_section::TablesSection;
use crate::backend::wasm_module::types_section::TypesSection;
use crate::backend::wasm_types::ValType;

pub struct WasmModule {
    types: TypesSection,
    imports: ImportsSection,
    functions: FunctionsSection,
    tables: TablesSection,
    memory: MemorySection,
    globals: GlobalsSection,
    exports: ExportsSection,
    start: StartSection,
    element: ElementSection,
    code: CodeSection,
    data: DataSection,
}

impl WasmModule {
    pub fn new() -> Self {
        WasmModule {
            types: TypesSection::new(),
            imports: ImportsSection::new(),
            functions: FunctionsSection::new(),
            tables: TablesSection::new(),
            memory: MemorySection::new(),
            globals: GlobalsSection::new(),
            exports: ExportsSection::new(),
            start: StartSection::new(),
            element: ElementSection::new(),
            code: CodeSection::new(),
            data: DataSection::new(),
        }
    }
}

pub struct WasmFunction {
    type_idx: TypeIdx,
    local_declarations: Vec<ValType>,
    body: WasmExpression,
}
