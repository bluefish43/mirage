use fxhash::FxHashMap;
use serde_derive::{Serialize, Deserialize};
use crate::value::MiType;

/// Represents a function' arguments
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct MiArgs {
    pub arguments: FxHashMap<String, MiType>,
    pub variant: Option<String>,
}