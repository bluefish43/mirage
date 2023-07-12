pub mod stack;
pub mod value;
pub mod class;
pub mod function;
pub mod result;
pub mod args;
pub mod instructions;
pub mod runtime;
pub mod meta;
pub mod builtins;
pub mod registers;
pub mod assembly;

use std::{fs::File, io::{Write, stdout, stderr, Read}, time::SystemTime, process::ExitCode};
use instructions::Instruction;
use meta::{Metadata, Manifest};
use result::MiResult;
use runtime::MirageRuntime;
use value::IntoValue;
use ansi_term::Color;
use std::process::exit;
use std::time::Instant;
use std::env::args;

const MIRAGE_VERSION: &'static str = "1.2.1";

use crate::value::{MiValue, MiType};

#[macro_export]
macro_rules! error_println {
    ($($args:expr),*) => {
        eprintln!("{} {}", Color::Red.bold().paint("Error:"), format_args!($($args),*))
    };
}

#[macro_export]
macro_rules! note_println {
    ($($args:expr),*) => {
        eprintln!("{} {}", Color::White.bold().paint("Note:"), format_args!($($args),*))
    };
}

#[macro_export]
macro_rules! example_println {
    ($($args:expr),*) => {
        let input = format!($($args),*);
        let lines: Vec<String> = input.lines().map(|line| format!("+ {}", ansi_term::Color::Green.paint(line))).collect();
        let output = lines.join("\n");
        eprintln!("{}", output);
    };
}

#[macro_export]
macro_rules! warning_println {
    ($($args:expr),*) => {
        eprintln!("{} {}", ansi_term::Color::Yellow.bold().paint("Warning:"), format_args!($($args),*))
    };
}

