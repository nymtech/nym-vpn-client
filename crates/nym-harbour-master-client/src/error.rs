#[derive(Debug, thiserror::Error)]
pub enum HabourMasterError {
    #[error("error")]
    General,
}
