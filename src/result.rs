use std::process::{Termination, ExitCode};

use crate::value::MiValue;
use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
/// Represents either an error or a value
pub enum MiResult {
    Ok(MiValue),
    Err(MiError),
}

impl Termination for MiResult {
    fn report(self) -> ExitCode {
        match self {
            MiResult::Ok(_) => ExitCode::SUCCESS,
            MiResult::Err(_) => ExitCode::FAILURE,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
/// Holds the error data that the VM can unwind
pub struct MiError {
    pub name: String,
    pub message: String,
    pub backtrace: String,
}