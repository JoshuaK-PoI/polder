use std::io::{BufWriter, stdin, stdout};

use clap::Parser;

use crate::{io::Io, lang::interpreter::Interpreter, logger::ErrorLogger};

mod args;
mod io;
mod lang;
mod logger;
mod program_options;

fn main() {
    let args = args::Args::parse();

    logger::init(if args.debug {
        log::LevelFilter::Debug
    } else if args.trace {
        log::LevelFilter::Trace
    } else {
        log::LevelFilter::Info
    })
    .expect("Could not initialize logger");

    let program_options = program_options::ProgramOptions {
        print_ast: args.print_ast,
    };

    let mut input = stdin().lock();
    let mut output = BufWriter::new(stdout().lock());

    let mut io = Io::new(&mut input, &mut output);

    let input = std::fs::read_to_string(&args.input).unwrap_or_else(|err| {
        panic!("Error reading file: {} ", err);
    });

    Interpreter::new(&input, &args.input, &program_options)
        .run(&mut io)
        .unwrap_or_else(|err| ErrorLogger::new(&input, &args.input).report(&err));
}
#[cfg(test)]
mod tests {
    use crate::{io::Io, lang::interpreter::Interpreter, logger, program_options::ProgramOptions};
    use log;
    use std::sync::Once;
    use std::{fs, path::Path};

    static INIT: Once = Once::new();

    fn initialize() {
        INIT.call_once(|| {
            logger::init(log::LevelFilter::Off)
                .unwrap_or_else(|e| panic!("Logger initialization failed: {}", e));
        })
    }

    #[test]
    fn test_happy_path() {
        initialize();
        let program_options = ProgramOptions { print_ast: false };
        let test_dir = Path::new("tests/cases/variabelen/voorbeelden");
        let mut test_files = fs::read_dir(test_dir)
            .unwrap()
            .map(|f| f.unwrap())
            .filter(|f| f.path().extension().unwrap().eq_ignore_ascii_case("pol"))
            .collect::<Vec<_>>();

        test_files.sort_by_key(|f| f.path());

        assert!(&test_files.len() > &0, "No test files found");
        let mut report: Vec<String> = Vec::new();

        for file in test_files {
            let path = file.path();
            let file_name = path.file_name().unwrap().to_str().unwrap();

            report.push(format!("Testing file: {}", file_name));
            report.push(format!("{:-<1$}", "", 40));

            let input_str =
                fs::read_to_string(path.with_extension("in")).unwrap_or_else(|_| String::new());
            let mut input: &[u8] = input_str.as_bytes();

            let mut output: Vec<u8> = Vec::new();
            let mut io = Io::new(&mut input, &mut output);

            Interpreter::new(
                fs::read_to_string(&path).unwrap().as_str(),
                file_name,
                &program_options,
            )
            .run(&mut io)
            .unwrap();
            let result = String::from_utf8(output).unwrap();
            let expected_output = fs::read_to_string(path.with_extension("uit")).unwrap();

            report.push(format!("Output: {:?}", result));
            report.push(format!("Expect: {:?}", expected_output));

            if result != expected_output {
                eprint!("{}", report.join("\n"));
                assert!(false);
            }

            report.clear();
        }
    }

    #[test]
    fn test_errors() {
        initialize();
        let program_options = ProgramOptions { print_ast: false };

        let test_dir = Path::new("tests/cases/variabelen/fouten");
        let mut test_files = fs::read_dir(test_dir)
            .unwrap()
            .map(|f| f.unwrap())
            .filter(|f| f.path().extension().unwrap().eq_ignore_ascii_case("pol"))
            .collect::<Vec<_>>();

        test_files.sort_by_key(|f| f.path());

        assert!(&test_files.len() > &0, "No test files found");
        let mut report: Vec<String> = Vec::new();

        for file in test_files {
            let path = file.path();
            let file_name = path.file_name().unwrap().to_str().unwrap();

            report.push(format!("Testing file: {}", file_name));
            report.push(format!("{:-<1$}", "", 40));
            let input_str =
                fs::read_to_string(path.with_extension("in")).unwrap_or_else(|_| String::new());
            let mut input: &[u8] = input_str.as_bytes();

            let mut output: Vec<u8> = Vec::new();
            let mut io = Io::new(&mut input, &mut output);

            Interpreter::new(
                fs::read_to_string(&path).unwrap().as_str(),
                file_name,
                &program_options,
            )
            .run(&mut io)
            .map_err(|e| {
                let error = serde_json::to_string(&e).unwrap();
                let expected_output = fs::read_to_string(path.with_extension("fout")).unwrap();
                let expected_output = expected_output.trim();
                report.push(format!("Output: {}", error));
                report.push(format!("Expect: {}", expected_output));
                report.push(format!("Same? : {}", error == expected_output));
                if error != expected_output {
                    eprint!("{}", report.join("\n"));
                    assert!(false);
                }
            })
            .expect_err(format!("Expected failure on {}, but passed", file_name).as_str());

            report.clear();
        }
    }
}
