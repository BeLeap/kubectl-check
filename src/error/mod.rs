use std::error::Error as StdError;
use std::process::ExitStatus;
use std::result::Result as StdResult;
use std::{fmt, io};

pub enum Error {
    KubeconfigIo(io::Error),
    KubeconfigParse(yaml_rust2::ScanError),
    MalformedKubeconfig,
    CurrentContextNotFound(String),
    NoCommandSpecified,
    NotConfirmed,
    CommandFailed(ExitStatus),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::KubeconfigIo(ref err) => {
                write!(f, "Could not read kubeconfig: {err}")
            }
            Error::KubeconfigParse(ref err) => {
                write!(f, "Could not parse kubeconfig: {err}")
            }
            Error::NotConfirmed => write!(f, "Execution cancelled."),
            Error::CommandFailed(status) => {
                write!(f, "{status}")
            }
            Error::MalformedKubeconfig => write!(f, "Malformed kubeconfig"),
            Error::CurrentContextNotFound(current_context) => {
                write!(f, "Context not found: {current_context}")
            }
            Error::NoCommandSpecified => write!(f, "No command for kubectl sepcified"),
        }
    }
}
impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
impl StdError for Error {}

pub type Result<T> = StdResult<T, Error>;
