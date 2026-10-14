use std::io::{BufRead, Write};

pub(crate) struct Io<'a> {
    pub(crate) input: &'a mut dyn BufRead,
    pub(crate) output: &'a mut dyn Write,
}

impl<'a> Io<'a> {
    pub(crate) fn new(input: &'a mut dyn BufRead, output: &'a mut dyn Write) -> Self {
        Io { input, output }
    }

    pub(crate) fn write(&mut self, text: &str) -> Result<(), std::io::Error> {
        self.output.write_all(text.as_bytes())
    }

    pub(crate) fn read_line(&mut self) -> Result<Option<String>, std::io::Error> {
        self.flush()?;

        let mut line = String::new();
        let bytes = self.input.read_line(&mut line)?;

        if bytes == 0 as usize {
            Ok(None)
        } else {
            if line.ends_with('\n') {
                line.pop();
                if line.ends_with('\r') {
                    line.pop();
                }
            }

            Ok(Some(line))
        }
    }

    pub(crate) fn flush(&mut self) -> Result<(), std::io::Error> {
        self.output.flush()
    }
}