fn main() -> ExitCode {
    let mut instant: Instant = Instant::now();
    let mut option = String::new();
    let mut input = String::new();
    let mut output = String::new();
    let mut asm = false;

    let mut args = args().skip(1);

    if args.len() < 1 {
        error_println!("Minimum number of arguments is 1");
        return ExitCode::FAILURE;
    }
    loop {
        let next_arg = args.next();
        match next_arg {
            Some(arg) => match arg.as_str() {
                "--asm" => {
                    asm = true;
                }
                "-i" => match args.next() {
                    Some(arg) => {
                        if input.as_str() != "" {
                            error_println!("-i can only be used once");
                            note_println!("each option can only be used once");
                            return ExitCode::FAILURE
                        }
                        input = arg;
                    }
                    None => {
                        error_println!("-i requires an argument");
                        note_println!("provide an argument like -i /path/to/file");
                        return ExitCode::FAILURE
                    }
                },
                "-o" => match args.next() {
                    Some(arg) => {
                        if output.as_str() != "" {
                            error_println!("-o can only be used once");
                            note_println!("each option can only be used once");
                            return ExitCode::FAILURE
                        }
                        output = arg;
                    }
                    None => {
                        error_println!("-o requires an argument");
                        note_println!("provide an argument like -i /path/to/file");
                        return ExitCode::FAILURE
                    }
                }
                "run" => {
                    if option != String::new() {
                        error_println!("The main option can only be used once");
                        return ExitCode::FAILURE
                    }
                    option = arg;
                }
                "build" => {
                    if option != String::new() {
                        error_println!("The main option can only be used once");
                        return ExitCode::FAILURE
                    }
                    option = arg;
                }
                _ => {
                    if &input != "" {
                        error_println!("input is already defined: assumed `{}` to be an input file as its not a recognized argument", input);
                        return ExitCode::FAILURE
                    } else {
                        input = arg;
                    }
                }
            },
            None => break,
        }
    }
    if &option == "build" {
        let mut input = input.clone();
        if output.is_empty() {
            output = format!("{}.mirage", if input.is_empty() { "out" } else { &input });
        }
        if input.is_empty() {
            input = "./manifest.json".to_string();
        }
        if output.is_empty() {
            output = format!("{input}.mirage");
        }
        let file = File::open(input);
        match file {
            Ok(mut file) => {
                let mut manifest_string = String::new();
                match file.read_to_string(&mut manifest_string) {
                    Ok(_) => {
                        let manifest = serde_json::from_str::<Manifest>(&manifest_string);
                        match manifest {
                            Ok(manifest) => {
                                match File::open(&manifest.main_file) {
                                    Ok(mut file) => {
                                        let mut main_file_string = String::new();
                                        match file.read_to_string(&mut main_file_string) {
                                            Ok(_) => {
                                                let tokens = assembly::tokens::tokenize(&main_file_string, &manifest.main_file);
                                                match tokens {
                                                    Ok(tokens) => {
                                                        let mut parser = assembly::parser::Parser::new(tokens);
                                                        match parser.parse() {
                                                            Ok(instructions) => {
                                                                let length = instructions.len();
                                                                let metadata = Metadata {
                                                                    package: manifest.package,
                                                                    version: manifest.version,
                                                                    timestamp: SystemTime::now(),
                                                                    description: manifest.description.unwrap_or(String::new()),
                                                                    author: manifest.author,
                                                                    debug: false,
                                                                    instructions,
                                                                    source_code: None,
                                                                    license: Some(manifest.license),
                                                                    total_instructions: length,
                                                                    compiled_version: MIRAGE_VERSION.to_string(),
                                                                };
                                                                match File::create(&output) {
                                                                    Ok(mut file) => {
                                                                        let converted = bincode::serialize(&metadata);
                                                                        match converted {
                                                                            Ok(converted) => {
                                                                                match file.write_all(&converted) {
                                                                                    Ok(_) => {
                                                                                        return ExitCode::SUCCESS
                                                                                    }
                                                                                    Err(err) => {
                                                                                        error_println!("Failed to write bytes to file: {err}");
                                                                                        return ExitCode::FAILURE
                                                                                    }
                                                                                }
                                                                            }
                                                                            Err(err) => {
                                                                                error_println!("Failed to serialize file metadata: {err}");
                                                                                return ExitCode::FAILURE
                                                                            }
                                                                        }
                                                                    }
                                                                    Err(err) => {
                                                                        error_println!("Failed to create output file: {err}");
                                                                        return ExitCode::FAILURE
                                                                    }
                                                                }
                                                            }
                                                            Err(err) => {
                                                                error_println!("Error parsing: {err}");
                                                                return ExitCode::FAILURE
                                                            }
                                                        }
                                                    }
                                                    Err(err) => {
                                                        error_println!("{err}");
                                                        return ExitCode::FAILURE
                                                    }
                                                }
                                            }
                                            Err(err) => {
                                                error_println!("Error reading the specified main file `{}`: {err}", &manifest.main_file);
                                                return ExitCode::FAILURE
                                            }
                                        }
                                    }
                                    Err(error) => {
                                        error_println!("Error opening the specified main file `{}`: {error}", &manifest.main_file);
                                        return ExitCode::FAILURE
                                    }
                                }
                            }
                            Err(error) => {
                                error_println!("Error parsing the manifest file: {error}");
                                return ExitCode::FAILURE
                            }
                        }
                    }
                    Err(err) => {
                        error_println!("Error reading from the file: {err}");
                        return ExitCode::FAILURE
                    }
                }
            }
            Err(err) => {
                error_println!("Failed to open input file: {}", err);
                return ExitCode::FAILURE
            }
        }
    } else if &option == "run" {
        match File::open(input) {
            Ok(mut file) => {
                let mut input_contents = Vec::new();
                match file.read_to_end(&mut input_contents) {
                    Ok(_) => {
                        let metadata = bincode::deserialize::<Metadata>(&input_contents);
                        match metadata {
                            Ok(metadata) => {
                                let mut runtime = MirageRuntime::new(metadata.instructions);
                                runtime.setup();
                                match runtime.run() {
                                    Ok(_) => {
                                        print!("\n");
                                        return ExitCode::SUCCESS;
                                    }
                                    Err(error) => {
                                        stdout().flush().unwrap();
                                        stderr().flush().unwrap();
                                        eprintln!("\n{} {}", Color::Red.bold().paint("Error:"), error.name);
                                        eprintln!("{} {}", Color::Green.bold().paint("Message:"), error.message);
                                        eprintln!("Stack Backtrace:");
                                        eprintln!("{}", error.backtrace);
                                    }
                                }
                            }
                            Err(err) => {
                                error_println!("Failed to decode the binary file metadata (invalid format)");
                            }
                        }
                        return ExitCode::SUCCESS
                    }
                    Err(err) => {
                        error_println!("Failed to read from input file: {err}");
                        return ExitCode::FAILURE
                    }
                }
            }
            Err(err) => {
                error_println!("Failed to open input file: {err}");
                return ExitCode::FAILURE
            }
        }
    } else {
        error_println!("Unknown option: {}", option);
        return ExitCode::FAILURE
    }
}
