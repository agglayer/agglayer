use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionMode {
    Default,
    DryRun,
}

impl ExecutionMode {
    pub const fn prefix(&self) -> &'static str {
        match self {
            ExecutionMode::Default => "",
            ExecutionMode::DryRun => "(Dry run) ",
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, thiserror::Error, PartialEq, Eq)]
pub enum GenerationType {
    Native,
    Prover,
}

impl fmt::Display for GenerationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenerationType::Native => write!(f, "native"),
            GenerationType::Prover => write!(f, "prover"),
        }
    }
}
