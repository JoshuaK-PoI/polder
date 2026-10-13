use std::{ops::Range, path::Path};

use log::{debug, info, trace};

mod store;

use crate::{
    lang::{
        compiler::{
            Compiler, CompilerError, ErrorCode, expression::ExpressionType, intrinsic::Intrinsics, statement::{
                IntrinsicStatement, SpannedStatement, StatementType, VariableAssignmentStatement, VariableDeclarationStatement,
            }, unshift::Unshift,
        }, program_options::ProgramOptions,
    }, logger::ErrorLogger,
};
use store::Store;

pub(crate) struct Interpreter<'a> {
    input: &'a str,
    filename: &'a str,
    error_logger: ErrorLogger<'a>,
    program_options: &'a ProgramOptions,
    store: Store,
}

impl Interpreter<'_> {
    pub(crate) fn new<'a>(
        input: &'a str,
        filename: &'a str,
        program_options: &'a ProgramOptions,
    ) -> Interpreter<'a> {
        let error_logger = ErrorLogger::new(input, filename);
        let store = Store::new();

        Interpreter {
            input,
            filename,
            error_logger,
            program_options,
            store,
        }
    }

    pub(crate) fn run(&mut self) -> Result<String, String> {
        let mut compiler = Compiler::new(&self.input, &self.filename);
        let mut ast = compiler.compile().map_err(|err| {
            self.error_logger.report(&err);
            format!("Error compiling file: {}", err.error_message)
        })?;

        if self.program_options.print_ast {
            let ast_outname = self.filename.to_owned() + "__ast.json";
            std::fs::write(
                Path::new(&ast_outname),
                format!("{}", serde_json::to_string_pretty(&ast).unwrap()),
            )
            .map_err(|e| format!("Failed to write AST to file: {}", e.to_string()))?;
            info!("Wrote AST output to {}", &ast_outname);
        }

        debug!("[interpreter] Compile OK, starting interpreter...");

        let mut output = Vec::<String>::new();
        let mut ast = ast.iter_mut().peekable();

        while let Some(stmt) = ast.unshift() {
            let stmt = self.interpret_stmt(stmt).map_err(|err| {
                self.error_logger.report(&err);
                format!("Error interpreting program: {}", err.error_message)
            })?;

            if stmt.is_some() {
                output.push(stmt.unwrap());
            }
        }

        Ok(output.join("\n").to_string())
    }

    fn interpret_stmt(&mut self, stmt: &SpannedStatement) -> Result<Option<String>, CompilerError> {
        trace!("[interpreter] Interpreting statement {}...", stmt.0);
        match stmt.0 {
            StatementType::VariableDeclaration(ref declaration) => self
                .interpret_variable_declaration(declaration, &stmt.1)
                .map(|_| None),
            StatementType::VariableAssignment(ref assignment) => {
                self.interpret_variable_assignment(assignment, &stmt.1)
                .map(|_| None)
            }
            StatementType::Intrinsic(ref intrinsic) => self.interpret_intrinsic(intrinsic, &stmt.1),
        }
    }

    fn interpret_intrinsic(
        &self,
        intrinsic: &IntrinsicStatement,
        range: &Range<usize>,
    ) -> Result<Option<String>, CompilerError> {
        let mut output = Vec::<String>::new();
        match intrinsic.intrinsic {
            Intrinsics::Print => self.intrinsic_print(intrinsic, range, &mut output)?,
        };

        Ok(Some(output.join("\n").to_string()))
    }

    fn intrinsic_print(
        &self,
        intrinsic: &IntrinsicStatement,
        range: &Range<usize>,
        output: &mut Vec<String>,
    ) -> Result<(), CompilerError> {
        if intrinsic.args.is_empty() {
            Err(CompilerError {
                error_code: ErrorCode::NoArgumentsProvided,
                error_message: format!("No arguments provided for print statement"),
                span_message: format!("Expected something to print, got nothing"),
                span: range.clone(),
                help: None,
                info: None,
            })
        } else {
            for arg in intrinsic.args.iter() {
                match arg.expression {
                    ExpressionType::StringExpr(ref content) => output.push(content.text.clone()),
                    ExpressionType::Identifier(ref content) => {
                        let value = self.store.get_variable_by_name(&content.name);
                        if value.is_none() {
                            return Err(CompilerError {
                                error_code: ErrorCode::UndefinedVariable,
                                error_message: format!(
                                    "Call to undefined variable {} ",
                                    &content.name
                                ),
                                span_message: format!("{} is not defined", &content.name),
                                span: range.clone(),
                                help: None,
                                info: None,
                            });
                        }
                        output.push(value.unwrap().get_value().clone().unwrap());
                    }
                }
            }
            Ok(())
        }
    }

    fn interpret_variable_declaration(
        &mut self,
        declaration: &VariableDeclarationStatement,
        range: &Range<usize>,
    ) -> Result<(), CompilerError> {
        self.store.declare(declaration.into(), range)?;
        Ok(())
    }

    fn interpret_variable_assignment(
        &mut self,
        assignment: &VariableAssignmentStatement,
        range: &Range<usize>,
    ) -> Result<(), CompilerError> {
        self.store.update(assignment, range)?;
        Ok(())
    }
}
