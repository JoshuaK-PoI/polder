use clap::Parser;

use crate::lang::{
    interpreter::Interpreter,
    printer::{PrintMode, Printer},
};

mod args;
mod lang;
mod logger;

fn main() {
    let args = args::Args::parse();

    logger::init(if args.debug {
        log::LevelFilter::Trace
    } else {
        log::LevelFilter::Info
    })
    .expect("Could not initialize logger");

    let input = std::fs::read_to_string(&args.input).unwrap_or_else(|err| {
        panic!("Error reading file: {} ", err);
    });

    let mut interpreter = Interpreter::new(&input, &args.input);

    let output = interpreter
        .run()
        .unwrap_or_else(|err| panic!("Error interpreting: {}", err));

    let printer = Printer::init(PrintMode::Stdout);
    printer.print(output);
}

#[cfg(test)]
mod tests {
    use crate::{lang::interpreter::Interpreter, logger};
    use log;
    use log::debug;
    use std::{fs, path::Path};

    #[test]
    fn should_open_file() {
        logger::init(log::LevelFilter::Off)
            .unwrap_or_else(|_| panic!("Logger initialization failed"));

        let test_dir = Path::new("tests/cases/variabelen/voorbeelden");
        let test_files = fs::read_dir(test_dir).unwrap().collect::<Vec<_>>();

        assert!(&test_files.len() > &0, "No test files found");

        for file in test_files {
            let file = file.unwrap();
            let path = file.path();
            let file_name = path.file_name().unwrap().to_str().unwrap();
            let file_ext = path.extension().unwrap().to_str().unwrap();

            if file_ext == "pol" {
                println!("Testing file: {}", file_name);
                let input = fs::read_to_string(&path).unwrap();
                let input = input.as_str();
                let mut interpreter = Interpreter::new(input, file_name);

                let output = interpreter.run().unwrap_or_else(|err| {
                    panic!("Error compiling file: {}", err);
                });

                let expected_output = fs::read_to_string(path.with_extension("uit"))
                    .unwrap_or_else(|err| {
                        panic!("Error opening outfile: {}", err);
                    });

                let expected_output = expected_output.trim().to_string();

                println!("Output: {:?}", output);
                println!("Expected output: {:?}", expected_output);
                println!("Same: {}", output == expected_output);
                assert!(output == expected_output)
            } else {
                debug!("Skipping file: {} (not a Polder source file)", file_name);
            }
        }
    }
}
