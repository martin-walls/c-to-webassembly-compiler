use crate::middle_end::ir::Program;
use crate::middle_end::middle_end_error::MiddleEndError;
use crate::middle_end::middle_end_optimiser::remove_redundancy::remove_unused_labels;
use crate::middle_end::middle_end_optimiser::tail_call_optimise::tail_call_optimise;

pub fn optimise_ir(prog: &mut Box<Program>) -> Result<(), MiddleEndError> {
    for (fun_id, function) in &mut prog.program_instructions.functions {
        tail_call_optimise(
            &mut function.instrs,
            fun_id,
            &function.param_var_mappings,
            &mut prog.program_metadata,
        );
        remove_unused_labels(&mut function.instrs)?;
    }
    remove_unused_labels(&mut prog.program_instructions.global_instrs)?;
    Ok(())
}
