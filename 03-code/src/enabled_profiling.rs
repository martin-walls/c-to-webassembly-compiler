use clap::ValueEnum;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum EnabledProfiling {
    All,
    None,
}

impl EnabledProfiling {
    pub fn is_stack_ptr_logging_enabled(&self) -> bool {
        match self {
            EnabledProfiling::All => true,
            EnabledProfiling::None => false,
        }
    }
}
