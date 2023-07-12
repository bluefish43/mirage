use fxhash::FxHashMap;

use crate::value::MiValue;

#[derive(Clone, PartialEq, Debug)]
pub struct CallStack {
    max_size: usize,
    frames: Vec<StackFrame>,
}

impl CallStack {
    pub fn new() -> CallStack {
        CallStack {
            max_size: 4000,
            frames: Vec::new(),
        }
    }

    pub fn push_frame(&mut self, frame: StackFrame) -> Result<(), String> {
        if self.frames.len() + 1 >= self.max_size {
            return Err(format!("Call stack size exceeded the maximum limit of {}", self.max_size));
        }

        self.frames.push(frame);
        Ok(())
    }

    pub fn pop_frame(&mut self) -> Option<StackFrame> {
        self.frames.pop()
    }

    pub fn last_frame_mut(&mut self) -> Option<&mut StackFrame> {
        self.frames.last_mut()
    }

    pub fn get_backtrace_string(&self) -> String {
        let mut backtrace = String::new();
        let mut prev_frame: Option<&StackFrame> = None;
        let mut prev_frame_count = 1;
        let mut frame_count = 0;

        for frame in self.frames.iter().rev() {
            if frame_count >= 8 {
                break;
            }

            if Some(frame) == prev_frame {
                prev_frame_count += 1;
            } else {
                if let Some(prev_frame) = prev_frame {
                    if prev_frame_count > 1 {
                        backtrace.push_str(&format!("\t<{} times called>\n", prev_frame_count));
                    }
                    backtrace.push('\n');
                }

                prev_frame = Some(frame);
                prev_frame_count = 1;

                backtrace.push_str(&format!("at {}\n", frame.name));
                backtrace.push_str("\t- Arguments:\n");
                for (arg_name, arg_value) in &frame.args {
                    backtrace.push_str(&format!("\t\t{}: {}\n", arg_name, arg_value.to_string()));
                }
                backtrace.push_str("\t- Local Variables:\n");
                for (var_name, var_value) in &frame.local_variables {
                    backtrace.push_str(&format!("\t\t{}: {}\n", var_name, var_value.to_string()));
                }
                if let Some(return_addr) = frame.return_addr {
                    backtrace.push_str(&format!("\t- Return Address: {}\n", return_addr));
                }
                if frame.handles_error {
                    backtrace.push_str(&format!("\t- Error Handling Address: {}\n", frame.error_handling_addr));
                }

                frame_count += 1;
            }
        }

        if let Some(prev_frame) = prev_frame {
            if prev_frame_count > 1 {
                backtrace.push_str(&format!(" <{} times called>", prev_frame_count));
            }
        }

        backtrace
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct StackFrame {
    pub name: String,
    pub args: FxHashMap<String, MiValue>,
    pub local_variables: FxHashMap<String, MiValue>,
    pub return_addr: Option<usize>,
    pub handles_error: bool,
    pub error_handling_addr: usize,
}

impl StackFrame {
    pub fn new(
        name: String,
        args: FxHashMap<String, MiValue>,
        return_addr: Option<usize>,
        handles_error: bool,
        error_handling_addr: usize,
    ) -> Self {
        Self {
            name,
            args,
            local_variables: FxHashMap::default(),
            return_addr,
            handles_error,
            error_handling_addr,
        }
    }
}
