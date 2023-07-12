use std::io::{stdout, Write, stderr, stdin, StdoutLock, StderrLock};

use fxhash::FxHashMap;

use crate::registers::Registers;
use crate::instructions::Instruction;
use crate::value::{MiType, MiValue, ToStringDebugged, IntoValue};
use crate::result::MiError;
use crate::stack::{CallStack, StackFrame};

/// Represents the Mirage runtime (virtual machine)
pub struct MirageRuntime<'rtm> {
    pub registers: Registers,
    stack: CallStack,
    program_counter: i32,
    instructions: Vec<Instruction>,
    labels: FxHashMap<String, i32>,
    argument_stack: Vec<MiValue>,
    function_addr_table: FxHashMap<String, (Vec<String>, MiType, i32)>,
    stdout_lock: StdoutLock<'rtm>,
    stderr_lock: StderrLock<'rtm>,
}

impl<'rtm> MirageRuntime<'rtm> {
    /// Creates a new MirageRuntime instance
    pub fn new(instructions: Vec<Instruction>) -> MirageRuntime<'rtm> {
        Self {
            registers: Registers::new(),
            stack: CallStack::new(),
            program_counter: -1,
            instructions,
            labels: FxHashMap::default(),
            argument_stack: Vec::new(),
            function_addr_table: FxHashMap::default(),
            stdout_lock: stdout().lock(),
            stderr_lock: stderr().lock(),
        }
    }

    /// Prechecks the runtime's labels before running
    pub fn setup(&mut self) {
        for (pos, instruction) in self.instructions.iter().enumerate() {
            match instruction {
                Instruction::DefineLabel(label) => {
                    self.labels.insert(label.clone(), pos as i32);
                }
                Instruction::DefineFnLabel(name, args, returns) => {
                    self.function_addr_table.insert(name.clone(), (args.clone(), returns.clone(), pos as i32));
                }
                _ => continue,
            }
        }
    }

    /// Runs the virtual machine to its end
    pub fn run(&mut self) -> Result<Option<MiValue>, MiError> {
        self.stack.push_frame(StackFrame::new(
            String::from("Main"),
            FxHashMap::default(),
            None,
            false,
            0,
        )).unwrap();

        loop {
            self.program_counter += 1;
            let ins = self.get_current();
            // eprintln!("{:?}", self.get_current());
            match ins {
                Some(instruction) => {
                    match instruction {
                        Instruction::Move(reg, value) => {
                            self.registers.set(reg, value.clone())?;
                        }
                        Instruction::MoveBetween(src, dst) => {
                            match self.registers.get(src) {
                                Some(value) => {
                                    self.registers.set(dst, value.clone())?;
                                }
                                None => {
                                    self.program_counter = self.throw(
                                        "UnsetRegister",
                                        format!("The register `{src} has not been set yet.")
                                    )?;
                                    continue;
                                },
                            }
                        }
                        Instruction::MoveArgument(arg, reg) => {
                            match self.stack.last_frame_mut().unwrap().args.get(&arg) {
                                Some(value) => {
                                    self.registers.set(reg, value.clone())?;
                                }
                                None => {
                                    self.program_counter = self.throw(
                                        "UndefinedArgument",
                                        format!("The argument `{}` has not been defined yet.", arg)
                                    )?;
                                    continue;
                                }
                            }
                        }
                        Instruction::MoveAsArgument(reg) => {
                            match self.registers.get(reg) {
                                Some(value) => {
                                    self.argument_stack.push(value.clone());
                                }
                                None => {
                                    self.program_counter = self.throw(
                                        "UnsetRegister",
                                        format!("The register `{reg}` has not been set yet.")
                                    )?;
                                    continue;
                                }
                            }
                        }
                        Instruction::Add(op1, op2, dst) => {
                            match self.registers.get(op1).cloned() {
                                Some(op1) => {
                                    if !op1.variant.is_numeric() {
                                        self.program_counter = self.throw(
                                            "InvalidType",
                                            format!("The type `{:?}` is not numeric", op1.variant)
                                        )?;
                                        continue;
                                    }
                                    match self.registers.get(op2).cloned() {
                                        Some(op2) => {
                                            if !op2.variant.is_numeric() {
                                                self.program_counter = self.throw(
                                                    "InvalidType",
                                                    format!("The type `{:?}` is not numeric", op2.variant)
                                                )?;
                                                continue;
                                            }

                                            // Addition implementation here
                                            match op1.variant {
                                                MiType::Int => {
                                                    match op2.variant {
                                                        MiType::Int => {
                                                            let val1 = i32::from_le_bytes(op1.bytes.try_into().unwrap());
                                                            let val2 = i32::from_le_bytes(op2.bytes.try_into().unwrap());
                                                            self.registers.set(dst, MiValue {
                                                                bytes: (val1 + val2).to_le_bytes().to_vec(),
                                                                variant: MiType::Int,
                                                            })?;
                                                        }
                                                        _ => {
                                                            self.program_counter = self.throw(
                                                                "InvalidType",
                                                                format!("Cannot add two different types: `{:?}` and  `{:?}`", op1.variant, op2.variant)
                                                            )?;
                                                            continue;
                                                        }
                                                    }
                                                }
                                                MiType::Float => {
                                                    match op2.variant {
                                                        MiType::Float => {
                                                            let val1 = f64::from_le_bytes(op1.bytes.try_into().unwrap());
                                                            let val2 = f64::from_le_bytes(op2.bytes.try_into().unwrap());
                                                            self.registers.set(dst, MiValue {
                                                                bytes: (val1 + val2).to_le_bytes().to_vec(),
                                                                variant: MiType::Float,
                                                            })?;
                                                        }
                                                        _ => {
                                                            self.program_counter = self.throw(
                                                                "InvalidType",
                                                                format!("Cannot add two different types: `{:?}` and  `{:?}`", op1.variant, op2.variant)
                                                            )?;
                                                            continue;
                                                        }
                                                    }
                                                }
                                                _ => unreachable!()
                                            }
                                        }
                                        None => {
                                            self.program_counter = self.throw(
                                                "UnsetRegister",
                                                format!("The register `{op2}` has not been set yet.")
                                            )?;
                                            continue;
                                        },
                                    }
                                }
                                None => {
                                    self.program_counter = self.throw(
                                        "UnsetRegister",
                                        format!("The register `{op1}` has not been set yet.")
                                    )?;
                                    continue;
                                },
                            }
                        }
                        Instruction::Sub(op1, op2, dst) => {
                            match self.registers.get(op1).cloned() {
                                Some(op1) => {
                                    if !op1.variant.is_numeric() {
                                        self.program_counter = self.throw(
                                            "InvalidType",
                                            format!("The type `{:?}` is not numeric", op1.variant)
                                        )?;
                                        continue;
                                    }
                                    match self.registers.get(op2).cloned() {
                                        Some(op2) => {
                                            if !op2.variant.is_numeric() {
                                                self.program_counter = self.throw(
                                                    "InvalidType",
                                                    format!("The type `{:?}` is not numeric", op2.variant)
                                                )?;
                                                continue;
                                            }

                                            // Subtraction implementation here
                                            match op1.variant {
                                                MiType::Int => {
                                                    match op2.variant {
                                                        MiType::Int => {
                                                            let val1 = i32::from_le_bytes(op1.bytes.try_into().unwrap());
                                                            let val2 = i32::from_le_bytes(op2.bytes.try_into().unwrap());
                                                            self.registers.set(dst, MiValue {
                                                                bytes: (val1 - val2).to_le_bytes().to_vec(),
                                                                variant: MiType::Int,
                                                            })?;
                                                        }
                                                        _ => {
                                                            self.program_counter = self.throw(
                                                                "InvalidType",
                                                                format!("Cannot subtract two different types: `{:?}` and  `{:?}`", op1.variant, op2.variant)
                                                            )?;
                                                            continue;
                                                        }
                                                    }
                                                }
                                                MiType::Float => {
                                                    match op2.variant {
                                                        MiType::Float => {
                                                            let val1 = f64::from_le_bytes(op1.bytes.try_into().unwrap());
                                                            let val2 = f64::from_le_bytes(op2.bytes.try_into().unwrap());
                                                            self.registers.set(dst, MiValue {
                                                                bytes: (val1 - val2).to_le_bytes().to_vec(),
                                                                variant: MiType::Float,
                                                            })?;
                                                        }
                                                        _ => {
                                                            self.program_counter = self.throw(
                                                                "InvalidType",
                                                                format!("Cannot subtract two different types: `{:?}` and  `{:?}`", op1.variant, op2.variant)
                                                            )?;
                                                            continue;
                                                        }
                                                    }
                                                }
                                                _ => unreachable!()
                                            }
                                        }
                                        None => {
                                            self.program_counter = self.throw(
                                                "UnsetRegister",
                                                format!("The register `{op2}` has not been set yet.")
                                            )?;
                                            continue;
                                        },
                                    }
                                }
                                None => {
                                    self.program_counter = self.throw(
                                        "UnsetRegister",
                                        format!("The register `{op1}` has not been set yet.")
                                    )?;
                                    continue;
                                },
                            }
                        }
                        Instruction::Mul(op1, op2, dst) => {
                            match self.registers.get(op1).cloned() {
                                Some(op1) => {
                                    if !op1.variant.is_numeric() {
                                        self.program_counter = self.throw(
                                            "InvalidType",
                                            format!("The type `{:?}` is not numeric", op1.variant)
                                        )?;
                                        continue;
                                    }
                                    match self.registers.get(op2).cloned() {
                                        Some(op2) => {
                                            if !op2.variant.is_numeric() {
                                                self.program_counter = self.throw(
                                                    "InvalidType",
                                                    format!("The type `{:?}` is not numeric", op2.variant)
                                                )?;
                                                continue;
                                            }

                                            // Multiplication implementation here
                                            match op1.variant {
                                                MiType::Int => {
                                                    match op2.variant {
                                                        MiType::Int => {
                                                            let val1 = i32::from_le_bytes(op1.bytes.try_into().unwrap());
                                                            let val2 = i32::from_le_bytes(op2.bytes.try_into().unwrap());
                                                            self.registers.set(dst, MiValue {
                                                                bytes: (val1 * val2).to_le_bytes().to_vec(),
                                                                variant: MiType::Int,
                                                            })?;
                                                        }
                                                        _ => {
                                                            self.program_counter = self.throw(
                                                                "InvalidType",
                                                                format!("Cannot multiply two different types: `{:?}` and  `{:?}`", op1.variant, op2.variant)
                                                            )?;
                                                            continue;
                                                        }
                                                    }
                                                }
                                                MiType::Float => {
                                                    match op2.variant {
                                                        MiType::Float => {
                                                            let val1 = f64::from_le_bytes(op1.bytes.try_into().unwrap());
                                                            let val2 = f64::from_le_bytes(op2.bytes.try_into().unwrap());
                                                            self.registers.set(dst, MiValue {
                                                                bytes: (val1 * val2).to_le_bytes().to_vec(),
                                                                variant: MiType::Float,
                                                            })?;
                                                        }
                                                        _ => {
                                                            self.program_counter = self.throw(
                                                                "InvalidType",
                                                                format!("Cannot multiply two different types: `{:?}` and  `{:?}`", op1.variant, op2.variant)
                                                            )?;
                                                            continue;
                                                        }
                                                    }
                                                }
                                                _ => unreachable!()
                                            }
                                        }
                                        None => {
                                            eprintln!("Error is in mul reg2 handler");
                                            self.program_counter = self.throw(
                                                "UnsetRegister",
                                                format!("The register `{op2}` has not been set yet.")
                                            )?;
                                            continue;
                                        },
                                    }
                                }
                                None => {
                                    eprintln!("Error is in mul reg1 handler");
                                    self.program_counter = self.throw(
                                        "UnsetRegister",
                                        format!("The register `{op1}` has not been set yet.")
                                    )?;
                                    continue;
                                },
                            }
                        }
                        Instruction::Div(op1, op2, dst) => {
                            match self.registers.get(op1).cloned() {
                                Some(op1) => {
                                    if !op1.variant.is_numeric() {
                                        self.program_counter = self.throw(
                                            "InvalidType",
                                            format!("The type `{:?}` is not numeric", op1.variant)
                                        )?;
                                        continue;
                                    }
                                    match self.registers.get(op2).cloned() {
                                        Some(op2) => {
                                            if !op2.variant.is_numeric() {
                                                self.program_counter = self.throw(
                                                    "InvalidType",
                                                    format!("The type `{:?}` is not numeric", op2.variant)
                                                )?;
                                                continue;
                                            }

                                            // Division implementation here
                                            match op1.variant {
                                                MiType::Int => {
                                                    match op2.variant {
                                                        MiType::Int => {
                                                            let val1 = i32::from_le_bytes(op1.bytes.try_into().unwrap());
                                                            let val2 = i32::from_le_bytes(op2.bytes.try_into().unwrap());
                                                            self.registers.set(dst, MiValue {
                                                                bytes: (val1 / val2).to_le_bytes().to_vec(),
                                                                variant: MiType::Int,
                                                            })?;
                                                        }
                                                        _ => {
                                                            self.program_counter = self.throw(
                                                                "InvalidType",
                                                                format!("Cannot divide two different types: `{:?}` and  `{:?}`", op1.variant, op2.variant)
                                                            )?;
                                                            continue;
                                                        }
                                                    }
                                                }
                                                MiType::Float => {
                                                    match op2.variant {
                                                        MiType::Float => {
                                                            let val1 = f64::from_le_bytes(op1.bytes.try_into().unwrap());
                                                            let val2 = f64::from_le_bytes(op2.bytes.try_into().unwrap());
                                                            self.registers.set(dst, MiValue {
                                                                bytes: (val1 / val2).to_le_bytes().to_vec(),
                                                                variant: MiType::Float,
                                                            })?;
                                                        }
                                                        _ => {
                                                            self.program_counter = self.throw(
                                                                "InvalidType",
                                                                format!("Cannot divide two different types: `{:?}` and  `{:?}`", op1.variant, op2.variant)
                                                            )?;
                                                            continue;
                                                        }
                                                    }
                                                }
                                                _ => unreachable!()
                                            }
                                        }
                                        None => {
                                            self.program_counter = self.throw(
                                                "UnsetRegister",
                                                format!("The register `{op2}` has not been set yet.")
                                            )?;
                                            continue;
                                        },
                                    }
                                }
                                None => {
                                    self.program_counter = self.throw(
                                        "UnsetRegister",
                                        format!("The register `{op1}` has not been set yet.")
                                    )?;
                                    continue;
                                },
                            }
                        }
                        Instruction::Rem(op1, op2, dst) => {
                            match self.registers.get(op1).cloned() {
                                Some(op1) => {
                                    if !op1.variant.is_numeric() {
                                        self.program_counter = self.throw(
                                            "InvalidType",
                                            format!("The type `{:?}` is not numeric", op1.variant)
                                        )?;
                                        continue;
                                    }
                                    match self.registers.get(op2).cloned() {
                                        Some(op2) => {
                                            if !op2.variant.is_numeric() {
                                                self.program_counter = self.throw(
                                                    "InvalidType",
                                                    format!("The type `{:?}` is not numeric", op2.variant)
                                                )?;
                                                continue;
                                            }

                                            // Remainder implementation here
                                            match op1.variant {
                                                MiType::Int => {
                                                    match op2.variant {
                                                        MiType::Int => {
                                                            let val1 = i32::from_le_bytes(op1.bytes.try_into().unwrap());
                                                            let val2 = i32::from_le_bytes(op2.bytes.try_into().unwrap());
                                                            self.registers.set(dst, MiValue {
                                                                bytes: (val1 % val2).to_le_bytes().to_vec(),
                                                                variant: MiType::Int,
                                                            })?;
                                                        }
                                                        _ => {
                                                            self.program_counter = self.throw(
                                                                "InvalidType",
                                                                format!("Cannot rem two different types: `{:?}` and  `{:?}`", op1.variant, op2.variant)
                                                            )?;
                                                            continue;
                                                        }
                                                    }
                                                }
                                                MiType::Float => {
                                                    match op2.variant {
                                                        MiType::Float => {
                                                            let val1 = f64::from_le_bytes(op1.bytes.try_into().unwrap());
                                                            let val2 = f64::from_le_bytes(op2.bytes.try_into().unwrap());
                                                            self.registers.set(dst, MiValue {
                                                                bytes: (val1 % val2).to_le_bytes().to_vec(),
                                                                variant: MiType::Float,
                                                            })?;
                                                        }
                                                        _ => {
                                                            self.program_counter = self.throw(
                                                                "InvalidType",
                                                                format!("Cannot rem two different types: `{:?}` and  `{:?}`", op1.variant, op2.variant)
                                                            )?;
                                                            continue;
                                                        }
                                                    }
                                                }
                                                _ => unreachable!()
                                            }
                                        }
                                        None => {
                                            self.program_counter = self.throw(
                                                "UnsetRegister",
                                                format!("The register `{op2}` has not been set yet.")
                                            )?;
                                            continue;
                                        },
                                    }
                                }
                                None => {
                                    self.program_counter = self.throw(
                                        "UnsetRegister",
                                        format!("The register `{op1}` has not been set yet.")
                                    )?;
                                    continue;
                                },
                            }
                        }
                        Instruction::Pow(op1, op2, dst) => {
                            match self.registers.get(op1).cloned() {
                                Some(op1) => {
                                    if !op1.variant.is_numeric() {
                                        self.program_counter = self.throw(
                                            "InvalidType",
                                            format!("The type `{:?}` is not numeric", op1.variant)
                                        )?;
                                        continue;
                                    }
                                    match self.registers.get(op2).cloned() {
                                        Some(op2) => {
                                            if !op2.variant.is_numeric() {
                                                self.program_counter = self.throw(
                                                    "InvalidType",
                                                    format!("The type `{:?}` is not numeric", op2.variant)
                                                )?;
                                                continue;
                                            }

                                            // Power implementation here
                                            match op1.variant {
                                                MiType::Int => {
                                                    match op2.variant {
                                                        MiType::Int => {
                                                            let val1 = i32::from_le_bytes(op1.bytes.try_into().unwrap());
                                                            let val2 = i32::from_le_bytes(op2.bytes.try_into().unwrap());
                                                            if val2 < 0 {
                                                                self.program_counter = self.throw(
                                                                    "MathError",
                                                                    format!("The exponent `{val2}` is not valid as it needs to be positive")
                                                                )?;
                                                                continue;
                                                            }
                                                            self.registers.set(dst, MiValue {
                                                                bytes: (val1.pow(val2 as u32)).to_le_bytes().to_vec(),
                                                                variant: MiType::Int,
                                                            })?;
                                                        }
                                                        _ => {
                                                            self.program_counter = self.throw(
                                                                "InvalidType",
                                                                format!("Cannot power two different types: `{:?}` and  `{:?}`", op1.variant, op2.variant)
                                                            )?;
                                                            continue;
                                                        }
                                                    }
                                                }
                                                MiType::Float => {
                                                    match op2.variant {
                                                        MiType::Float => {
                                                            let val1 = f64::from_le_bytes(op1.bytes.try_into().unwrap());
                                                            let val2 = f64::from_le_bytes(op2.bytes.try_into().unwrap());
                                                            self.registers.set(dst, MiValue {
                                                                bytes: (val1.powf(val2)).to_le_bytes().to_vec(),
                                                                variant: MiType::Float,
                                                            })?;
                                                        }
                                                        _ => {
                                                            self.program_counter = self.throw(
                                                                "InvalidType",
                                                                format!("Cannot add two different types: `{:?}` and  `{:?}`", op1.variant, op2.variant)
                                                            )?;
                                                            continue;
                                                        }
                                                    }
                                                }
                                                _ => unreachable!()
                                            }
                                        }
                                        None => {
                                            self.program_counter = self.throw(
                                                "UnsetRegister",
                                                format!("The register `{op2}` has not been set yet.")
                                            )?;
                                            continue;
                                        },
                                    }
                                }
                                None => {
                                    self.program_counter = self.throw(
                                        "UnsetRegister",
                                        format!("The register `{op1}` has not been set yet.")
                                    )?;
                                    continue;
                                },
                            }
                        }
                        Instruction::Or(op1, op2, dst) => {
                            match self.registers.get(op1).cloned() {
                                Some(op1) => {
                                    if op1.variant != MiType::Bool {
                                        self.program_counter = self.throw(
                                            "InvalidType",
                                            format!("The type `{:?}` is not boolean", op1.variant)
                                        )?;
                                        continue;
                                    }
                                    match self.registers.get(op2).cloned() {
                                        Some(op2) => {
                                            if op1.variant != MiType::Bool {
                                                self.program_counter = self.throw(
                                                    "InvalidType",
                                                    format!("The type `{:?}` is not boolean", op2.variant)
                                                )?;
                                                continue;
                                            }

                                            // Or implementation here
                                            let b1: bool = op1.bytes[0] != 0;
                                            let b2: bool = op2.bytes[0] != 0;
                                            self.registers.set(dst, MiValue {
                                                bytes: if b1 || b2 { [1].to_vec() } else { [0].to_vec() },
                                                variant: MiType::Bool,
                                            })?;
                                        }
                                        None => {
                                            self.program_counter = self.throw(
                                                "UnsetRegister",
                                                format!("The register `{op2}` has not been set yet.")
                                            )?;
                                            continue;
                                        },
                                    }
                                }
                                None => {
                                    self.program_counter = self.throw(
                                        "UnsetRegister",
                                        format!("The register `{op1}` has not been set yet.")
                                    )?;
                                    continue;
                                },
                            }
                        }
                        Instruction::Xor(op1, op2, dst) => {
                            match self.registers.get(op1).cloned() {
                                Some(op1) => {
                                    if op1.variant != MiType::Bool {
                                        self.program_counter = self.throw(
                                            "InvalidType",
                                            format!("The type `{:?}` is not boolean", op1.variant)
                                        )?;
                                        continue;
                                    }
                                    match self.registers.get(op2).cloned() {
                                        Some(op2) => {
                                            if op1.variant != MiType::Bool {
                                                self.program_counter = self.throw(
                                                    "InvalidType",
                                                    format!("The type `{:?}` is not boolean", op2.variant)
                                                )?;
                                                continue;
                                            }

                                            // Or implementation here
                                            let b1: bool = op1.bytes[0] != 0;
                                            let b2: bool = op2.bytes[0] != 0;
                                            self.registers.set(dst, MiValue {
                                                bytes: if b1 ^ b2 { [1].to_vec() } else { [0].to_vec() },
                                                variant: MiType::Bool,
                                            })?;
                                        }
                                        None => {
                                            self.program_counter = self.throw(
                                                "UnsetRegister",
                                                format!("The register `{op2}` has not been set yet.")
                                            )?;
                                            continue;
                                        },
                                    }
                                }
                                None => {
                                    self.program_counter = self.throw(
                                        "UnsetRegister",
                                        format!("The register `{op1}` has not been set yet.")
                                    )?;
                                    continue;
                                },
                            }
                        }
                        Instruction::And(op1, op2, dst) => {
                            match self.registers.get(op1).cloned() {
                                Some(op1) => {
                                    if op1.variant != MiType::Bool {
                                        self.program_counter = self.throw(
                                            "InvalidType",
                                            format!("The type `{:?}` is not boolean", op1.variant)
                                        )?;
                                        continue;
                                    }
                                    match self.registers.get(op2).cloned() {
                                        Some(op2) => {
                                            if op1.variant != MiType::Bool {
                                                self.program_counter = self.throw(
                                                    "InvalidType",
                                                    format!("The type `{:?}` is not boolean", op2.variant)
                                                )?;
                                                continue;
                                            }

                                            // Or implementation here
                                            let b1: bool = op1.bytes[0] != 0;
                                            let b2: bool = op2.bytes[0] != 0;
                                            self.registers.set(dst, MiValue {
                                                bytes: if b1 && b2 { [1].to_vec() } else { [0].to_vec() },
                                                variant: MiType::Bool,
                                            })?;
                                        }
                                        None => {
                                            self.program_counter = self.throw(
                                                "UnsetRegister",
                                                format!("The register `{op2}` has not been set yet.")
                                            )?;
                                            continue;
                                        },
                                    }
                                }
                                None => {
                                    self.program_counter = self.throw(
                                        "UnsetRegister",
                                        format!("The register `{op1}` has not been set yet.")
                                    )?;
                                    continue;
                                },
                            }
                        }
                        Instruction::Not(src, dst) => {
                            match self.registers.get(src).cloned() {
                                Some(op1) => {
                                    if op1.variant != MiType::Bool {
                                        self.program_counter = self.throw(
                                            "InvalidType",
                                            format!("The type `{:?}` is not boolean", op1.variant)
                                        )?;
                                        continue;
                                    }
                                    let b1: bool = op1.bytes[0] != 0;
                                    self.registers.set(dst, MiValue {
                                        bytes: if !b1 { [1].to_vec() } else { [0].to_vec() },
                                        variant: MiType::Bool,
                                    })?;
                                }
                                None => {
                                    self.program_counter = self.throw(
                                        "UnsetRegister",
                                        format!("The register `{src} has not been set yet.")
                                    )?;
                                    continue;
                                },
                            }
                        }
                        Instruction::Lt(op1, op2, dst) => {
                            match self.registers.get(op1).cloned() {
                                Some(op1) => {
                                    if !op1.variant.is_numeric() {
                                        self.program_counter = self.throw(
                                            "InvalidType",
                                            format!("The type `{:?}` is not numeric", op1.variant)
                                        )?;
                                        continue;
                                    }
                                    match self.registers.get(op2).cloned() {
                                        Some(op2) => {
                                            if !op2.variant.is_numeric() {
                                                self.program_counter = self.throw(
                                                    "InvalidType",
                                                    format!("The type `{:?}` is not numeric", op2.variant)
                                                )?;
                                                continue;
                                            }

                                            // Power implementation here
                                            match op1.variant {
                                                MiType::Int => {
                                                    match op2.variant {
                                                        MiType::Int => {
                                                            let val1 = i32::from_le_bytes(op1.bytes.try_into().unwrap());
                                                            let val2 = i32::from_le_bytes(op2.bytes.try_into().unwrap());
                                                            self.registers.set(dst, MiValue {
                                                                bytes: vec![(val1 < val2) as u8],
                                                                variant: MiType::Bool,
                                                            })?;
                                                        }
                                                        _ => {
                                                            self.program_counter = self.throw(
                                                                "InvalidType",
                                                                format!("Cannot LT two different types: `{:?}` and  `{:?}`", op1.variant, op2.variant)
                                                            )?;
                                                            continue;
                                                        }
                                                    }
                                                }
                                                MiType::Float => {
                                                    match op2.variant {
                                                        MiType::Float => {
                                                            let val1 = f64::from_le_bytes(op1.bytes.try_into().unwrap());
                                                            let val2 = f64::from_le_bytes(op2.bytes.try_into().unwrap());
                                                            self.registers.set(dst, MiValue {
                                                                bytes: vec![(val1 < val2) as u8],
                                                                variant: MiType::Bool,
                                                            })?;
                                                        }
                                                        _ => {
                                                            self.program_counter = self.throw(
                                                                "InvalidType",
                                                                format!("Cannot LT two different types: `{:?}` and  `{:?}`", op1.variant, op2.variant)
                                                            )?;
                                                            continue;
                                                        }
                                                    }
                                                }
                                                _ => unreachable!()
                                            }
                                        }
                                        None => {
                                            self.program_counter = self.throw(
                                                "UnsetRegister",
                                                format!("The register `{op2}` has not been set yet.")
                                            )?;
                                            continue;
                                        },
                                    }
                                }
                                None => {
                                    self.program_counter = self.throw(
                                        "UnsetRegister",
                                        format!("The register `{op1}` has not been set yet.")
                                    )?;
                                    continue;
                                },
                            }
                        }
                        Instruction::Le(op1, op2, dst) => {
                            match self.registers.get(op1).cloned() {
                                Some(op1) => {
                                    if !op1.variant.is_numeric() {
                                        self.program_counter = self.throw(
                                            "InvalidType",
                                            format!("The type `{:?}` is not numeric", op1.variant)
                                        )?;
                                        continue;
                                    }
                                    match self.registers.get(op2).cloned() {
                                        Some(op2) => {
                                            if !op2.variant.is_numeric() {
                                                self.program_counter = self.throw(
                                                    "InvalidType",
                                                    format!("The type `{:?}` is not numeric", op2.variant)
                                                )?;
                                                continue;
                                            }

                                            // Power implementation here
                                            match op1.variant {
                                                MiType::Int => {
                                                    match op2.variant {
                                                        MiType::Int => {
                                                            let val1 = i32::from_le_bytes(op1.bytes.try_into().unwrap());
                                                            let val2 = i32::from_le_bytes(op2.bytes.try_into().unwrap());
                                                            self.registers.set(dst, MiValue {
                                                                bytes: vec![(val1 <= val2) as u8],
                                                                variant: MiType::Bool,
                                                            })?;
                                                        }
                                                        _ => {
                                                            self.program_counter = self.throw(
                                                                "InvalidType",
                                                                format!("Cannot LE two different types: `{:?}` and  `{:?}`", op1.variant, op2.variant)
                                                            )?;
                                                            continue;
                                                        }
                                                    }
                                                }
                                                MiType::Float => {
                                                    match op2.variant {
                                                        MiType::Float => {
                                                            let val1 = f64::from_le_bytes(op1.bytes.try_into().unwrap());
                                                            let val2 = f64::from_le_bytes(op2.bytes.try_into().unwrap());
                                                            self.registers.set(dst, MiValue {
                                                                bytes: vec![(val1 <= val2) as u8],
                                                                variant: MiType::Bool,
                                                            })?;
                                                        }
                                                        _ => {
                                                            self.program_counter = self.throw(
                                                                "InvalidType",
                                                                format!("Cannot LE two different types: `{:?}` and  `{:?}`", op1.variant, op2.variant)
                                                            )?;
                                                            continue;
                                                        }
                                                    }
                                                }
                                                _ => unreachable!()
                                            }
                                        }
                                        None => {
                                            self.program_counter = self.throw(
                                                "UnsetRegister",
                                                format!("The register `{op2}` has not been set yet.")
                                            )?;
                                            continue;
                                        },
                                    }
                                }
                                None => {
                                    self.program_counter = self.throw(
                                        "UnsetRegister",
                                        format!("The register `{op1}` has not been set yet.")
                                    )?;
                                    continue;
                                },
                            }
                        }
                        Instruction::Gt(op1, op2, dst) => {
                            match self.registers.get(op1).cloned() {
                                Some(op1) => {
                                    if !op1.variant.is_numeric() {
                                        self.program_counter = self.throw(
                                            "InvalidType",
                                            format!("The type `{:?}` is not numeric", op1.variant)
                                        )?;
                                        continue;
                                    }
                                    match self.registers.get(op2).cloned() {
                                        Some(op2) => {
                                            if !op2.variant.is_numeric() {
                                                self.program_counter = self.throw(
                                                    "InvalidType",
                                                    format!("The type `{:?}` is not numeric", op2.variant)
                                                )?;
                                                continue;
                                            }

                                            // Power implementation here
                                            match op1.variant {
                                                MiType::Int => {
                                                    match op2.variant {
                                                        MiType::Int => {
                                                            let val1 = i32::from_le_bytes(op1.bytes.try_into().unwrap());
                                                            let val2 = i32::from_le_bytes(op2.bytes.try_into().unwrap());
                                                            self.registers.set(dst, MiValue {
                                                                bytes: vec![(val1 > val2) as u8],
                                                                variant: MiType::Bool,
                                                            })?;
                                                        }
                                                        _ => {
                                                            self.program_counter = self.throw(
                                                                "InvalidType",
                                                                format!("Cannot GT two different types: `{:?}` and  `{:?}`", op1.variant, op2.variant)
                                                            )?;
                                                            continue;
                                                        }
                                                    }
                                                }
                                                MiType::Float => {
                                                    match op2.variant {
                                                        MiType::Float => {
                                                            let val1 = f64::from_le_bytes(op1.bytes.try_into().unwrap());
                                                            let val2 = f64::from_le_bytes(op2.bytes.try_into().unwrap());
                                                            self.registers.set(dst, MiValue {
                                                                bytes: vec![(val1 > val2) as u8],
                                                                variant: MiType::Bool,
                                                            })?;
                                                        }
                                                        _ => {
                                                            self.program_counter = self.throw(
                                                                "InvalidType",
                                                                format!("Cannot FT two different types: `{:?}` and  `{:?}`", op1.variant, op2.variant)
                                                            )?;
                                                            continue;
                                                        }
                                                    }
                                                }
                                                _ => unreachable!()
                                            }
                                        }
                                        None => {
                                            self.program_counter = self.throw(
                                                "UnsetRegister",
                                                format!("The register `{op2}` has not been set yet.")
                                            )?;
                                            continue;
                                        },
                                    }
                                }
                                None => {
                                    self.program_counter = self.throw(
                                        "UnsetRegister",
                                        format!("The register `{op1}` has not been set yet.")
                                    )?;
                                    continue;
                                },
                            }
                        }
                        Instruction::Ge(op1, op2, dst) => {
                            match self.registers.get(op1).cloned() {
                                Some(op1) => {
                                    if !op1.variant.is_numeric() {
                                        self.program_counter = self.throw(
                                            "InvalidType",
                                            format!("The type `{:?}` is not numeric", op1.variant)
                                        )?;
                                        continue;
                                    }
                                    match self.registers.get(op2).cloned() {
                                        Some(op2) => {
                                            if !op2.variant.is_numeric() {
                                                self.program_counter = self.throw(
                                                    "InvalidType",
                                                    format!("The type `{:?}` is not numeric", op2.variant)
                                                )?;
                                                continue;
                                            }

                                            // Power implementation here
                                            match op1.variant {
                                                MiType::Int => {
                                                    match op2.variant {
                                                        MiType::Int => {
                                                            let val1 = i32::from_le_bytes(op1.bytes.try_into().unwrap());
                                                            let val2 = i32::from_le_bytes(op2.bytes.try_into().unwrap());
                                                            self.registers.set(dst, MiValue {
                                                                bytes: vec![(val1 >= val2) as u8],
                                                                variant: MiType::Bool,
                                                            })?;
                                                        }
                                                        _ => {
                                                            self.program_counter = self.throw(
                                                                "InvalidType",
                                                                format!("Cannot GE two different types: `{:?}` and  `{:?}`", op1.variant, op2.variant)
                                                            )?;
                                                            continue;
                                                        }
                                                    }
                                                }
                                                MiType::Float => {
                                                    match op2.variant {
                                                        MiType::Float => {
                                                            let val1 = f64::from_le_bytes(op1.bytes.try_into().unwrap());
                                                            let val2 = f64::from_le_bytes(op2.bytes.try_into().unwrap());
                                                            self.registers.set(dst, MiValue {
                                                                bytes: vec![(val1 >= val2) as u8],
                                                                variant: MiType::Bool,
                                                            })?;
                                                        }
                                                        _ => {
                                                            self.program_counter = self.throw(
                                                                "InvalidType",
                                                                format!("Cannot GE two different types: `{:?}` and  `{:?}`", op1.variant, op2.variant)
                                                            )?;
                                                            continue;
                                                        }
                                                    }
                                                }
                                                _ => unreachable!()
                                            }
                                        }
                                        None => {
                                            self.program_counter = self.throw(
                                                "UnsetRegister",
                                                format!("The register `{op2}` has not been set yet.")
                                            )?;
                                            continue;
                                        },
                                    }
                                }
                                None => {
                                    self.program_counter = self.throw(
                                        "UnsetRegister",
                                        format!("The register `{op1}` has not been set yet.")
                                    )?;
                                    continue;
                                },
                            }
                        }
                        Instruction::Return => {
                            match self.stack.pop_frame() {
                                Some(frame) => match frame.return_addr {
                                    Some(addr) => {
                                        self.program_counter = addr as i32;
                                    }
                                    None => {
                                        return Ok(self.registers.get(15).cloned());
                                    }
                                }
                                None => panic!("No frame to return to")
                            }
                        }
                        Instruction::SetVariable(reg, name) => {
                            match self.registers.get(reg) {
                                Some(value) => {
                                    let frame = self.stack.last_frame_mut();
                                    match frame {
                                        Some(frame) => {
                                            frame.local_variables.insert(name, value.clone());
                                        }
                                        None => {
                                            panic!("Current frame is not valid")
                                        }
                                    }
                                }
                                None => {
                                    self.program_counter = self.throw(
                                        "UnsetRegister",
                                        format!("The register `{}` is not valid.", reg)
                                    )?;
                                    continue;
                                }
                            }
                        }
                        Instruction::MovFromVariable(name, reg) => {
                            let frame = self.stack.last_frame_mut();
                            match frame {
                                Some(frame) => {
                                    let var = frame.local_variables.get(&name);
                                    match var {
                                        Some(value) => {
                                            self.registers.set(reg, value.clone())?;
                                        }
                                        None => {
                                            self.program_counter = self.throw(
                                                "UndefinedVariable",
                                                format!("Cannot move value of variable `{}` to register `{}` because `{}` is not defined.", &name, reg, name)
                                            )?;
                                            continue;
                                        }
                                    }
                                }
                                None => {
                                    panic!("Current frame is not valid")
                                }
                            }
                        }
                        Instruction::ThrowFrom(reason_reg, msg_reg) => {
                            match self.registers.get(reason_reg) {
                                Some(value) => {
                                    let reason = String::from_utf8_lossy(&value.bytes).to_string();
                                    match self.registers.get(msg_reg) {
                                        Some(value) => {
                                            let msg = String::from_utf8_lossy(&value.bytes).to_string();
                                            self.program_counter = self.throw(reason, msg)?;
                                            continue;
                                        }
                                        None => {
                                            self.program_counter = self.throw(
                                                "UnsetRegister",
                                                format!("The register `{msg_reg}` has not been set yet.")
                                            )?;
                                            continue;
                                        }
                                    }
                                }
                                None => {
                                    self.program_counter = self.throw(
                                        "UnsetRegister",
                                        format!("The register `{reason_reg}` has not been set yet.")
                                    )?;
                                    continue;
                                }
                            }
                        }
                        Instruction::Eq(op1, op2, dst) => {
                            match self.registers.get(op1) {
                                Some(op1) => {
                                    match self.registers.get(op2) {
                                        Some(op2) => {
                                            self.registers.set(dst, MiValue {
                                                bytes: vec![(op1 == op2) as u8],
                                                variant: MiType::Bool,
                                            })?;
                                        }
                                        None => {
                                            self.program_counter = self.throw(
                                                "UnsetRegister",
                                                format!("The register `{op2}` has not been set yet.")
                                            )?;
                                            continue;
                                        }
                                    }
                                }
                                None => {
                                    self.program_counter = self.throw(
                                        "UnsetRegister",
                                        format!("The register `{op1}` has not been set yet.")
                                    )?;
                                    continue;
                                }
                            }
                        }
                        Instruction::Ne(op1, op2, dst) => {
                            match self.registers.get(op1) {
                                Some(op1) => {
                                    match self.registers.get(op2) {
                                        Some(op2) => {
                                            self.registers.set(dst, MiValue {
                                                bytes: vec![(op1 != op2) as u8],
                                                variant: MiType::Bool,
                                            })?;
                                        }
                                        None => {
                                            self.program_counter = self.throw(
                                                "UnsetRegister",
                                                format!("The register `{op2}` has not been set yet.")
                                            )?;
                                            continue;
                                        }
                                    }
                                }
                                None => {
                                    self.program_counter = self.throw(
                                        "UnsetRegister",
                                        format!("The register `{op1}` has not been set yet.")
                                    )?;
                                    continue;
                                }
                            }
                        }
                        Instruction::DefineLabel(name) => {
                            continue;
                        }
                        Instruction::JumpUnconditional(name) => {
                            if let Some(label_pos) = self.labels.get(&name) {
                                self.program_counter = label_pos + 1;
                            } else {
                                self.program_counter = self.throw(
                                    "UnsetLabel",
                                    format!("The label `{name}` is currently not defined.")
                                )?;
                                continue;
                            }
                        }
                        Instruction::JumpConditional(reg, name) => {
                            if let Some(value) = self.registers.get(reg) {
                                if value.bytes[0] == 1u8 {
                                    if let Some(label_pos) = self.labels.get(&name) {
                                        self.program_counter = label_pos + 1;
                                    } else {
                                        self.program_counter = self.throw(
                                            "UnsetLabel",
                                            format!("The label `{name}` is currently not defined.")
                                        )?;
                                        continue;
                                    }
                                }
                            } else {
                                self.program_counter = self.throw(
                                    "UnsetRegister",
                                    format!("The register `{reg}` has not been set yet.")
                                )?;
                                continue;
                            }
                        }
                        Instruction::Call(name) => {
                            let funname = name.clone();
                            let fun = self.function_addr_table.get(&name).cloned();
                            match fun {
                                Some((args_names, _, real_label)) => {
                                    let mut args_hash = FxHashMap::default();
                                    let mut reversed_names = args_names.iter().rev().collect::<Vec<_>>();
                                    while let Some(name) = reversed_names.pop() {
                                        let value = self.argument_stack.pop();
                                        if let Some(value) = value {
                                            args_hash.insert(name.clone(), value);
                                        } else {
                                            self.program_counter = self.throw(
                                                "NotEnoughArguments",
                                                format!("Cannot satisfy the arguments size for the function `{}`: {}", &funname, args_names.len())
                                            )?;
                                            continue;
                                        }
                                    }
                                    let has_overflowed: Result<(), String> = self.stack.push_frame(StackFrame {
                                        name: name.clone(),
                                        args: args_hash,
                                        local_variables: FxHashMap::default(),
                                        return_addr: Some((self.program_counter + 1) as usize),
                                        handles_error: false,
                                        error_handling_addr: 0
                                    });
                                    self.program_counter = real_label;
                                    if let Err(err) = has_overflowed {
                                        self.program_counter = self.throw(
                                            "StackOverflow",
                                            err,
                                        )?;
                                    }
                                
                                    continue;
                                }
                                None => {
                                    self.program_counter = self.throw(
                                        "UndefinedFunction",
                                        format!("Cannot call undefined function `{name}`")
                                    )?;
                                    continue;
                                }
                            }
                        }
                        Instruction::DefineFnLabel(name, args, returns) => {
                            while let Some(instruction) = self.instructions.get((self.program_counter + 1) as usize) {
                                self.program_counter += 1;
                                match instruction {
                                    Instruction::EndFunction => break,
                                    _ => continue,
                                }
                            }
                        }
                        Instruction::StdoutWrite(reg) => {
                            match self.registers.get(reg) {
                                Some(value) => {
                                    let res = write!(self.stdout_lock, "{}", value.to_string());
                                    match res {
                                        Ok(_) => {
                                            continue;
                                        }
                                        Err(err) => {
                                            self.program_counter = self.throw(
                                                "IOError",
                                                format!("Error writing to stderr: {}", err)
                                            )?;
                                            continue;
                                        }
                                    }
                                }
                                None => {
                                    self.program_counter = self.throw(
                                        "UnsetRegister",
                                        format!("The register `{reg}` has not been set yet.")
                                    )?;
                                    continue;
                                }
                            }
                        }
                        Instruction::StdoutWriteDebugged(reg) => {
                            match self.registers.get(reg) {
                                Some(value) => {
                                    let res = write!(self.stdout_lock, "{}", value.to_string_debugged());
                                    match res {
                                        Ok(_) => {
                                            continue;
                                        }
                                        Err(err) => {
                                            self.program_counter = self.throw(
                                                "IOError",
                                                format!("Error writing to stderr: {}", err)
                                            )?;
                                            continue;
                                        }
                                    }
                                }
                                None => {
                                    self.program_counter = self.throw(
                                        "UnsetRegister",
                                        format!("The register `{reg}` has not been set yet.")
                                    )?;
                                    continue;
                                }
                            }
                        }
                        Instruction::StdoutFlush => {
                            stdout().flush().unwrap();
                        }
                        Instruction::StderrWrite(reg) => {
                            match self.registers.get(reg) {
                                Some(value) => {
                                    let res = write!(self.stderr_lock, "{}", value.to_string());
                                    match res {
                                        Ok(_) => {
                                            continue;
                                        }
                                        Err(err) => {
                                            self.program_counter = self.throw(
                                                "IOError",
                                                format!("Error writing to stderr: {}", err)
                                            )?;
                                            continue;
                                        }
                                    }
                                }
                                None => {
                                    self.program_counter = self.throw(
                                        "UnsetRegister",
                                        format!("The register `{reg}` has not been set yet.")
                                    )?;
                                    continue;
                                }
                            }
                        }
                        Instruction::StderrWriteDebugged(reg) => {
                            match self.registers.get(reg) {
                                Some(value) => {
                                    let res = write!(self.stderr_lock, "{}", value.to_string_debugged());
                                    match res {
                                        Ok(_) => {
                                            continue;
                                        }
                                        Err(err) => {
                                            self.program_counter = self.throw(
                                                "IOError",
                                                format!("Error writing to stderr: {}", err)
                                            )?;
                                            continue;
                                        }
                                    }
                                }
                                None => {
                                    self.program_counter = self.throw(
                                        "UnsetRegister",
                                        format!("The register `{reg}` has not been set yet.")
                                    )?;
                                    continue;
                                }
                            }
                        }
                        Instruction::StderrFlush => {
                            stderr().flush().unwrap();
                        }
                        Instruction::BufferedStdinRead(reg) => {
                            let mut buf = String::new();
                            let line = stdin().read_line(&mut buf);
                            match line {
                                Ok(_) => {
                                    self.registers.set(reg, buf.into_value())?;
                                }
                                Err(err) => {
                                    self.program_counter = self.throw(
                                        "IOError",
                                        format!("Unable to read a line from stdin: {}", err)
                                    )?;
                                    continue;
                                }
                            }
                        }
                        Instruction::EndFunction => {
                            continue;
                        }
                    }
                }
                None => break,
            }
        }

        return Ok(self.registers.get(15).cloned())
    }

    /// Returns an `Option<Instruction>` representing the current instruction according to the current program counter.
    pub fn get_current(&mut self) -> Option<Instruction> {
        let val = self.instructions.get(self.program_counter as usize);
        match val {
            Some(ins) => Some(ins.clone()),
            None => None,
        }
    }

    /// Unwinds the stack frames looking for an error handler
    pub fn unwind_stack(&mut self, error: MiError) -> Result<i32, MiError> {
        while let Some(frame) = self.stack.pop_frame() {
            if frame.handles_error {
                return Ok(frame.error_handling_addr as i32)
            }
        }
        Err(error)
    }

    /// Gets the stack backtrace
    pub fn get_backtrace(&self) -> String {
        self.stack.get_backtrace_string()
    }

    /// Throws an error
    pub fn throw<T: ToString, T2: ToString>(&mut self, name: T, message: T2) -> Result<i32, MiError> {
        let error = MiError {
            name: name.to_string(),
            message: message.to_string(),
            backtrace: self.get_backtrace(),
        };
        let res = self.unwind_stack(error);
        res
    }
}