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
            parser::{
                expression::{
                    BinaryExpression, Expression, ExpressionType, UnaryExpression, literal::Literal,
                },
                statement::{
                    BinaryOp, SpannedStatement, StatementType, UnaryOp,
                    VariableAssignmentStatement, VariableDeclarationStatement,
                },
                types::PolderType,
            },
        },
        interpreter::{
            runtime_error::{RuntimeError, RuntimeErrorKind},
            store::Variable,
        },
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
        }
    }

    fn interpret_variable_declaration(
        &mut self,
        declaration: &VariableDeclarationStatement,
        range: &Range<usize>,
    ) -> Result<(), RuntimeError> {
        let (expression, expression_type) = match &declaration.value_expression.expression {
            ExpressionType::Type(type_) => (None, type_.to_owned()),
            _expr => {
                let expr_result = self.interpret_expression(&declaration.value_expression)?;
                let expression_result_type: PolderType = (&expr_result).into();
                (Some(expr_result), expression_result_type)
            }
        };

        let variable = Variable {
            declared_at: declaration.span.clone(),
            identifier: declaration.identifier.clone(),
            value: expression,
            type_: expression_type,
            is_const: declaration.is_const,
        };

        self.store.declare(variable, range)?;
        Ok(())
    }

    fn interpret_variable_assignment(
        &mut self,
        assignment: &VariableAssignmentStatement,
        range: &Range<usize>,
    ) -> Result<(), RuntimeError> {
        let expression = self.interpret_expression(&assignment.expression)?;
        self.store
            .write(&assignment.identifier, &expression, range)?;
        Ok(())
    }

    fn interpret_expression<'a>(
        &'a self,
        expression: &'a Expression,
    ) -> Result<Literal, RuntimeError> {
        match &expression.expression {
            ExpressionType::Literal(lit) => Ok(lit.clone()),
            ExpressionType::Unary(un) => self.interpret_unary_expression(un),
            ExpressionType::Binary(bin) => self.interpret_binary_expression(bin),
            ExpressionType::Identifier(ident) => self.get_from_store(ident, &expression.span),
            _ => Err(runtime_error!(
                expression.span.clone(),
                RuntimeErrorKind::UnsupportedFunction
            )),
        }
    }

    fn interpret_unary_expression(
        &self,
        expression: &UnaryExpression,
    ) -> Result<Literal, RuntimeError> {
        let expr = self.interpret_expression(&expression.expr)?;

        if ![PolderType::Integer, PolderType::Float].contains(&(&expr).into()) {
            return Err(runtime_error!(
                expression.expr.span.clone(),
                RuntimeErrorKind::CannotCalculateNonNumeric
            ));
        }

        match expression.op {
            UnaryOp::Increment => Ok(expr + Literal::Integer(1)),
            UnaryOp::Decrement => Ok(expr - Literal::Integer(1)),
        }
    }

    fn interpret_binary_expression(
        &self,
        expression: &BinaryExpression,
    ) -> Result<Literal, RuntimeError> {
        let lhs = self.interpret_expression(&expression.lhs)?;
        let rhs = self.interpret_expression(&expression.rhs)?;

        if ![PolderType::Integer, PolderType::Float].contains(&(&lhs).into())
            || ![PolderType::Integer, PolderType::Float].contains(&(&rhs).into())
        {
            return Err(runtime_error!(
                expression.lhs.span.start..expression.rhs.span.end,
                RuntimeErrorKind::CannotCalculateNonNumeric
            ));
        }

        match expression.op {
            BinaryOp::Add => Ok(lhs + rhs),
            BinaryOp::Subtract => Ok(lhs - rhs),
            BinaryOp::Multiply => Ok(lhs * rhs),
            BinaryOp::Divide => {
                if match rhs {
                    Literal::Integer(int) => int == 0,
                    Literal::Float(flt) => flt == 0f64,
                    _ => unreachable!(),
                } {
                    Err(runtime_error!(
                        expression.rhs.span.clone(),
                        RuntimeErrorKind::DivideByZero
                    ))
                } else {
                    Ok(lhs / rhs)
                }
            }
        }
    }

    fn get_from_store(
        &self,
        variable_identifier: &String,
        span: &Range<usize>,
    ) -> Result<Literal, RuntimeError> {
        match self.store.get(variable_identifier) {
            Some(variable) => match variable.get_value() {
                Some(value) => Ok(value.clone()),
                None => Err(runtime_error!(
                    span.clone(),
                    RuntimeErrorKind::VariableHasNoValue,
                    variable_identifier
                )),
            },
            None => Err(runtime_error!(
                span.clone(),
                RuntimeErrorKind::UndefinedVariable,
                variable_identifier
            )),
        }
    }
}
