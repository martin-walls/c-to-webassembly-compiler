use crate::backend::integer_encoding::encode_unsigned_int;
use crate::backend::target_code_generation_context::ModuleContext;
use crate::backend::to_bytes::ToBytes;
use crate::backend::wasm_indices::{FuncIdx, TypeIdx, WasmIdx, WasmIdxGenerator};
use crate::backend::wasm_instructions::WasmExpression;
use crate::backend::wasm_module::code_section::{CodeSection, WasmFunctionCode};
use crate::backend::wasm_module::data_section::DataSection;
use crate::backend::wasm_module::element_section::ElementSection;
use crate::backend::wasm_module::exports_section::ExportsSection;
use crate::backend::wasm_module::functions_section::FunctionsSection;
use crate::backend::wasm_module::globals_section::GlobalsSection;
use crate::backend::wasm_module::imports_section::{ImportDescriptor, ImportsSection, WasmImport};
use crate::backend::wasm_module::memory_section::MemorySection;
use crate::backend::wasm_module::start_section::StartSection;
use crate::backend::wasm_module::tables_section::TablesSection;
use crate::backend::wasm_module::types_section::{TypesSection, WasmFunctionType};
use crate::backend::wasm_types::ValType;
use crate::relooper::relooper::ReloopedFunction;
use log::info;
use std::collections::HashMap;

const IMPORTS_MODULE_NAME: &str = "wasm_stdlib";

pub struct WasmModule {
    pub types_section: TypesSection,
    type_idx_generator: WasmIdxGenerator<TypeIdx>,
    pub imports_section: ImportsSection,
    pub functions_section: FunctionsSection,
    pub tables_section: TablesSection,
    pub memory_section: MemorySection,
    pub globals_section: GlobalsSection,
    pub exports_section: ExportsSection,
    pub start_section: StartSection,
    pub element_section: ElementSection,
    pub code_section: CodeSection,
    pub data_section: DataSection,
}

impl WasmModule {
    pub fn new() -> Self {
        WasmModule {
            types_section: TypesSection::new(),
            type_idx_generator: WasmIdxGenerator::new(),
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

    pub fn insert_type(&mut self, function_type: WasmFunctionType) -> TypeIdx {
        self.types_section.function_types.push(function_type);
        self.type_idx_generator.new_idx()
    }

    pub fn insert_defined_functions(
        &mut self,
        mut func_idx_to_body_code_map: HashMap<FuncIdx, WasmExpression>,
        mut func_idx_to_type_idx_map: HashMap<FuncIdx, TypeIdx>,
        module_context: &ModuleContext,
    ) {
        let mut func_idx = module_context.defined_func_idx_range.0.to_owned();
        loop {
            if func_idx == module_context.defined_func_idx_range.1 {
                break;
            }
            match func_idx_to_type_idx_map.remove(&func_idx) {
                None => {
                    break;
                }
                Some(type_idx) => {
                    info!("adding function {:?} to module", func_idx);
                    let body_code = func_idx_to_body_code_map.remove(&func_idx).unwrap();
                    self.functions_section.function_type_idxs.push(type_idx);

                    let code_entry = WasmFunctionCode {
                        local_declarations: Vec::new(),
                        function_body: body_code,
                    };

                    self.code_section.function_bodies.push(code_entry);
                }
            }
            func_idx = func_idx.next_idx();
        }
        // because we remove each entry as we process it, the maps should be empty at the end
        assert!(func_idx_to_type_idx_map.is_empty());
        assert!(func_idx_to_body_code_map.is_empty());
    }

    pub fn insert_imported_functions(
        &mut self,
        mut imported_func_idx_to_type_idx_map: HashMap<FuncIdx, TypeIdx>,
        mut imported_func_idx_to_name_map: HashMap<FuncIdx, String>,
        module_context: &ModuleContext,
    ) {
        let mut func_idx = module_context.imported_func_idx_range.0.to_owned();
        loop {
            if func_idx == module_context.imported_func_idx_range.1 {
                break;
            }
            match imported_func_idx_to_type_idx_map.remove(&func_idx) {
                None => break,
                Some(type_idx) => {
                    let name = imported_func_idx_to_name_map.remove(&func_idx).unwrap();
                    info!("importing function {:?} ({}) to module", func_idx, name);

                    let import = WasmImport {
                        module_name: IMPORTS_MODULE_NAME.to_owned(),
                        field_name: name,
                        import_descriptor: ImportDescriptor::Func {
                            func_type_idx: type_idx,
                        },
                    };
                    self.imports_section.imports.push(import);
                }
            }
            func_idx = func_idx.next_idx();
        }
        // because we remove each entry as we process it, the maps should be empty at the end
        assert!(imported_func_idx_to_type_idx_map.is_empty());
        assert!(imported_func_idx_to_name_map.is_empty());
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

    // don't need to output anything for empty section
    if body.len() == 0 {
        return bytes;
    }

    // section code
    bytes.push(section_code);
    // section size
    bytes.append(&mut encode_unsigned_int(body.len() as u128));
    // body
    bytes.append(&mut body);
    bytes
}
