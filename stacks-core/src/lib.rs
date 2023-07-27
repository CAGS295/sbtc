use thiserror::Error;

pub mod address;
pub mod c32;
pub mod crypto;
pub mod utils;

#[derive(Error, Debug, Clone)]
pub enum StacksError {
    #[error("Invalid arguments: {0}")]
    InvalidArguments(&'static str),
    #[error("Could not crackford32 encode or decode: {0}")]
    C32Error(#[from] c32::C32Error),
    #[error("Address version is invalid: {0}")]
    InvalidAddressVersion(u8),
}

pub type StacksResult<T> = Result<T, StacksError>;

pub mod prelude {
    pub use super::StacksError;
    pub use super::StacksResult;

    pub use crate::address::*;
    pub use crate::c32::*;
    pub use crate::crypto::*;
    pub use crate::utils::*;
}
