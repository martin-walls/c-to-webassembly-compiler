use crate::CliConfig;

#[derive(Debug)]
pub struct EnabledOptimisations {
    tail_call: bool,
    unreachable_procedure: bool,
    stack_allocation: bool,
}

impl EnabledOptimisations {
    fn defaults() -> Self {
        EnabledOptimisations {
            tail_call: true,
            unreachable_procedure: true,
            stack_allocation: true,
        }
    }

    pub fn construct(cli_config: &CliConfig) -> Self {
        let mut enabled_optimisations = EnabledOptimisations::defaults();

        // the flags are mutually exclusive
        if cli_config.opt_tailcall {
            enabled_optimisations.tail_call = true;
        } else if cli_config.noopt_tailcall {
            enabled_optimisations.tail_call = false;
        }

        if cli_config.opt_unreachable_procedure {
            enabled_optimisations.unreachable_procedure = true;
        } else if cli_config.noopt_unreachable_procedure {
            enabled_optimisations.unreachable_procedure = false;
        }

        if cli_config.opt_stack_allocation {
            enabled_optimisations.stack_allocation = true;
        } else if cli_config.noopt_stack_allocation {
            enabled_optimisations.stack_allocation = false;
        }

        enabled_optimisations
    }

    pub fn is_tail_call_optimisation_enabled(&self) -> bool {
        self.tail_call
    }

    pub fn is_unreachable_procedure_elimination_enabled(&self) -> bool {
        self.unreachable_procedure
    }

    pub fn is_stack_allocation_optimisation_enabled(&self) -> {
        self.stack_allocation
    }
}
