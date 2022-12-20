use crate::backend::integer_encoding::encode_unsigned_int;
use crate::backend::to_bytes::ToBytes;
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
    types_section: TypesSection,
    imports_section: ImportsSection,
    functions_section: FunctionsSection,
    tables_section: TablesSection,
    memory_section: MemorySection,
    globals_section: GlobalsSection,
    exports_section: ExportsSection,
    start_section: StartSection,
    element_section: ElementSection,
    code_section: CodeSection,
    data_section: DataSection,
}

impl WasmModule {
    pub fn new() -> Self {
        WasmModule {
            types_section: TypesSection::new(),
            imports_section: ImportsSection::new(),
            functions_section: FunctionsSection::new(),
            tables_section: TablesSection::new(),
            memory_section: MemorySection::new(),
            globals_section: GlobalsSection::new(),
            exports_section: ExportsSection::new(),
            start_section: StartSection::new(),
            element_section: ElementSection::new(),
            code_section: CodeSection::new(),
            data_section: DataSection::new(),
        }
    }
}

impl ToBytes for WasmModule {
    fn to_bytes(&self) -> Vec<u8> {
        // WebAssembly magic number
        let mut bytes = vec![0x00, 0x61, 0x73, 0x6d];
        // WebAssembly version
        bytes.append(&mut vec![0x01, 0x00, 0x00, 0x00]);

        bytes.append(&mut self.types_section.to_bytes());
        bytes.append(&mut self.imports_section.to_bytes());
        bytes.append(&mut self.functions_section.to_bytes());
        bytes.append(&mut self.tables_section.to_bytes());
        bytes.append(&mut self.memory_section.to_bytes());
        bytes.append(&mut self.globals_section.to_bytes());
        bytes.append(&mut self.exports_section.to_bytes());
        bytes.append(&mut self.start_section.to_bytes());
        bytes.append(&mut self.element_section.to_bytes());
        bytes.append(&mut self.code_section.to_bytes());
        bytes.append(&mut self.data_section.to_bytes());

        bytes
    }
}

pub fn encode_section(section_code: u8, mut body: Vec<u8>) -> Vec<u8> {
    let mut bytes = Vec::new();
    // section code
    bytes.push(section_code);
    // section size
    bytes.append(&mut encode_unsigned_int(body.len() as u128));
    // body
    bytes.append(&mut body);
    bytes
}
