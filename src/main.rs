use std::{
    io::{BufWriter, Write, stdin, stdout},
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

#[hotpath::main]
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

    if args.test_interactive {
        exit(interactive_test::run(&args.input, &mut io));
    }

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

    let result = Interpreter::new().run(&mut io, &mut ast);
    if result.is_err() {
        exit(1);
    }

    let out = output.flush();
    if out.is_err() {
        eprintln!("Failed to flush output: {}", out.unwrap_err());
        exit(1);
    }

    exit(0);
}

mod interactive_test {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use serde::Serialize;

    use crate::{
        io::Io,
        lang::{compiler, interpreter::Interpreter},
        logger::PolderError,
    };

    pub(crate) fn run(input_dir: &str, stdio: &mut Io) -> i32 {
        let test_dir = Path::new(input_dir);
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
            eprintln!("[ERROR] No test files found: {}", test_dir.display());
            return 1;
        }

        test_files.sort_by_key(|f| f.path());

        let mut report: Vec<String> = Vec::new();
        let file_count = test_files.len();
        let start_time = std::time::Instant::now();
        for file in test_files {
            let path = file.path();
            let file_name = path.file_name().unwrap().to_str().unwrap();

            let input_str =
                fs::read_to_string(path.with_extension("in")).unwrap_or_else(|_| String::new());
            let mut input: &[u8] = input_str.as_bytes();

            let mut output: Vec<u8> = Vec::new();
            let mut io = Io::new(&mut input, &mut output);

            let source = read_source_file(&path);
            if source.is_none() {
                continue;
            }

            report.push(format!(
                "Testing file: {}",
                test_dir.join(file_name).to_str().unwrap()
            ));
            report.push(format!("{:-<1$}", "", 40));
            let ast = compiler::compile(source.unwrap().as_str());
            if ast.is_err() {
                if let Some(code) = check_error(
                    Into::<PolderError>::into(ast.err().unwrap()),
                    &path,
                    &mut report,
                    stdio,
                ) {
                    return code;
                }
                report.clear();
                continue;
            }

            match Interpreter::new().run(&mut io, &mut ast.unwrap()) {
                Ok(_) => {
                    let result = String::from_utf8(output).unwrap();
                    let expected_output = match fs::read_to_string(path.with_extension("uit")) {
                        Ok(file) => file,
                        Err(_) => {
                            eprintln!(
                                "Could not find or open {}, perhaps the input file expected to fail and it succeeded.",
                                path.with_extension("uit").display()
                            );
                            return 1;
                        }
                    };

                    report.push(format!("Output: {:?}", result));
                    report.push(format!("Expect: {:?}", expected_output));

                    if result != expected_output
                        && let Some(code) =
                            ask_to_update_result("uit", &mut report, result, &path, stdio)
                    {
                        return code;
                    }

                    report.clear();
                }
                Err(err) => {
                    let err = err
                        .into_iter()
                        .map(|e| e.message)
                        .collect::<Vec<String>>()
                        .join("\n");
                    if let Some(code) = check_error(err, &path, &mut report, stdio) {
                        return code;
                    }
                }
            }

            report.clear();
        }
        let end_time = std::time::Instant::now();
        println!(
            "{:<50} {:>4} files OK ({:>4.3} ms)",
            test_dir.display(),
            file_count,
            (end_time - start_time).as_micros() as f64 / 1000.0
        );

