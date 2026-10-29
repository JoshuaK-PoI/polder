use ariadne::{Color, ColorGenerator, Label, Report, ReportKind, Source};
use log::{LevelFilter, Metadata, Record};
use serde::Serialize;

use crate::lang::compiler::Span;

#[derive(Debug, Serialize)]
pub(crate) struct PolderError {
    pub(crate) message: String,
    pub(crate) span_message: String,
    pub(crate) span: Span,
}

pub(crate) struct ErrorLogger<'a> {
    filename: &'a str,
    input: &'a str,
}

impl<'a> ErrorLogger<'a> {
    pub(crate) fn new(input: &'a str, filename: &'a str) -> ErrorLogger<'a> {
        ErrorLogger { input, filename }
    }

    pub(crate) fn report<'b>(&'a self, error_detail: &'b PolderError) -> () {
        let mut colors = ColorGenerator::new();
        let color_1 = colors.next();

        let report = Report::build(
            ReportKind::Custom("Doe normaal!", Color::Red),
            (self.filename, error_detail.span.0..error_detail.span.1),
        )
        .with_message(&error_detail.message)
        .with_label(
            Label::new((self.filename, error_detail.span.0..error_detail.span.1))
                .with_message(&error_detail.span_message)
                .with_color(color_1),
        );

        report
            .finish()
            .eprint((self.filename, Source::from(self.input)))
            .unwrap();
    }
}

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;
pub fn init(level: LevelFilter) -> Result<(), log::SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(level))
}
