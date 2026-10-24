use std::{
    io::{BufWriter, stdin, stdout},
    path::Path,
    process::exit,
};

use clap::Parser;
use log::{error, info};

use crate::{
    io::Io,
    lang::{compiler, interpreter::Interpreter},
    logger::ErrorLogger,
};

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

    let error_logger = ErrorLogger::new(&input, &args.input);
    let mut ast = compiler::compile(&input).unwrap_or_else(|err| {
        error_logger.report(&err.into());
        Vec::new()
    });

    if ast.is_empty() {
        exit(1)
    }

    if program_options.print_ast {
        let ast_outname = args.input.to_owned() + "__ast.json";
        let _ = std::fs::write(
            Path::new(&ast_outname),
            format!("{}", serde_json::to_string_pretty(&ast).unwrap()),
        )
        .map_err(|e| {
            error!("Failed writing AST: {}", e);
        })
        .and_then(|()| {
            info!("Wrote AST output to {}", &ast_outname);
            Ok(())
        });
    }

    Interpreter::new()
        .run(&mut io, &mut ast)
        .unwrap_or_else(|err| error_logger.report(&err));
}

#[cfg(test)]
mod tests {
    use crate::lang::compiler;
    use crate::logger::PolderError;
    use crate::{io::Io, lang::interpreter::Interpreter, logger};
    use log;
    use std::fs::DirEntry;
    use std::panic;
    use std::path::PathBuf;
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
        let file_count = test_files.len();

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

            let mut ast = compiler::compile(fs::read_to_string(&path).unwrap().as_str())
                .unwrap_or_else(|err| {
                    report.push(format!("Compile failed! Error: {:?}", err));
                    eprint!("{}", report.join("\n"));
                    Vec::new()
                });

            assert!(!ast.is_empty());

            match Interpreter::new().run(&mut io, &mut ast) {
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

        println!(
            "{:<50} {:>4} files OK",
            test_dir.into_string().unwrap(),
            file_count
        );
    }

    fn test_errors(dir: &DirEntry) {
        initialize();
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
        let file_count = test_files.len();

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

            let ast = compiler::compile(fs::read_to_string(&path).unwrap().as_str());
            if ast.is_err() {
                assert!(check_error(ast.err().unwrap().into(), &path, &mut report));
                report.clear();
                continue;
            }

            Interpreter::new()
                .run(&mut io, &mut ast.unwrap())
                .map_err(|e| assert!(check_error(e, &path, &mut report)))
                .expect_err(
                    format!(
                        "Expected failure on {}, but passed",
                        test_dir.join(file_name).to_str().unwrap()
                    )
                    .as_str(),
                );

            report.clear();
        }

        println!(
            "{:<50} {:>4} files OK",
            test_dir.into_string().unwrap(),
            file_count
        );

        fn check_error(error: PolderError, path: &PathBuf, report: &mut Vec<String>) -> bool {
            let error = serde_json::to_string(&error).unwrap();
            let expected_output = fs::read_to_string(path.with_extension("fout")).unwrap();
            let expected_output = expected_output.trim();
            report.push(format!("Output: {}", error));
            report.push(format!("Expect: {}", expected_output));
            report.push(format!("Same? : {}", error == expected_output));
            if error != expected_output {
                eprint!("{}", report.join("\n"));
                false
            } else {
                true
            }
        }
    }
}
