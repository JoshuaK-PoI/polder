use clap::Parser;

use crate::lang::{
    interpreter::Interpreter,
    printer::{PrintMode, Printer},
    program_options::ProgramOptions,
};

mod args;
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

    let input = std::fs::read_to_string(&args.input).unwrap_or_else(|err| {
        panic!("Error reading file: {} ", err);
    });

    let output = Interpreter::new(&input, &args.input, &program_options)
        .run()
        .unwrap_or_else(|err| panic!("Error interpreting: {}", err));

    Printer::init(PrintMode::Stdout).print(output);
}

#[cfg(test)]
mod tests {
    use crate::{
        lang::{interpreter::Interpreter, program_options::ProgramOptions},
        logger,
    };
    use log;
    use std::{fs, path::Path};

    #[test]
    fn should_open_file() {
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
            let input = fs::read_to_string(&path).unwrap();
            let input = input.as_str();
            let mut interpreter = Interpreter::new(input, file_name, &program_options);

            let output = interpreter.run().unwrap();
            let expected_output = fs::read_to_string(path.with_extension("uit")).unwrap();
            let expected_output = expected_output.trim().to_string();

            println!("Output: {:?}", output);
            println!("Expect: {:?}", expected_output);
            println!("Same? : {}", output == expected_output);
            assert!(output == expected_output);
            println!("{}", "");
        }
    }
}
