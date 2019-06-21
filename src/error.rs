use std::error::Error;
use std::fmt;
use std::fmt::Display;

#[derive(Debug)]
pub struct InstallError {
    details: String,
}

impl InstallError {
    pub fn new(msg: String) -> InstallError {
        InstallError { details: msg }
    }
}

impl Display for InstallError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for InstallError {
    fn description(&self) -> &str {
        &self.details
    }
}
