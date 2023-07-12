use crate::instructions::Instruction;
use crate::value::IntoValue;
use crate::value::MiType;
use crate::value::MiValue;
use super::tokens::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    pc: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            pc: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Instruction>, String> {
        let mut instructions = vec![];
        while let Some(ctoken) = self.tokens.get(self.pc) {
            self.pc += 1;
            match &ctoken.token_type {
                TokenType::Keyword(kw) => match kw.as_str() {
                    "move" => {
                        let addr1 = self.parse_reg()?;
                        
                        let val = self.parse_value()?;
                        instructions.push(Instruction::Move(addr1, val))
                    }
                    "movebetween" => {
                        let addr1 = self.parse_reg()?;
                        
                        let addr2 = self.parse_reg()?;
                        instructions.push(Instruction::MoveBetween(addr1, addr2))
                    }
                    "moveargument" => {
                        let arg = self.parse_string()?;
                        
                        let addr = self.parse_reg()?;
                        instructions.push(Instruction::MoveArgument(arg, addr))
                    }
                    "moveasargument" => {
                        let reg = self.parse_reg()?;
                        instructions.push(Instruction::MoveAsArgument(reg))
                    }
                    "add" => {
                        let op1 = self.parse_reg()?;
                        
                        let op2 = self.parse_reg()?;
                        
                        let dst = self.parse_reg()?;
                        instructions.push(Instruction::Add(op1, op2, dst))
                    }
                    "sub" => {
                        let op1 = self.parse_reg()?;
                        
                        let op2 = self.parse_reg()?;
                        
                        let dst = self.parse_reg()?;
                        instructions.push(Instruction::Sub(op1, op2, dst))
                    }
                    "mul" => {
                        let op1 = self.parse_reg()?;
                        
                        let op2 = self.parse_reg()?;
                        
                        let dst = self.parse_reg()?;
                        instructions.push(Instruction::Mul(op1, op2, dst))
                    }
                    "div" => {
                        let op1 = self.parse_reg()?;
                        
                        let op2 = self.parse_reg()?;
                        
                        let dst = self.parse_reg()?;
                        instructions.push(Instruction::Div(op1, op2, dst))
                    }
                    "rem" => {
                        let op1 = self.parse_reg()?;
                        
                        let op2 = self.parse_reg()?;
                        
                        let dst = self.parse_reg()?;
                        instructions.push(Instruction::Rem(op1, op2, dst))
                    }
                    "pow" => {
                        let op1 = self.parse_reg()?;
                        
                        let op2 = self.parse_reg()?;
                        
                        let dst = self.parse_reg()?;
                        instructions.push(Instruction::Pow(op1, op2, dst))
                    }
                    "or" => {
                        let op1 = self.parse_reg()?;
                        
                        let op2 = self.parse_reg()?;
                        
                        let dst = self.parse_reg()?;
                        instructions.push(Instruction::Or(op1, op2, dst))
                    }
                    "xor" => {
                        let op1 = self.parse_reg()?;
                        
                        let op2 = self.parse_reg()?;
                        
                        let dst = self.parse_reg()?;
                        instructions.push(Instruction::Xor(op1, op2, dst))
                    }
                    "and" => {
                        let op1 = self.parse_reg()?;
                        
                        let op2 = self.parse_reg()?;
                        
                        let dst = self.parse_reg()?;
                        instructions.push(Instruction::And(op1, op2, dst))
                    }
                    "not" => {
                        let op1 = self.parse_reg()?;
                        
                        let dst = self.parse_reg()?;
                        instructions.push(Instruction::Not(op1, dst))
                    }
                    "lt" => {
                        let op1 = self.parse_reg()?;
                        
                        let op2 = self.parse_reg()?;
                        
                        let dst = self.parse_reg()?;
                        instructions.push(Instruction::Lt(op1, op2, dst))
                    }
                    "le" => {
                        let op1 = self.parse_reg()?;
                        
                        let op2 = self.parse_reg()?;
                        
                        let dst = self.parse_reg()?;
                        instructions.push(Instruction::Le(op1, op2, dst))
                    }
                    "gt" => {
                        let op1 = self.parse_reg()?;
                        
                        let op2 = self.parse_reg()?;
                        
                        let dst = self.parse_reg()?;
                        instructions.push(Instruction::Gt(op1, op2, dst))
                    }
                    "ge" => {
                        let op1 = self.parse_reg()?;
                        
                        let op2 = self.parse_reg()?;
                        
                        let dst = self.parse_reg()?;
                        instructions.push(Instruction::Ge(op1, op2, dst))
                    }
                    "return" => {
                        instructions.push(Instruction::Return)
                    }
                    "setvariable" => {
                        let reg = self.parse_reg()?;
                        
                        let var = self.parse_identifier()?;
                        instructions.push(Instruction::SetVariable(reg, var));
                    }
                    "movfromvariable" => {
                        let ident = self.parse_identifier()?;
                        
                        let reg = self.parse_reg()?;
                        instructions.push(Instruction::MovFromVariable(ident, reg))
                    }
                    "throwfrom" => {
                        let addr1 = self.parse_reg()?;
                        
                        let addr2 = self.parse_reg()?;
                        instructions.push(Instruction::ThrowFrom(addr1, addr2))
                    }
                    "eq" => {
                        let op1 = self.parse_reg()?;
                        
                        let op2 = self.parse_reg()?;
                        
                        let dst = self.parse_reg()?;
                        instructions.push(Instruction::Eq(op1, op2, dst))
                    }
                    "ne" => {
                        let op1 = self.parse_reg()?;
                        
                        let op2 = self.parse_reg()?;
                        
                        let dst = self.parse_reg()?;
                        instructions.push(Instruction::Ne(op1, op2, dst))
                    }
                    "definelabel" => {
                        let label = self.parse_identifier()?;
                        instructions.push(Instruction::DefineLabel(label))
                    }
                    "jumpunc" => {
                        let label = self.parse_identifier()?;
                        instructions.push(Instruction::JumpUnconditional(label))
                    }
                    "jumpc" => {
                        let reg = self.parse_reg()?;
                        
                        let label = self.parse_identifier()?;
                        instructions.push(Instruction::JumpConditional(reg, label))
                    }
                    "call" => {
                        let name = self.parse_identifier()?;
                        instructions.push(Instruction::Call(name))
                    }
                    "definefnlabel" => {
                        let mut args: Vec<String> = vec![];
                        let name = self.parse_identifier()?;

                        let len = self.parse_int()? as usize;
                        if len != 0 {
                            for _ in 0..len - 1 {
                                args.push(self.parse_identifier()?);
                            }
                        }
                        let returns = self.parse_type()?;
                        instructions.push(Instruction::DefineFnLabel(name, args, returns))
                    }
                    "endfunction" => {
                        instructions.push(Instruction::EndFunction)
                    }
                    "stdoutwrite" => {
                        let reg = self.parse_reg()?;
                        instructions.push(Instruction::StdoutWrite(reg))
                    }
                    "stdoutwritedebugged" => {
                        let reg = self.parse_reg()?;
                        instructions.push(Instruction::StdoutWriteDebugged(reg))
                    }
                    "stdoutflush" => {
                        instructions.push(Instruction::StdoutFlush)
                    }
                    "stderrwrite" => {
                        let reg = self.parse_reg()?;
                        instructions.push(Instruction::StderrWrite(reg))
                    }
                    "stderrwritedebugged" => {
                        let reg = self.parse_reg()?;
                        instructions.push(Instruction::StderrWriteDebugged(reg))
                    }
                    "stderrflush" => {
                        instructions.push(Instruction::StderrFlush)
                    }
                    "bufferedstdinread" => {
                        let reg = self.parse_reg()?;
                        instructions.push(Instruction::BufferedStdinRead(reg))
                    }
                    _ => return Err(format!("{}:{}->{}: Invalid keyword '{}'", ctoken.line, ctoken.column, ctoken.length + ctoken.column, kw)),
                },
                _ => {
                    return Err(format!(
                        "{}:{}->{}: Invalid position for token {:?}",
                        ctoken.line, ctoken.column, ctoken.length + ctoken.column, ctoken.token_type
                    ))
                }
            }
        }
        Ok(instructions)
    }

    pub fn expect_kind(&mut self, kind: TokenType) -> Result<(), String> {
        self.pc += 1;
        match self.tokens.get(self.pc) {
            Some(token) => {
                if &token.token_type != &kind {
                    return Err(format!("Expected token `{:?}`, found `{:?}`", kind, &token.token_type))
                } else {
                    Ok(())
                }
            }
            None => {
                return Err(format!("Expected token `{:?}`, found `EOF`", kind))
            }
        }
    }

    fn parse_identifier(&mut self) -> Result<String, String> {
        if let Some(ctoken) = self.tokens.get(self.pc) {
            self.pc += 1;
            match &ctoken.token_type {
                TokenType::Identifier(name) => {
                    Ok(name.clone())
                },
                _ => Err(format!(
                    "{}:{}->{}: Expected an identifier, found {:?}",
                    ctoken.line, ctoken.column, ctoken.length + ctoken.column, ctoken.token_type, 
                )),
            }
        } else {
            let tok = self.tokens.get(self.pc - 1).unwrap();
            Err(format!("{}:{}->{}: Unexpected end of tokens", tok.line, tok.column, tok.length + tok.column))
        }
    }

    fn parse_value(&mut self) -> Result<MiValue, String> {
        if let Some(ctoken) = self.tokens.get(self.pc) {
            self.pc += 1;
            match &ctoken.token_type {
                TokenType::Type(kw) => match kw.as_str() {
                    "None" => Ok(MiValue::new(vec![], MiType::None)),
                    "int" => {
                        let value = self.parse_int()?;
                        Ok(value.into_value())
                    },
                    "float" => {
                        let value = self.parse_float()?;
                        Ok(value.into_value())
                    },
                    "string" => {
                        let value = self.parse_string()?;
                        Ok(value.into_value())
                    },
                    "bool" => {
                        let value = self.parse_bool()?;
                        Ok(value.into_value())
                    },
                    _ => Err(format!("{}:{}: Invalid value keyword '{}'", ctoken.line, ctoken.column, kw)),
                },
                _ => Err(format!(
                    "{}:{}->{}: Expected a value keyword, found {:?}",
                    ctoken.line, ctoken.column, ctoken.length + ctoken.column, ctoken.token_type
                )),
            }
        } else {
            let tok = self.tokens.get(self.pc - 1).unwrap();
            Err(format!("{}:{}->{}: Unexpected end of tokens", tok.line, tok.column, tok.column + tok.length))
        }
    }

    fn parse_int(&mut self) -> Result<i32, String> {
        if let Some(ctoken) = self.tokens.get(self.pc) {
            self.pc += 1;
            match &ctoken.token_type {
                TokenType::Int(i) => {
                    Ok(*i)
                }
                _ => {
                    return Err(format!("{}:{}->{}: Unexpected token: {:?} expected token int", ctoken.line, ctoken.column, ctoken.column + ctoken.length, ctoken.token_type))
                }
            }
        } else {
            let tok = self.tokens.get(self.pc - 1).unwrap();
            return Err(format!("{}:{}->{}: Unexpected end of tokens", tok.line, tok.column, tok.column + tok.length))
        }
    }

    fn parse_reg(&mut self) -> Result<usize, String> {
        if let Some(ctoken) = self.tokens.get(self.pc) {
            self.pc += 1;
            match &ctoken.token_type {
                TokenType::Register(i) => {
                    Ok(*i)
                }
                _ => {
                    return Err(format!("{}:{}->{}: Unexpected token: {:?} expected token register", ctoken.line, ctoken.column, ctoken.column + ctoken.length, ctoken.token_type))
                }
            }
        } else {
            let tok = self.tokens.get(self.pc - 1).unwrap();
            return Err(format!("{}:{}->{}: Unexpected end of tokens", tok.line, tok.column, tok.column + tok.length))
        }
    }

    fn parse_float(&mut self) -> Result<f64, String> {
        if let Some(ctoken) = self.tokens.get(self.pc) {
            self.pc += 1;
            match &ctoken.token_type {
                TokenType::Float(i) => {
                    Ok(*i)
                }
                _ => {
                    return Err(format!("{}:{}->{}: Unexpected token: {:?} expected token float", ctoken.line, ctoken.column, ctoken.column + ctoken.length, ctoken.token_type))
                }
            }
        } else {
            let tok = self.tokens.get(self.pc - 1).unwrap();
            return Err(format!("{}:{}->{}: Unexpected end of tokens", tok.line, tok.column, tok.column + tok.length))
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        if let Some(ctoken) = self.tokens.get(self.pc) {
            self.pc += 1;
            match &ctoken.token_type {
                TokenType::String(s) => {
                    Ok(s.clone())
                }
                _ => {
                    return Err(format!("{}:{}->{}: Unexpected token: {:?} expected token string", ctoken.line, ctoken.column, ctoken.column + ctoken.length, ctoken.token_type))
                }
            }
        } else {
            let tok = self.tokens.get(self.pc - 1).unwrap();
            return Err(format!("{}:{}->{}: Unexpected end of tokens", tok.line, tok.column, tok.column + tok.length))
        }
    }

    fn parse_bool(&mut self) -> Result<bool, String> {
        if let Some(ctoken) = self.tokens.get(self.pc) {
            self.pc += 1;
            match &ctoken.token_type {
                TokenType::Boolean(b) => {
                    Ok(*b)
                }
                _ => {
                    return Err(format!("{}:{}->{}: Unexpected token: {:?} expected token bool", ctoken.line, ctoken.column, ctoken.column + ctoken.length, ctoken.token_type))
                }
            }
        } else {
            let tok = self.tokens.get(self.pc - 1).unwrap();
            return Err(format!("{}:{}->{}: Unexpected end of tokens", tok.line, tok.column, tok.column + tok.length))
        }
    }

    
    fn parse_type(&mut self) -> Result<MiType, String> {
        if let Some(ctoken) = self.tokens.get(self.pc) {
            self.pc += 1;
            match &ctoken.token_type {
                TokenType::Type(ttype) => {
                    match ttype.as_str() {
                        "None" => {
                            Ok(MiType::None)
                        }
                        "int" => {
                            Ok(MiType::Int)
                        }
                        "float" => {
                            Ok(MiType::Float)
                        }
                        "string" => {
                            Ok(MiType::String)
                        }
                        "boolean" => {
                            Ok(MiType::Bool)
                        }
                        _ => {
                            return Err(format!("{}:{}->{}: Unrecognized type '{}'", ctoken.line, ctoken.column, ctoken.column + ctoken.length, ttype));
                        }
                    }
                }
                _ => {
                    return Err(format!("{}:{}->{}: Expected Type token, found: '{:?}'", ctoken.line, ctoken.column, ctoken.column + ctoken.length, &ctoken.token_type));
                }
            }
        } else {
            let tok = self.tokens.get(self.pc - 1).unwrap();
            return Err(format!("{}:{}->{}: Unexpected end of tokens", tok.line, tok.column, tok.column + tok.length))
        }
    }
}
