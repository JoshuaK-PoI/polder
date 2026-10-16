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
            lexer::token::{Token, TokenType},
            parser::statement::{
                ArithmeticOp, ArithmeticStatement, SpannedStatement, StatementType,
                VariableAssignmentStatement, VariableDeclarationStatement,
            },
        },
        interpreter::runtime_error::{RuntimeError, RuntimeErrorKind},
    },
    logger::PolderError,
    program_options::ProgramOptions,
    runtime_error,
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
            StatementType::Arithmetic(ref arith_op) => self.interpret_arithmetic(arith_op, &stmt.1),
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

    fn interpret_arithmetic(
        &mut self,
        arith_op: &ArithmeticStatement,
        _span: &Range<usize>,
    ) -> Result<(), RuntimeError> {
        let dst = if [ArithmeticOp::Add, ArithmeticOp::Subtract].contains(&arith_op.operation) {
            &arith_op.rhs
        } else {
            &arith_op.lhs
        };

        let left = self.value_as_number(&arith_op.lhs)?;
        let right = self.value_as_number(&arith_op.rhs)?;

        let result = match arith_op.operation {
            ArithmeticOp::Add => right + left,
            ArithmeticOp::Subtract => right - left,
            ArithmeticOp::Multiply => left * right,
            ArithmeticOp::Divide => {
                if right == 0 {
                    return Err(runtime_error!(
                        arith_op.rhs.span.clone(),
                        RuntimeErrorKind::DivideByZero
                    ));
                }
                left / right
            }
        };

        self.store.set(&dst.value, result.to_string(), &dst.span)?;

        Ok(())
    }

    fn value_as_number(&mut self, token: &Token) -> Result<usize, RuntimeError> {
        match token.type_ {
            TokenType::Identifier => Ok(self
                .store
                .get(&token.value)
                .unwrap()
                .get_value()
                .as_ref()
                .unwrap()
                .value
                .parse::<usize>()
                .unwrap()),
            TokenType::IntegerLiteral => Ok(token.value.parse::<usize>().unwrap()),
            TokenType::FloatLiteral => todo!(),
            _ => Err(runtime_error!(
                token.span.clone(),
                RuntimeErrorKind::MismatchedType,
                format!("Verwachtte een getal, maar vond '{}'", token.type_)
            )),
        }
    }
}
