use std::{ops::Range, path::Path};

use log::{debug, info, trace};

mod intrinsic;
mod runtime_error;
mod store;

use crate::{
    io::Io,
    lang::{
        compiler::{
            self,
            iter::unshift::Unshift,
            parser::statement::{
                SpannedStatement, StatementType, VariableAssignmentStatement,
                VariableDeclarationStatement,
            },
        },
        interpreter::runtime_error::RuntimeError,
    },
    logger::PolderError,
    program_options::ProgramOptions,
};
use store::Store;

pub(crate) struct Interpreter<'a> {
    input: &'a str,
    filename: &'a str,
    program_options: &'a ProgramOptions,
    store: Store,
}

impl Interpreter<'_> {
    pub(crate) fn new<'a>(
        input: &'a str,
        filename: &'a str,
        program_options: &'a ProgramOptions,
    ) -> Interpreter<'a> {
        let store = Store::new();

        Interpreter {
            input,
            filename,
            program_options,
            store,
        }
    }

    pub(crate) fn run(&mut self, io: &mut Io) -> Result<(), PolderError> {
        let mut ast = compiler::compile(&self.input)?;

        if self.program_options.print_ast {
            let ast_outname = self.filename.to_owned() + "__ast.json";
            std::fs::write(
                Path::new(&ast_outname),
                format!("{}", serde_json::to_string_pretty(&ast).unwrap()),
            )
            .map_err(|e| PolderError {
                message: format!("Failed to write AST to file: {}", e.to_string()),
                span_message: "".to_string(),
                span: 0..0,
            })?;
            info!("Wrote AST output to {}", &ast_outname);
        }

        debug!("[interpreter] Compile OK, starting interpreter...");

        let mut ast = ast.iter_mut().peekable();

        while let Some(stmt) = ast.unshift() {
            self.interpret_stmt(stmt, io)?;
        }

        Ok(())
    }

    fn interpret_stmt(&mut self, stmt: &SpannedStatement, io: &mut Io) -> Result<(), RuntimeError> {
        trace!("[interpreter] Interpreting statement {}...", stmt.0);
        match stmt.0 {
            StatementType::VariableDeclaration(ref declaration) => {
                self.interpret_variable_declaration(declaration, &stmt.1)
            }
            StatementType::VariableAssignment(ref assignment) => {
                self.interpret_variable_assignment(assignment, &stmt.1)
            }
            StatementType::Intrinsic(ref intrinsic) => {
                intrinsic::interpret(intrinsic, &stmt.1, &mut self.store, io)
            }
        }
    }

    fn interpret_variable_declaration(
        &mut self,
        declaration: &VariableDeclarationStatement,
        range: &Range<usize>,
    ) -> Result<(), RuntimeError> {
        self.store.declare(declaration.into(), range)?;
        Ok(())
    }

    fn interpret_variable_assignment(
        &mut self,
        assignment: &VariableAssignmentStatement,
        range: &Range<usize>,
    ) -> Result<(), RuntimeError> {
        self.store.update(assignment, range)?;
        Ok(())
    }
}
