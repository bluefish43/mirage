use serde_derive::{Serialize, Deserialize};

use crate::value::{MiValue, MiType};

/// Represents the instructions the program will run
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Instruction {
    // ########### Register operations
    /// Moves a value to one register
    Move(usize, MiValue),

    /// Moves a value from one register to another
    /// 
    /// SRC - DST
    MoveBetween(usize, usize),

    /// Moves an argument from the caller to a register
    /// 
    /// ARGUMENT - DST
    MoveArgument(String, usize),

    /// Moves the value stored at the specified register to the arguments stack
    /// 
    /// SRC 
    MoveAsArgument(usize),

    // ########### Math operations
    /// Adds two registers and stores the result in the last specified register
    /// 
    /// OP1 - OP2 - DST
    Add(usize, usize, usize),

    /// Subtracts two registers and stores the result in the last specified register
    /// 
    /// OP1 - OP2 - DST
    Sub(usize, usize, usize),

    /// Multiplies two registers and stores the result in the last specified register
    /// 
    /// OP1 - OP2 - DST
    Mul(usize, usize, usize),

    /// Divides two registers and stores the result in the last specified register
    /// 
    /// OP1 - OP2 - DST
    Div(usize, usize, usize),

    /// Applies the remainder operator in two registers and stores the result in the last specified register
    /// 
    /// OP1 - OP2 - DST
    Rem(usize, usize, usize),

    /// Applies the power operator to two registers and stores the result in the last specified register
    /// 
    /// OP1 - OP2 - DST
    Pow(usize, usize, usize),

    // ########### Numerical logic operations
    /// Applies the logical OR operation to two registers and stores the result in the last specified register
    /// 
    /// OP1 - OP2 - DST
    Or(usize, usize, usize),

    /// Applies the logical XOR operation to two registers and stores the result in the last specified register
    /// 
    /// OP1 - OP2 - DST
    Xor(usize, usize, usize),

    /// Applies the logical AND operation to two registers and stores the result in the last specified register
    /// 
    /// OP1 - OP2 - DST
    And(usize, usize, usize),

    /// Applies the logical NOT operation to two registers and stores the result in the last specified register
    /// 
    /// OP1 - DST
    Not(usize, usize),

    // ########### Math operations
    /// Applies the logical LT (less than) operation to two registers and stores the result in the last specified register
    /// 
    /// OP1 - OP2 - DST
    Lt(usize, usize, usize),

    /// Applies the logical LE (less than or equal) operation to two registers and stores the result in the last specified register
    /// 
    /// OP1 - OP2 - DST
    Le(usize, usize, usize),

    /// Applies the logical GT (greater than) operation to two registers and stores the result in the last specified register
    /// 
    /// OP1 - OP2 - DST
    Gt(usize, usize, usize),

    /// Applies the logical GE (greater than or equal) operation to two registers and stores the result in the last specified register
    /// 
    /// OP1 - OP2 - DST'
    Ge(usize, usize, usize),

    // ########### Other operations
    /// Returns from the current stack frame
    Return,

    /// Sets the value of the specified variable to the value stored at the register specified.
    /// If the register's value is not set, an error is returned.
    SetVariable(usize, String),

    /// Moves the value from the local variable specified to the specified register.
    MovFromVariable(String, usize),

    /// Gets the stringified version of a value at the specified register and then throws it
    /// 
    /// TYPE - MESSAGE
    ThrowFrom(usize, usize),

    /// Applies the logical EQ (equal) operation to two registers and stores the result in the last specified register
    /// 
    /// OP1 - OP2 - DST
    Eq(usize, usize, usize),

    /// Applies the logical NE (not equal) operation to two registers and stores the result in the last specified register
    /// 
    ///O P1 - OP2 - DST
    Ne(usize, usize, usize),

    /// Defines a label of the current instruction
    DefineLabel(String),

    /// Jumps to a label unconditionally
    JumpUnconditional(String),

    /// Jumps to a label conditionally if the stored value of
    /// the specified register has its first byte as one
    JumpConditional(usize, String),

    /// Calls the specified label
    /// 
    /// Function label
    Call(String),

    /// Defines the specified function label
    /// 
    /// Function name - Arguments names - Return type - Normal label at pos
    DefineFnLabel(String, Vec<String>, MiType),

    EndFunction,

    /// Writes a value to the Stdout
    StdoutWrite(usize),

    /// Writes a debugged file to the Stdout
    StdoutWriteDebugged(usize),

    /// Flushes the Stdout
    StdoutFlush,

    /// Writes a value to the Stderr
    StderrWrite(usize),

    /// Writes a debugged file to the Stdout
    StderrWriteDebugged(usize),

    /// Flushes the Stderr
    StderrFlush,

    /// Reads a line from the Stdin and stores it on the specified register
    BufferedStdinRead(usize),
}