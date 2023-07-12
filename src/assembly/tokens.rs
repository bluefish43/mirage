#[derive(Clone, Debug, PartialEq)]
pub enum TokenType {
    Register(usize),
    Keyword(String),
    Identifier(String),
    Type(String),
    Int(i32),
    Float(f64),
    String(String),
    Boolean(bool),
    Comma,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub length: usize,
    pub line: usize,
    pub column: usize,
}

pub fn tokenize(input: &str, filename: &str) -> Result<Vec<Token>, String> {
    let mut iterator = input.chars().peekable();
    let mut tokens_stream: Vec<Token> = Vec::new();
    let mut line = 1;
    let mut column = 0;
    loop {
        match iterator.next() {
            Some(character) => {
                match character {
                    'a'..='z' | 'A'..='Z' | '_' => {
                        let mut identifier = String::new();
                        identifier.push(character);
                        while let Some(c) = iterator.peek() {
                            if c.is_whitespace() {
                                break;
                            } else if c.is_alphanumeric() || c == &'_' || c == &'.' {
                                identifier.push(iterator.next().unwrap());
                            } else {
                                break;
                            }
                        }
                        let identifier_len = identifier.len();
                        column += identifier_len;
                        if [
                            "move", "movebetween", "moveargument", "moveasargument",
                            "add", "sub", "mul", "div", "rem", "pow", "or", "xor", "and",
                            "not", "lt", "le", "gt", "ge", "return", "setvariable", "movfromvariable",
                            "throwfrom", "eq", "ne", "definelabel", "jumpunc", "jumpc",
                            "call", "definefnlabel", "endfunction", "stdoutwrite", "stdoutwritedebugged",
                            "stdoutflush", "stderrwrite", "stderrwritedebugged", "stderrflush", "bufferedstdinread",
                        ].contains(&identifier.as_str()) {
                            tokens_stream.push(Token {
                                token_type: TokenType::Keyword(identifier),
                                length: identifier_len,
                                line,
                                column,
                            })
                        } else if ["true", "false"].contains(&identifier.as_str()) {
                            tokens_stream.push(Token {
                                token_type: TokenType::Boolean(&identifier == "true"),
                                length: identifier_len,
                                line,
                                column,
                            })
                        } else if [
                            "int", "float", "string", "class", "function", "None"
                        ].contains(&identifier.as_str()) {
                            tokens_stream.push(Token {
                                token_type: TokenType::Type(identifier),
                                length: identifier_len,
                                line,
                                column,
                            })
                        } else if identifier.as_str() == "plch" {
                            continue;
                        } else if identifier.starts_with("r") && identifier.chars().last().unwrap().is_numeric()
                                                              && identifier.len() <= 3 {
                            let num: String = identifier.chars().skip(1).collect();
                            let res = num.parse::<u8>();
                            match res {
                                Ok(reg) => {
                                    if reg >= 16 {
                                        return Err(format!("Invalid register index {}", reg))
                                    }
                                    tokens_stream.push(Token {
                                        token_type: TokenType::Register(reg as usize),
                                        length: identifier_len,
                                        line,
                                        column
                                    });
                                    column += num.len() + 1;
                                }
                                Err(reg) => {
                                    return Err(format!("Unable to parse register value {}", reg))
                                }
                            }
                        } else {
                            tokens_stream.push(Token {
                                token_type: TokenType::Identifier(identifier),
                                length: identifier_len,
                                line,
                                column,
                            });
                            column += identifier_len;
                        }
                    }
                    ',' => {
                        tokens_stream.push(Token {
                            token_type: TokenType::Comma,
                            length: 1,
                            line,
                            column,
                        });
                        column += 1;
                    }
                    // Inside the 'match character' block:
                    '0'..='9' => {
                        let mut number = String::new();
                        number.push(character);
                        while let Some(c) = iterator.next() {
                            if c.is_numeric() {
                                number.push(c);
                            } else {
                                break;
                            }
                        }
                        if number.contains('.') {
                            let parsed_number = number.parse::<f64>();
                            if let Ok(num) = parsed_number {
                                let identifier_len = number.len();
                                tokens_stream.push(Token {
                                    token_type: TokenType::Float(num),
                                    length: identifier_len,
                                    line,
                                    column,
                                });
                                column += identifier_len;
                            } else if let Err(err) = parsed_number {
                                return Err(format!("{}:{}:{}: Error parsing float number: {}", filename, line, column, err));
                            }
                        } else {
                            let parsed_number = number.parse::<i32>();
                            if let Ok(num) = parsed_number {
                                let identifier_len = number.len();
                                tokens_stream.push(Token {
                                    token_type: TokenType::Int(num),
                                    length: identifier_len,
                                    line,
                                    column,
                                });
                                column += identifier_len; 
                            } else if let Err(err) = parsed_number {
                                return Err(format!("{}:{}:{}: Error parsing int number: {}", filename, line, column, err));
                            }
                        }
                    }

                    '\"' => {
                        let mut reached = false;
                        let mut string = String::new();
                        while let Some(ch) = iterator.next() {
                            match ch {
                                '\"' => {
                                    reached = true;
                                    break;
                                }
                                '\n' => {
                                    line += 1;
                                    string.push('\n');
                                }
                                '\r' => {
                                    if Some(&'\n') == iterator.peek() {
                                        line += 1;
                                        string.push_str("\r\n");
                                        iterator.next();
                                    } else {
                                        string.push('\r');
                                    }
                                }
                                '\\' => {
                                    match iterator.next() {
                                        Some(c) => {
                                            match c {
                                                'n' => {
                                                    string.push('\n');
                                                }
                                                'r' => {
                                                    string.push('\r');
                                                }
                                                't' => {
                                                    string.push('\t');
                                                }
                                                '\\' => {
                                                    string.push('\\');
                                                }
                                                '0' => {
                                                    string.push('\0');
                                                }
                                                'u' => {
                                                    let mut digits = String::new();
                                                    for _ in 0..4 {
                                                        match iterator.next() {
                                                            Some(digit) => {
                                                                digits.push(digit);
                                                            }
                                                            None => {
                                                                return Err(format!(
                                                                    "{}:{}: A unicode escape sequence must have 4 hexadecimal digits in the sense of \\u{{7FFF}}",
                                                                    line, column
                                                                ))
                                                            }
                                                        }
                                                    }
                                                    if let Ok(num) = u32::from_str_radix(&digits, 16) {
                                                        if let Some(ch) = char::from_u32(num) {
                                                            string.push(ch);
                                                        }
                                                    } else if let Err(err) =
                                                        u32::from_str_radix(&digits, 16)
                                                    {
                                                        return Err(format!(
                                                            "{}:{}: Error during unicode escape sequence '\\u{}' parsing: {}",
                                                            line, column, digits, err
                                                        ));
                                                    }
                                                }
                                                '"' => {
                                                    string.push('"');
                                                }
                                                _ => {
                                                    return Err(format!(
                                                        "{}:{}: Unknown escape sequence '\\{}'",
                                                        line, column, c
                                                    ))
                                                }
                                            }
                                        }
                                        None => {
                                            return Err(format!(
                                                "{}:{}: Unclosed string literal",
                                                line, column
                                            ));
                                        }
                                    }
                                }
                                _ => {
                                    string.push(ch);
                                }
                            }
                        }
                        if !reached {
                            return Err(format!(
                                "{}:{}: Unclosed string literal",
                                line, column
                            ));
                        }
                        let strlen = string.len();
                        tokens_stream.push(Token {
                            token_type: TokenType::String(string),
                            length: strlen + 2,
                            line,
                            column,
                        });
                    }
                    '-' => {
                        if Some(&'-') == iterator.peek() {
                            iterator.next();
                            while let Some(c) = iterator.next() {
                                if c == '\n' {
                                    line += 1;
                                    column = 0;
                                    break;
                                } else {
                                    continue;
                                }
                            }
                        } else {
                            return Err(format!("{}:{}:{}: Unrecognized token '-{}'", filename, line, column, iterator.peek().unwrap_or(&'?')))
                        }
                    }
                    '\n' => {
                        column = 0;
                        line += 1;
                    }
                    _ => {
                        if character.is_whitespace() {
                            continue;
                        } else {
                            return Err(format!("{}:{}:{}: Unrecognized token '{}'", filename, line, column, character))
                        }
                    }
                }
            }
            None => break,
        }
    }
    Ok(tokens_stream)
}