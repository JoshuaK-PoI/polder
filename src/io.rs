use std::io::{BufRead, Write};

use crate::lang::compiler::CompilerError;

pub(crate) struct Io<'a> {
    pub(crate) input: &'a mut dyn BufRead,
    pub(crate) output: &'a mut dyn Write,
}

impl<'a> Io<'a> {
    pub(crate) fn new(input: &'a mut dyn BufRead, output: &'a mut dyn Write) -> Self {
        Io { input, output }
    }

    pub(crate) fn write(&mut self, text: &str) -> Result<(), String> {
        self.output
            .write_all(text.as_bytes())
            .map_err(|e| format!("Failed writing to output buffer: {}", e.to_string()))
    }

    pub(crate) fn write_line(&mut self, text: &str) -> Result<(), String> {
        writeln!(self.output, "{}", text)
            .map_err(|e| format!("Failed writing to output buffer: {}", e.to_string()))
    }

    pub(crate) fn read_line(&mut self) -> Result<String, String> {
        self.flush()?;

        let mut line = String::new();
        let bytes = self
            .input
            .read_line(&mut line)
            .map_err(|e| format!("Failed to read input line: {}", e.to_string()))?;

        if bytes == 0 as usize {
            return Err(format!("No input given"));
        }

        if line.ends_with('\n') {
            line.pop();
            if line.ends_with('\r') {
                line.pop();
            }
        }

        Ok(line)
    }

    pub(crate) fn flush(&mut self) -> Result<(), String> {
        self.output
            .flush()
            .map_err(|e| format!("Failed to flush output buffer: {}", e.to_string()))
    }
}
