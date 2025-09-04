use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtoConversionError {
    #[error("wrong enum value")]
    WrongEnumValue(#[from] prost::UnknownEnumValue),
    #[error("missing value for field `{0}`")]
    MissingValue(&'static str),
}