        0
    }

    fn ask_to_update_result(
        ext: &'static str,
        report: &mut Vec<String>,
        result: String,
        path: &PathBuf,
        stdio: &mut Io,
    ) -> Option<i32> {
        println!("{}", report.join("\n"));
        println!(
            "Output was different from expected result; do you want to update the expected result? (y/n)"
        );
        let mut ans = String::new();

        stdio
            .input
            .read_line(&mut ans)
            .expect("Failed to read line");
        match ans {
            c if c.to_ascii_lowercase().trim() == "y" => {
                fs::write(path.with_extension(ext), result).unwrap();
                println!(
                    "Expected result was updated in the {} file, continuing...\n\n\n",
                    path.with_extension(ext).display()
                );
                None
            }
            _ => {
                println!("User did not confirm updating the expected result; aborting test run...");
                eprint!("{}", report.join("\n"));
                Some(1)
            }
        }
    }

    fn check_error<T: Serialize>(
        error: T,
        path: &PathBuf,
        report: &mut Vec<String>,
        stdio: &mut Io,
    ) -> Option<i32> {
        let error = serde_json::to_string(&error).unwrap();

        let expected_output = match fs::read_to_string(path.with_extension("fout")) {
            Ok(file) => file,
            Err(_) => {
                eprintln!("{}", report.join("\n"));
                eprintln!("{}\n", error);
                eprintln!(
                    "[ERROR] Could not find or open {}, perhaps the input file expected to succeed but it failed; see the error above.",
                    path.with_extension("fout").display()
                );
                
                return Some(1);
            }
        };
        let expected_output = expected_output.trim();
        report.push(format!("Output: {}", error));
        report.push(format!("Expect: {}", expected_output));
        report.push(format!("Same? : {}", error == expected_output));
        if error != expected_output {
            ask_to_update_result("fout", report, error, path, stdio)
        } else {
            None
        }
    }

    fn read_source_file(path: &PathBuf) -> Option<String> {
        let source = fs::read_to_string(path);
        if source.is_err() {
            assert!(
                false,
                "Failed opening source file {}:{}",
                &path.display(),
                source.unwrap_err()
            );
        }

        let source = source.unwrap();
        if source.starts_with("#SLA_OVER:") {
            println!(
                "{}",
                format!(
                    "[WARN] Skipping file {}:{}",
                    path.display(),
                    source
                        .split_once("\n")
                        .map_or("", |s| s.0)
                        .replace("#SLA_OVER:", "")
                )
            );
            None
        } else {
            Some(source)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lang::compiler::{self};
    use crate::logger::PolderError;
    use crate::{io::Io, lang::interpreter::Interpreter, logger};
    use log;
    use serde::Serialize;
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
        let start_time = std::time::Instant::now();
        for file in test_files {
            let path = file.path();

            let input_str =
                fs::read_to_string(path.with_extension("in")).unwrap_or_else(|_| String::new());
            let mut input: &[u8] = input_str.as_bytes();
            let mut output: Vec<u8> = Vec::new();
            let mut io = Io::new(&mut input, &mut output);

            let source = read_source_file(&path);
            if source.is_none() {
                continue;
            }

            report.push(format!("Testing file: {}", path.display()));
            report.push(format!("{:-<1$}", "", 40));

            let mut ast = compiler::compile(source.unwrap().as_str()).unwrap_or_else(|err| {
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
        let end_time = std::time::Instant::now();
        println!(
            "{:<50} {:>4} files OK ({:>4.3} ms)",
            test_dir.into_string().unwrap(),
            file_count,
            (end_time - start_time).as_micros() as f64 / 1000.0f64
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
        let start_time = std::time::Instant::now();
        for file in test_files {
            let path = file.path();
            let file_name = path.file_name().unwrap().to_str().unwrap();

            let input_str =
                fs::read_to_string(path.with_extension("in")).unwrap_or_else(|_| String::new());
            let mut input: &[u8] = input_str.as_bytes();

            let mut output: Vec<u8> = Vec::new();
            let mut io = Io::new(&mut input, &mut output);

            let source = read_source_file(&path);
            if source.is_none() {
                continue;
            }

            report.push(format!(
                "Testing file: {}",
                test_dir.join(file_name).to_str().unwrap()
            ));
            report.push(format!("{:-<1$}", "", 40));
            let ast = compiler::compile(source.unwrap().as_str());
            if ast.is_err() {
                assert!(check_error(
                    Into::<PolderError>::into(ast.err().unwrap()),
                    &path,
                    &mut report
                ));
                report.clear();
                continue;
            }

            Interpreter::new()
                .run(&mut io, &mut ast.unwrap())
                .map_err(|err| {
                    let err = err
                        .into_iter()
                        .map(|e| e.message)
                        .collect::<Vec<String>>()
                        .join("\n");
                    assert!(check_error(err, &path, &mut report))
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
        let end_time = std::time::Instant::now();
        println!(
            "{:<50} {:>4} files OK ({:>4.3} ms)",
            test_dir.into_string().unwrap(),
            file_count,
            (end_time - start_time).as_micros() as f64 / 1000.0
        );

        fn check_error<T: Serialize>(error: T, path: &PathBuf, report: &mut Vec<String>) -> bool {
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

    fn read_source_file(path: &PathBuf) -> Option<String> {
        let source = fs::read_to_string(path);
        if source.is_err() {
            assert!(
                false,
                "Failed opening source file {}:{}",
                &path.display(),
                source.unwrap_err()
            );
        }

        let source = source.unwrap();
        if source.starts_with("#SLA_OVER:") {
            println!(
                "{}",
                format!(
                    "[WARN] Skipping file {}:{}",
                    path.display(),
                    source
                        .split_once("\n")
                        .map_or("", |s| s.0)
                        .replace("#SLA_OVER:", "")
                )
            );
            None
        } else {
            Some(source)
        }
    }
}
