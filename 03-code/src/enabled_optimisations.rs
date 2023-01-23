use clap::ValueEnum;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum EnabledOptimisations {
    All,
    None,
}

impl EnabledOptimisations {
    pub fn is_tail_call_optimisation_enabled(&self) -> bool {
        match self {
            EnabledOptimisations::All => true,
            EnabledOptimisations::None => false,
        }
    }

    pub fn is_unreachable_procedure_elimination_enabled(&self) -> bool {
        match self {
            EnabledOptimisations::All => true,
            EnabledOptimisations::None => false,
        }
    }
}
