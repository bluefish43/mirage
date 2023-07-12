use std::time::SystemTime;

use serde_derive::{Serialize, Deserialize};

use crate::instructions::Instruction;

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub package: String,
    pub version: Option<String>,
    pub timestamp: SystemTime,
    pub author: Option<String>,
    pub debug: bool,
    pub instructions: Vec<Instruction>,
    pub source_code: Option<String>,
    pub description: String,
    pub license: Option<String>,
    pub total_instructions: usize,
    pub compiled_version: String,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub package: String,
    pub version: Option<String>,
    pub author: Option<String>,
    pub main_file: String,
    pub description: Option<String>,
    pub license: String,
}