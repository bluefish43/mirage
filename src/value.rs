use crate::{class::Class, function::{Function, format_function}};
use bincode::{serialize, deserialize};
use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct MiValue {
    pub bytes: Vec<u8>,
    pub variant: MiType,
}

impl MiValue {
    pub fn new<T: Into<Vec<u8>>>(bytes: T, variant: MiType) -> MiValue {
        Self {
            bytes: bytes.into(),
            variant,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum MiType {
    Int,
    Float,
    String,
    Bool,
    Class,
    Function,
    None,
}

impl MiType {
    pub fn is_numeric(&self) -> bool {
        return
            self == &MiType::Int
            || self == &MiType::Float
    }
}

pub trait IntoValue {
    fn into_value(&self) -> MiValue;
}

impl IntoValue for i32 {
    fn into_value(&self) -> MiValue {
        return MiValue::new(self.to_le_bytes(), MiType::Int)
    }
}

impl IntoValue for f64 {
    fn into_value(&self) -> MiValue {
        return MiValue::new(self.to_le_bytes(), MiType::Float)
    }
}

impl IntoValue for String {
    fn into_value(&self) -> MiValue {
        return MiValue::new(serialize(self).unwrap(), MiType::String)
    }
}

impl IntoValue for bool {
    fn into_value(&self) -> MiValue {
        return MiValue::new(if *self { [1] } else { [0] }, MiType::Bool)
    }
}

impl IntoValue for Class {
    fn into_value(&self) -> MiValue {
        MiValue::new(serialize(self).unwrap(), MiType::Class)
    }
}

impl IntoValue for Function {
    fn into_value(&self) -> MiValue {
        MiValue::new(serialize(self).unwrap(), MiType::Function)
    }
}

pub trait ToStringDebugged {
    fn to_string_debugged(&self) -> String;
}



impl ToString for MiValue {
    fn to_string(&self) -> String {
        match self.variant {
            MiType::Bool => {
                if self.bytes[0] == 1 {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            MiType::String => {
                deserialize::<String>(&self.bytes).unwrap()
            }
            MiType::None => {
                "None".to_string()
            }
            MiType::Int => {
                let num = i32::from_le_bytes(self.bytes.clone().try_into().unwrap());
                format!("{}", num)
            }
            MiType::Float => {
                let num = f64::from_le_bytes(self.bytes.clone().try_into().unwrap());
                format!("{}", num)
            }
            MiType::Function => {
                let fun = deserialize::<Function>(&self.bytes);
                match fun {
                    Ok(fun) => {
                        match fun {
                            Function::Builtin(num) => {
                                return format!("<builtin function at index={}>", num);
                            }
                            Function::Defined(structure) => {
                                format_function(&structure)
                            }
                        }
                    }
                    Err(err) => {
                        panic!("Error deserializing function object: {err}")
                    }
                }
            }
            MiType::Class => {
                let class = deserialize::<Class>(&self.bytes);
                match class {
                    Ok(class) => {
                        format!("<class at {:?}>", &class as *const Class)
                    }
                    Err(err) => {
                        panic!("Error deserializing function object: {err}")
                    }
                }
            }
        }
    }
}

impl ToStringDebugged for MiValue {
    fn to_string_debugged(&self) -> String {
        match self.variant {
            MiType::Bool => {
                if self.bytes[0] == 1 {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            MiType::String => {
                let length: [u8; 4] = self.bytes[0..=4].try_into().unwrap();
                let len = u32::from_le_bytes(length);
                let mut string = String::new();
                string.push('"');
                for i in 0..len {
                    string.push(self.bytes[(i + 4) as usize] as char);
                }
                string.push('"');
                string
            }
            MiType::None => {
                "None".to_string()
            }
            MiType::Int => {
                let num = i32::from_le_bytes(self.bytes.clone().try_into().unwrap());
                format!("{}", num)
            }
            MiType::Float => {
                let num = f64::from_le_bytes(self.bytes.clone().try_into().unwrap());
                format!("{}", num)
            }
            MiType::Function => {
                let fun = deserialize::<Function>(&self.bytes);
                match fun {
                    Ok(fun) => {
                        match fun {
                            Function::Builtin(num) => {
                                return format!("<builtin function at index={}>", num);
                            }
                            Function::Defined(structure) => {
                                format_function(&structure)
                            }
                        }
                    }
                    Err(err) => {
                        panic!("Error deserializing function object: {err}")
                    }
                }
            }
            MiType::Class => {
                let class = deserialize::<Class>(&self.bytes);
                match class {
                    Ok(class) => {
                        class.format_debugged()
                    }
                    Err(err) => {
                        panic!("Error deserializing function object: {err}")
                    }
                }
            }
        }
    }
}