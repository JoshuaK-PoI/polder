use std::io::{BufWriter, stdin, stdout};

use clap::Parser;

use crate::{
    io::Io,
    lang::{interpreter::Interpreter, program_options::ProgramOptions},
    logger::ErrorLogger,
};

mod args;
mod io;
mod lang;
mod logger;

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

    let program_options = ProgramOptions {
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
    use crate::{
        io::Io,
        lang::{interpreter::Interpreter, program_options::ProgramOptions},
        logger,
    };
    use log;
    use std::{fs, path::Path};

    #[test]
    fn test_happy_path() {
        logger::init(log::LevelFilter::Off)
            .unwrap_or_else(|_| panic!("Logger initialization failed"));
        let program_options = ProgramOptions { print_ast: false };

        let test_dir = Path::new("tests/cases/variabelen/voorbeelden");
        let mut test_files = fs::read_dir(test_dir)
            .unwrap()
            .map(|f| f.unwrap())
            .filter(|f| f.path().extension().unwrap().eq_ignore_ascii_case("pol"))
            .collect::<Vec<_>>();

        test_files.sort_by_key(|f| f.path());

        assert!(&test_files.len() > &0, "No test files found");

        for file in test_files {
            let path = file.path();
            let file_name = path.file_name().unwrap().to_str().unwrap();

            println!("Testing file: {}", file_name);
            println!("{:-<1$}", "", 40);
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

            println!("Output: {:?}", result);
            println!("Expect: {:?}", expected_output);
            println!("Same? : {}", result == expected_output);
            assert!(result == expected_output);
            println!("{}", "");
        }
    }

    #[test]
    fn test_errors() {
        logger::init(log::LevelFilter::Off)
            .unwrap_or_else(|_| panic!("Logger initialization failed"));
        let program_options = ProgramOptions { print_ast: false };

        let test_dir = Path::new("tests/cases/variabelen/fouten");
        let mut test_files = fs::read_dir(test_dir)
            .unwrap()
            .map(|f| f.unwrap())
            .filter(|f| f.path().extension().unwrap().eq_ignore_ascii_case("pol"))
            .collect::<Vec<_>>();

        test_files.sort_by_key(|f| f.path());

        assert!(&test_files.len() > &0, "No test files found");

        for file in test_files {
            let path = file.path();
            let file_name = path.file_name().unwrap().to_str().unwrap();

            println!("Testing file: {}", file_name);
            println!("{:-<1$}", "", 40);
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
            let expected_output = fs::read_to_string(path.with_extension("fout")).unwrap();

            println!("Output: {:?}", result);
            println!("Expect: {:?}", expected_output);
            println!("Same? : {}", result == expected_output);
            assert!(result == expected_output);
            println!("{}", "");
        }
    }
}
