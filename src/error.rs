pub type SdCwtResult<T> = Result<T, SdCwtError>;

#[derive(Debug, thiserror::Error)]
pub enum SdCwtError {
    #[error("Invalid YAML input provided {0}")]
    InvalidYamlInput(serde_yaml::value::Tag),
    #[error(transparent)]
    JwtSimpleError(#[from] jwt_simple::Error),
    #[error(transparent)]
    YamlError(#[from] serde_yaml::Error),
    #[error("CborError")]
    CborError,
}