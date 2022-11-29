use crate::middle_end::ir::Program;
use crate::middle_end::middle_end_error::MiddleEndError;
use crate::middle_end::middle_end_optimiser::remove_redundancy::remove_unused_labels;

pub fn optimise_ir(prog: &mut Box<Program>) -> Result<(), MiddleEndError> {
    for (_fun_id, function) in &mut prog.functions {
        remove_unused_labels(&mut function.instrs)?;
    }
    remove_unused_labels(&mut prog.global_instrs)?;
    Ok(())
}
