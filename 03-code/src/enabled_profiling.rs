use crate::CliConfig;

#[derive(Debug)]
pub struct EnabledProfiling {
    stack_ptr_logging: bool,
}

impl EnabledProfiling {
    fn defaults() -> Self {
        EnabledProfiling {
            stack_ptr_logging: false,
        }
    }

    pub fn construct(cli_config: &CliConfig) -> Self {
        let mut enabled_profiling = EnabledProfiling::defaults();

        // the flags are mutually exclusive
        if cli_config.prof_stack {
            enabled_profiling.stack_ptr_logging = true;
        } else if cli_config.noprof_stack {
            enabled_profiling.stack_ptr_logging = false;
        }

        enabled_profiling
    }

    pub fn is_stack_ptr_logging_enabled(&self) -> bool {
        self.stack_ptr_logging
    }
}
