use crate::{value::MiValue, result::MiError};

#[derive(Clone, PartialEq, Debug)]
pub struct Registers {
    registers: [Option<MiValue>; 16],
}

impl Registers {
    pub fn new() -> Self {
        Self { registers: [None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None] }
    }

    pub fn get(&self, index: usize) -> Option<&MiValue> {
        self.registers.get(index).and_then(|v| v.as_ref())
    }

    pub fn set(&mut self, index: usize, value: MiValue) -> Result<(), MiError> {
        if let Some(register) = self.registers.get_mut(index) {
            *register = Some(value);
            Ok(())
        } else {
            return Err(MiError {
                name: "InvalidRegister".to_string(),
                message: format!("The register `{}` is not valid as is not between 0-15", index),
                backtrace: "".to_string(),
            })
        }
    }
}