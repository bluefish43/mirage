use fxhash::FxHashMap;
use serde_derive::{Serialize, Deserialize};
use crate::value::{ToStringDebugged, MiValue, MiType};
use crate::function::Function;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]

pub struct Class {
    pub name: String,
    pub properties: FxHashMap<String, MiValue>,
}

impl Class {
    pub fn format_debugged(&self) -> String {
        let properties = self
            .properties
            .iter()
            .map(|(key, value)| format!("   {}: {},", key, value.to_string_debugged()))
            .collect::<Vec<String>>()
            .join("\n");
    
        format!("{} {{\n{}\n}}", self.name, properties)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ClassBlueprint {
    pub name: String,
    pub functions: FxHashMap<String, Function>,
    pub variables: FxHashMap<String, MiType>,
}