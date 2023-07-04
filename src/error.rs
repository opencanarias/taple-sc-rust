use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum Error {
  #[error("Serialization Error")]
  SerializationError,
  #[error("Deserialization Error")]
  DeserializationError
}
