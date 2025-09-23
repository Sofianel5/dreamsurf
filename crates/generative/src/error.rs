use async_openai::error::OpenAIError;
use thiserror::Error;

pub type GenerativeResult<T> = Result<T, GenerativeError>;

#[derive(Debug, Error)]
pub enum GenerativeError {
    #[error("OpenAI request failed: {0}")]
    OpenAi(#[from] OpenAIError),

    #[error("Image response did not include expected data")]
    MissingImageData,
}
