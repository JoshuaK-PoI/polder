mod intrinsic;
mod runtime_error;
mod store;

use crate::{
    io::Io,
    lang::{
        compiler::{
            iter::unshift::Unshift,
            parser::{
                expression::{
                    BinaryExpression, Expression, ExpressionType, UnaryExpression, literal::Literal,
                },
                intrinsic::Intrinsics,
                statement::{
                    BinaryOp, IfStatement, IntrinsicStatement, Statement, UnaryOp,
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
    runtime_error,
};
use store::Store;

pub(crate) struct Interpreter {
    store: Store,
}

impl Interpreter {
    pub(crate) fn new() -> Interpreter {
        Interpreter {
            store: Store::new(),
        }
    }

    pub(crate) fn run(&mut self, io: &mut Io, ast: &mut Vec<Statement>) -> Result<(), PolderError> {
        let mut ast = ast.iter_mut().peekable();

        while let Some(stmt) = ast.unshift() {
            self.interpret_statement(stmt.to_owned(), io)?;
        }

        Ok(())
    }

    fn interpret_statement(&mut self, stmt: Statement, io: &mut Io) -> Result<(), RuntimeError> {
        match stmt {
            Statement::VariableDeclaration(declaration) => {
                self.interpret_variable_declaration(declaration)
            }
            Statement::VariableAssignment(assignment) => {
                self.interpret_variable_assignment(assignment)
            }
            Statement::Intrinsic(intrinsic) => self.interpret_intrinsic(intrinsic, io),
            Statement::If(if_statement) => self.interpret_if_statement(if_statement, io),
        }
    }

    fn interpret_variable_declaration(
        &mut self,
        declaration: VariableDeclarationStatement,
    ) -> Result<(), RuntimeError> {
        let (literal, expression_type) = match &declaration.value_expression.expression {
            ExpressionType::Type(type_) => (None, type_.to_owned()),
            _expr => {
                let expr_result = self.interpret_expression(&declaration.value_expression)?;
                let expression_result_type: PolderType = (&expr_result).into();
                (Some(expr_result), expression_result_type)
            }
        };

        let variable = Variable {
            declared_at: declaration.declared_at,
            value: literal,
            type_: expression_type,
            is_const: declaration.is_const,
        };

        self.store
            .declare(declaration.identifier, variable, declaration.declared_at)?;
        Ok(())
    }

    fn interpret_variable_assignment(
        &mut self,
        assignment: VariableAssignmentStatement,
    ) -> Result<(), RuntimeError> {
        let expression = self.interpret_expression(&assignment.expression)?;
        self.store.write(
            assignment.identifier,
            &expression,
            assignment.expression.span,
        )?;
        Ok(())
    }

    fn interpret_expression(&self, expression: &Expression) -> Result<Literal, RuntimeError> {
        match &expression.expression {
            ExpressionType::Literal(lit) => Ok(lit.clone()),
            ExpressionType::Unary(un) => self.interpret_unary_expression(un),
            ExpressionType::Binary(bin) => self.interpret_binary_expression(bin),
            ExpressionType::Identifier(ident) => self.get_from_store(ident, expression.span),
            _ => Err(runtime_error!(
                expression.span,
                RuntimeErrorKind::UnsupportedFunction
            )),
        }
    }

    fn interpret_unary_expression(
        &self,
        expression: &UnaryExpression,
    ) -> Result<Literal, RuntimeError> {
        let expr = self.interpret_expression(&expression.expr)?;

        match expression.op {
            UnaryOp::Increment => {
                self.check_numeric(&expr, &expression.expr)?;
                Ok(expr + Literal::Integer(1))
            }
            UnaryOp::Decrement => {
                self.check_numeric(&expr, &expression.expr)?;
                Ok(expr - Literal::Integer(1))
            }
            UnaryOp::Negation => Ok(!expr),
        }
    }

    fn interpret_binary_expression(
        &self,
        expression: &BinaryExpression,
    ) -> Result<Literal, RuntimeError> {
        let lhs = self.interpret_expression(&expression.lhs)?;
        let rhs = self.interpret_expression(&expression.rhs)?;

        self.check_numeric(&lhs, &expression.lhs)?;
        self.check_numeric(&rhs, &expression.rhs)?;

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
                        expression.rhs.span,
                        RuntimeErrorKind::DivideByZero
                    ))
                } else {
                    Ok(lhs / rhs)
                }
            }
            BinaryOp::Less => Ok(Literal::Boolean(lhs < rhs)),
            BinaryOp::Greater => Ok(Literal::Boolean(lhs > rhs)),
            BinaryOp::Equal => Ok(Literal::Boolean(lhs == rhs)),
        }
    }

    fn interpret_intrinsic(
        &mut self,
        intrinsic: IntrinsicStatement,
        io: &mut Io,
    ) -> Result<(), RuntimeError> {
        match intrinsic.intrinsic {
            Intrinsics::Print | Intrinsics::Write => {
                let literal: Vec<Literal> = intrinsic
                    .args
                    .clone()
                    .into_iter()
                    .map(|arg| self.interpret_expression(&arg))
                    .collect::<Result<_, _>>()?;
                intrinsic::print(literal, intrinsic.intrinsic == Intrinsics::Print, io)
            }
            Intrinsics::Readline => {
                let (identifier, span) = match intrinsic.args.get(0) {
                    Some(ref expr) => match expr.expression {
                        ExpressionType::Identifier(ref ident) => Ok((ident.clone(), expr.span)),
                        ref other => Err(runtime_error!(
                            expr.span,
                            RuntimeErrorKind::MismatchedType,
                            other
                        )),
                    },
                    None => Err(runtime_error!(
                        intrinsic.span,
                        RuntimeErrorKind::NoArgumentsProvided
                    )),
                }?;

                if self.store.get(&identifier).is_none() {
                    return Err(runtime_error!(
                        span,
                        RuntimeErrorKind::UndefinedVariable,
                        identifier
                    ));
                }

                let value = intrinsic::readline(span, io)?;
                self.store.write_input(identifier, value, span)?;
                Ok(())
            }
        }
    }

    fn interpret_if_statement(
        &mut self,
        statement: IfStatement,
        io: &mut Io,
    ) -> Result<(), RuntimeError> {
        let condition = self.interpret_expression(&statement.check_expression)?;

        if self.is_truthy(&condition) {
            self.interpret_statement(*statement.then_statement, io)?;
        } else if let Some(else_stmt) = *statement.else_statement {
            self.interpret_statement(else_stmt, io)?;
        }

        Ok(())
    }

    fn is_truthy(&self, condition: &Literal) -> bool {
        match condition {
            Literal::Boolean(b) => *b,
            Literal::Char(c) => *c != '\0',
            Literal::Float(f) => *f != 0f64,
            Literal::Integer(i) => *i != 0,
            Literal::String(s) => !s.is_empty(),
        }
    }

    fn get_from_store(
        &self,
        variable_identifier: &String,
        span: (usize, usize),
    ) -> Result<Literal, RuntimeError> {
        match self.store.get(variable_identifier) {
            Some(variable) => match variable.get_value() {
                Some(value) => Ok(value.to_owned()),
                None => Err(runtime_error!(
                    span,
                    RuntimeErrorKind::VariableHasNoValue,
                    variable_identifier
                )),
            },
            None => Err(runtime_error!(
                span,
                RuntimeErrorKind::UndefinedVariable,
                variable_identifier
            )),
        }
    }

    fn check_numeric(&self, lit: &Literal, expr: &Expression) -> Result<(), RuntimeError> {
        if ![PolderType::Integer, PolderType::Float].contains(&(lit).into()) {
            Err(runtime_error!(
                expr.span,
                RuntimeErrorKind::CannotCalculateNonNumeric
            ))
        } else {
            Ok(())
        }
    }
}
