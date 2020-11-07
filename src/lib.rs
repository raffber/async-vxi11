use std::io;
use thiserror::Error;

mod tcp_client;
mod client;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO Error occurred: {0}")]
    Io(io::Error),
    #[error("Cannot unpack message")]
    CannotUnpack,

}

pub type Result<T> = std::result::Result<T, Error>;
