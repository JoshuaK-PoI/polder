use std::{
    io::{BufWriter, stdin, stdout},
    process::exit,
};

use clap::Parser;
use log::debug;

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
    .unwrap_or_else(|e| {
        eprintln!("Could not start logger: {}", e);
        exit(1);
    });

    let program_options = program_options::ProgramOptions {
        print_ast: args.print_ast,
    };

    let mut input = stdin().lock();
    let mut output = BufWriter::new(stdout().lock());

    let mut io = Io::new(&mut input, &mut output);

    let input = std::fs::read_to_string(&args.input).unwrap_or_else(|err| {
        eprintln!("Could not read file {}: {} ", &args.input, err);
        exit(1);
    });

    Interpreter::new(&input, &args.input, &program_options)
        .run(&mut io)
        .unwrap_or_else(|err| {
            debug!(
                " [ Error ] : {}",
                serde_json::to_string_pretty(&err).unwrap()
            );
            ErrorLogger::new(&input, &args.input).report(&err)
        });
}

#[cfg(test)]
mod tests {
    use crate::{io::Io, lang::interpreter::Interpreter, logger, program_options::ProgramOptions};
    use log;
    use std::fs::DirEntry;
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
    fn end_to_end_tests() {
        initialize();
        let test_case_master_dir = Path::new("tests/cases");

        for dir in fs::read_dir(test_case_master_dir)
            .unwrap()
            .filter(|res| {
                res.as_ref()
                    .is_ok_and(|entry| entry.metadata().is_ok_and(|r| r.is_dir()))
            })
            .map(|res| res.unwrap())
        {
            test_happy_path(&dir);
            test_errors(&dir);
        }
    }

    fn test_happy_path(dir: &DirEntry) {
        initialize();
        let program_options = ProgramOptions { print_ast: false };
        let test_dir = dir.path().join("voorbeelden");
        let mut test_files = fs::read_dir(&test_dir)
            .unwrap()
            .map(|f| f.unwrap())
            .filter(|f| {
                if f.path().extension().is_some() {
                    f.path().extension().unwrap().eq_ignore_ascii_case("pol")
                } else {
                    false
                }
            })
            .collect::<Vec<_>>();

        if test_files.len() == 0 {
            println!("No test files found: {}", test_dir.into_string().unwrap());
            return;
        }

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

            match Interpreter::new(
                fs::read_to_string(&path).unwrap().as_str(),
                file_name,
                &program_options,
            )
            .run(&mut io)
            {
                Ok(_) => {
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
                Err(e) => {
                    report.push(format!("Test failed! Error: {:?}", e));
                    eprint!("{}", report.join("\n"));
                    assert!(false)
                }
            }
        }
    }

    fn test_errors(dir: &DirEntry) {
        initialize();
        let program_options = ProgramOptions { print_ast: false };

        let test_dir = dir.path().join("fouten");
        let mut test_files = fs::read_dir(&test_dir)
            .unwrap()
            .map(|f| f.unwrap())
            .filter(|f| {
                if f.path().extension().is_some() {
                    f.path().extension().unwrap().eq_ignore_ascii_case("pol")
                } else {
                    false
                }
            })
            .collect::<Vec<_>>();

        if test_files.len() == 0 {
            println!("No test files found: {}", test_dir.into_string().unwrap());
            return;
        }

        test_files.sort_by_key(|f| f.path());

        let mut report: Vec<String> = Vec::new();

        for file in test_files {
            let path = file.path();
            let file_name = path.file_name().unwrap().to_str().unwrap();

            report.push(format!(
                "Testing file: {}",
                test_dir.join(file_name).to_str().unwrap()
            ));
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
            .expect_err(
                format!(
                    "Expected failure on {}, but passed",
                    test_dir.join(file_name).to_str().unwrap()
                )
                .as_str(),
            );

            report.clear();
        }
    }
}
