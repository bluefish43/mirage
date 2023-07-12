use crate::args::MiArgs;
use crate::instructions::Instruction;
use crate::value::MiType;
use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
/// Represents a user-defined function
pub struct MiFunction {
    pub name: String,
    pub arguments: MiArgs,
    pub returns: MiType,
    pub instructions: Vec<Instruction>,
}

/// Represents a function, that being a builtin or a user-defined as an enum
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Function {
    // the u32 represents the index at which this function is stored at the vm
    // as an Arc<dyn Fn([MiValue]) -> MiResult> cannot be computed into bytes
    Builtin(u32),
    Defined(MiFunction),
}

pub fn format_function(func: &MiFunction) -> String {
    let arg_string = func
        .arguments
        .arguments
        .values()
        .map(|arg| format!("{:?}", arg).to_lowercase())
        .collect::<Vec<String>>()
        .join(", ");

    let return_type = format!("{:?}", func.returns);

    format!("fun {}({}): {}", func.name, arg_string, return_type)
}