use thiserror::Error;

use crate::packet::OptionParseErr;

#[derive(Debug, Error)]
pub enum DHCPError {
    #[error("{0}")]
    IoError(#[from] std::io::Error),
    #[error("Protocol error: {0}")]
    Protocol(String),
    #[error("Parse error {0}")]
    OptionsParseError(#[from] OptionParseErr)
}
