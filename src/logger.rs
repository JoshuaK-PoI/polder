use crate::lang::compiler::CompilerError;

pub(crate) struct ErrorLogger<'a> {
    filename: &'a str,
    input: &'a str,
}

impl<'a> ErrorLogger<'a> {
    pub(crate) fn new(input: &'a str, filename: &'a str) -> ErrorLogger<'a> {
        ErrorLogger { input, filename }
    }

    pub(crate) fn report<'b>(&'a self, error_detail: &'b CompilerError) -> () {
        use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};
        let mut colors = ColorGenerator::new();
        let color_1 = colors.next();

        let mut report = Report::build(
            ReportKind::Error,
            (self.filename, error_detail.span.clone()),
        )
        .with_code(&error_detail.error_code)
        .with_message(&error_detail.error_message)
        .with_label(
            Label::new((self.filename, error_detail.span.clone()))
                .with_message(&error_detail.span_message)
                .with_color(color_1),
        );

        if let Some(help) = &error_detail.help {
            report = report.with_help(help);
        }

        if let Some(info) = &error_detail.info {
            report = report.with_note(info);
        }

        report
            .finish()
            .eprint((self.filename, Source::from(self.input)))
            .unwrap();
    }

    pub(crate) fn report_many(&self, errors: &[CompilerError]) -> () {
        errors.iter().for_each(|error| self.report(error));
    }
}

use log::{LevelFilter, Metadata, Record};

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
