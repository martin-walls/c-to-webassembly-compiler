use crate::middle_end::ir::Program;
use crate::middle_end::middle_end_error::MiddleEndError;
use crate::middle_end::middle_end_optimiser::remove_redundancy::remove_unused_labels;
use crate::middle_end::middle_end_optimiser::tail_call_optimise::tail_call_optimise;
use crate::middle_end::middle_end_optimiser::unreachable_procedure_elimination::remove_unused_functions;
use crate::EnabledOptimisations;

pub fn optimise_ir(
    prog: &mut Box<Program>,
    enabled_optimisations: &EnabledOptimisations,
) -> Result<(), MiddleEndError> {
    for (fun_id, function) in &mut prog.program_instructions.functions {
        if enabled_optimisations.is_tail_call_optimisation_enabled() {
            tail_call_optimise(
                &mut function.instrs,
                fun_id,
                &function.param_var_mappings,
                &mut prog.program_metadata,
            );
        }
        remove_unused_labels(&mut function.instrs)?;
    }
    remove_unused_labels(&mut prog.program_instructions.global_instrs)?;

    if enabled_optimisations.is_unreachable_procedure_elimination_enabled() {
        remove_unused_functions(prog)?;
    }

    Ok(())
}
