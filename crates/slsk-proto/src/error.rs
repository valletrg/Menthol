use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtoError {
    #[error("unexpected end of input (need {needed} bytes, have {have})")]
    UnexpectedEof { needed: usize, have: usize },

    #[error("invalid message code: {0}")]
    UnknownCode(u32),

    #[error("invalid string encoding")]
    InvalidString(#[from] std::string::FromUtf8Error),

    #[error("invalid enum value {value} for {type_name}")]
    InvalidEnumValue { value: u32, type_name: &'static str },
}
