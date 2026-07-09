//! Certificate lifecycle stage vocabulary shared by the metric families
//! carrying a `stage` label.

/// Label key carrying the certificate lifecycle stage.
pub(crate) const STAGE_LABEL: &str = "stage";

/// A certificate lifecycle stage, rendered as the `stage` label value.
///
/// The metric families each use a subset: the duration histograms time the
/// non-terminal stages (`Pending`, `Proven`, `Candidate`), while the
/// per-network height gauge reports pointer positions (`Pending`, `Proven`,
/// `Settled`). Sharing one enum keeps the label values consistent across
/// families.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CertificateStage {
    Pending,
    Proven,
    Candidate,
    Settled,
}

impl CertificateStage {
    /// The `stage` label value for this stage.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Proven => "proven",
            Self::Candidate => "candidate",
            Self::Settled => "settled",
        }
    }
}

impl std::fmt::Display for CertificateStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
