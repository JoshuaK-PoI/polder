pub(crate) struct Printer {
    mode: PrintMode,
}

impl Printer {
    pub(crate) fn init(mode: PrintMode) -> Printer {
        Printer { mode }
    }

    pub(crate) fn print(&self, text: String) -> String {
        if self.mode == PrintMode::Stdout {
            println!("{}", text.as_str());
        }

        text
    }
}

#[derive(Eq, PartialEq)]
pub(crate) enum PrintMode {
    Stdout,
}
