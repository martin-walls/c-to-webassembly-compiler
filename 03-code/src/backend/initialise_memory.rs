use log::info;

use crate::backend::import_export_names::{MEMORY_IMPORT_FIELD_NAME, MEMORY_IMPORT_MODULE_NAME};
use crate::backend::memory_constants::{PTR_SIZE, STACK_PTR_ADDR};
use crate::backend::target_code_generation_context::ModuleContext;
use crate::backend::wasm_instructions::{WasmExpression, WasmInstruction};
use crate::backend::wasm_module::data_section::DataSegment;
use crate::backend::wasm_module::imports_section::{ImportDescriptor, WasmImport};
use crate::backend::wasm_module::module::WasmModule;
use crate::backend::wasm_types::{Limits, MemoryType};
use crate::middle_end::ir::ProgramMetadata;

pub fn initialise_memory(
    wasm_module: &mut WasmModule,
    module_context: &mut ModuleContext,
    prog_metadata: &Box<ProgramMetadata>,
) -> u32 {
    // ----------------------------------------------------------
    // | FP | temp FP | SP | String literals | ...stack frames...
    // ----------------------------------------------------------
    // initialise with placeholder values for frame ptr and stack ptr
    let mut data: Vec<u8> = vec![0x00; (3 * PTR_SIZE) as usize];

    // store string literals in memory
    for (string_literal_id, string) in &prog_metadata.string_literals {
        // store the pointer to the string
        let ptr_to_string = data.len();
        info!(
            "Storing string literal {:?} at addr {}",
            string, ptr_to_string
        );
        module_context
            .string_literal_id_to_ptr_map
            .insert(string_literal_id.to_owned(), ptr_to_string as u32);
        // insert the string
        data.append(&mut string.as_bytes().to_vec());
        // null terminate the string
        data.push(0x00);
    }

    // set stack ptr to point at top of stack
    let stack_ptr_value = data.len();
    info!("Setting stack ptr to {}", stack_ptr_value);
    data[STACK_PTR_ADDR as usize] = (stack_ptr_value & 0xFF) as u8;
    data[(STACK_PTR_ADDR + 1) as usize] = ((stack_ptr_value >> 8) & 0xFF) as u8;
    data[(STACK_PTR_ADDR + 2) as usize] = ((stack_ptr_value >> 16) & 0xFF) as u8;
    data[(STACK_PTR_ADDR + 3) as usize] = ((stack_ptr_value >> 24) & 0xFF) as u8;

    // insert data segment to module
    let data_segment = DataSegment::ActiveSegmentMemIndexZero {
        offset_expr: WasmExpression {
            instrs: vec![WasmInstruction::I32Const { n: 0 }],
        },
        data,
    };
    wasm_module.data_section.data_segments.push(data_segment);

    // import memory from JS runtime
    let memory_import = WasmImport {
        module_name: MEMORY_IMPORT_MODULE_NAME.to_owned(),
        field_name: MEMORY_IMPORT_FIELD_NAME.to_owned(),
        import_descriptor: ImportDescriptor::Mem {
            mem_type: MemoryType {
                limits: Limits { min: 1, max: None },
            },
        },
    };
    wasm_module.imports_section.imports.push(memory_import);

    stack_ptr_value as u32
}
