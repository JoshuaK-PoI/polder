mod intrinsic;
mod methods;
mod runtime_error;
mod store;

use crate::{
    io::Io,
    lang::{
        compiler::{
            iter::unshift::Unshift,
            parser::{
                expression::{
                    BinaryExpression, Expression, ExpressionType, MethodCallExpression,
                    UnaryExpression, literal::Literal,
                },
                intrinsic::Intrinsics,
                statement::{
                    BinaryOp, ForeachStatement, IfStatement, IntrinsicStatement, Statement,
                    UnaryOp, VariableAssignmentStatement, VariableDeclarationStatement,
                    WhileStatement,
                },
                types::PolderType,
            },
        },
        interpreter::{
            runtime_error::{RuntimeError, RuntimeErrorKind},
            store::{StoreValue, Variable},
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
            Statement::While(while_statement) => {
                self.interpret_while_statement(while_statement, io)
            }
            Statement::Foreach(foreach_statement) => {
                self.interpret_foreach_statement(foreach_statement, io)
            }
        }
    }

    fn interpret_variable_declaration(
        &mut self,
        declaration: VariableDeclarationStatement,
    ) -> Result<(), RuntimeError> {
        let (store_value, expression_type) = match &declaration.value_expression.expression {
            ExpressionType::Type(type_) => (None, type_.to_owned()),
            _expr => {
                let expr_result = self.interpret_expression(&declaration.value_expression)?;
                let expression_result_type: PolderType = (&expr_result).into();
                (
                    Some((expr_result, declaration.value_expression.span)),
                    expression_result_type,
                )
            }
        };

        let variable = Variable {
            declared_at: declaration.declared_at,
            value: store_value,
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
        let literal = self.interpret_expression(&assignment.expression)?;
        self.store.write(
            assignment.identifier,
            (literal, assignment.expression.span),
            assignment.expression.span,
        )?;
        Ok(())
    }

    fn interpret_expression(&self, expression: &Expression) -> Result<StoreValue, RuntimeError> {
        match &expression.expression {
            ExpressionType::Literal(lit) => Ok(StoreValue::Literal(lit.clone())),
            ExpressionType::Unary(un) => self.interpret_unary_expression(un),
            ExpressionType::Binary(bin) => self.interpret_binary_expression(bin),
            ExpressionType::Identifier(ident) => {
                let variable = self.get_from_store(ident, expression.span)?;
                match variable.get_value() {
                    Some(value) => Ok(value.clone().0),
                    None => Err(runtime_error!(
                        expression.span,
                        RuntimeErrorKind::VariableHasNoValue,
                        ident
                    )),
                }
            }
            ExpressionType::ArrayDecl(arr) => self.interpret_array(arr),
            ExpressionType::MethodCall(method) => {
                self.interpret_method_call(method, expression.span)
            }
            _ => Err(runtime_error!(
                expression.span,
                RuntimeErrorKind::UnsupportedFunction
            )),
        }
    }

    fn interpret_unary_expression(
        &self,
        expression: &UnaryExpression,
    ) -> Result<StoreValue, RuntimeError> {
        let store_value = self.interpret_expression(&expression.expr)?;

        match expression.op {
            UnaryOp::Increment => {
                self.check_numeric(&store_value, &expression.expr)?;
                StoreValue::literal(store_value, expression.expr.span, |v| {
                    v + Literal::Integer(1)
                })
            }
            UnaryOp::Decrement => {
                self.check_numeric(&store_value, &expression.expr)?;
                StoreValue::literal(store_value, expression.expr.span, |v| {
                    v - Literal::Integer(1)
                })
            }
            UnaryOp::Negation => StoreValue::literal(store_value, expression.expr.span, |v| !v),
        }
    }

    fn interpret_binary_expression(
        &self,
        expression: &BinaryExpression,
    ) -> Result<StoreValue, RuntimeError> {
        let lhs = self.interpret_expression(&expression.lhs)?;
        let rhs = self.interpret_expression(&expression.rhs)?;

        self.check_numeric(&lhs, &expression.lhs)?;
        self.check_numeric(&rhs, &expression.rhs)?;

        let lhs = match lhs {
            StoreValue::Literal(l) => l,
            _ => unreachable!(),
        };

        let rhs = match rhs {
            StoreValue::Literal(l) => l,
            _ => unreachable!(),
        };

        Ok(StoreValue::Literal(match expression.op {
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
            BinaryOp::Modulo => Ok(lhs % rhs),
            BinaryOp::Less => Ok(Literal::Boolean(lhs < rhs)),
            BinaryOp::Greater => Ok(Literal::Boolean(lhs > rhs)),
            BinaryOp::Equal => Ok(Literal::Boolean(lhs == rhs)),
        }?))
    }

    fn interpret_intrinsic(
        &mut self,
        intrinsic: IntrinsicStatement,
        io: &mut Io,
    ) -> Result<(), RuntimeError> {
        match intrinsic.intrinsic {
            Intrinsics::Print | Intrinsics::Write => {
                let store_value: Vec<StoreValue> = intrinsic
                    .args
                    .clone()
                    .into_iter()
                    .map(|arg| self.interpret_expression(&arg))
                    .collect::<Result<_, _>>()?;
                intrinsic::print(store_value, intrinsic.intrinsic == Intrinsics::Print, io)
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

    fn interpret_method_call(
        &self,
        method_call_expression: &MethodCallExpression,
        span: (usize, usize),
    ) -> Result<StoreValue, RuntimeError> {
        match &method_call_expression.class.expression {
            ExpressionType::Identifier(identifier) => {
                let mut arguments = Vec::new();

                for argument in &method_call_expression.arguments {
                    arguments.push((self.interpret_expression(argument)?, argument.span));
                }

                methods::call_method(
                    self.get_from_store(identifier, span)?,
                    identifier,
                    &method_call_expression.method_name,
                    arguments,
                    span,
                )
            }
            ExpressionType::Literal(literal) => {
                todo!("Interpret method call for literal {}", literal)
            }
            expression_type => Err(runtime_error!(
                span,
                RuntimeErrorKind::UnsupportedFunction,
                expression_type,
                format!(
                    "Methode aanroepen op een {} is niet mogelijk",
                    expression_type
                )
            )),
        }
    }

    fn interpret_array(&self, array: &Vec<Expression>) -> Result<StoreValue, RuntimeError> {
        let mut result = Vec::new();
        for expression in array {
            result.push(StoreValue::unwrap_literal(
                self.interpret_expression(expression)?,
                expression.span,
            )?)
        }

        Ok(StoreValue::Array(result))
    }

    fn interpret_if_statement(
        &mut self,
        statement: IfStatement,
        io: &mut Io,
    ) -> Result<(), RuntimeError> {
        let condition = self.interpret_expression(&statement.predicate)?;

        if self.is_truthy(&condition) {
            statement
                .then_statements
                .iter()
                .try_for_each(|stmt| self.interpret_statement(stmt.to_owned(), io))?;
        } else {
            statement
                .else_statements
                .iter()
                .try_for_each(|stmt| self.interpret_statement(stmt.to_owned(), io))?;
        }

        Ok(())
    }

    fn interpret_while_statement(
        &mut self,
        statement: WhileStatement,
        io: &mut Io,
    ) -> Result<(), RuntimeError> {
        let mut condition = self.interpret_expression(&statement.predicate)?;

        while self.is_truthy(&condition) {
            statement
                .then_statements
                .iter()
                .try_for_each(|stmt| self.interpret_statement(stmt.to_owned(), io))?;
            condition = self.interpret_expression(&statement.predicate)?;
        }

        Ok(())
    }

    fn is_truthy(&self, condition: &StoreValue) -> bool {
        match condition {
            StoreValue::Literal(literal) => match literal {
                Literal::Boolean(b) => *b,
                Literal::Char(c) => *c != '\0',
                Literal::Float(f) => *f != 0f64,
                Literal::Integer(i) => *i != 0,
                Literal::String(s) => !s.is_empty(),
            },
            StoreValue::Array(elements) => !elements.is_empty(),
        }
    }

    fn get_from_store(
        &self,
        variable_identifier: &String,
        span: (usize, usize),
    ) -> Result<&Variable, RuntimeError> {
        match self.store.get(variable_identifier) {
            Some(variable) => Ok(variable),
            None => Err(runtime_error!(
                span,
                RuntimeErrorKind::UndefinedVariable,
                variable_identifier
            )),
        }
    }

    fn check_numeric(
        &self,
        store_value: &StoreValue,
        expr: &Expression,
    ) -> Result<(), RuntimeError> {
        match store_value {
            StoreValue::Literal(lit)
                if [PolderType::Integer, PolderType::Float].contains(&(lit).into()) =>
            {
                Ok(())
            }
            _ => Err(runtime_error!(
                expr.span,
                RuntimeErrorKind::CannotCalculateNonNumeric,
                format!(""),
                format!("Waarde '{}' is een {}", store_value, store_value.get_type())
            )),
        }
    }

    fn interpret_foreach_statement(
        &mut self,
        foreach_statement: ForeachStatement,
        io: &mut Io<'_>,
    ) -> Result<(), RuntimeError> {
        let variable = match self.store.get(foreach_statement.list_name.as_str()) {
            Some(var) => Ok(var),
            None => Err(runtime_error!(
                foreach_statement.span,
                RuntimeErrorKind::UndefinedVariable,
                foreach_statement.list_name
            )),
        }?
        .clone();

        match variable.type_ {
            PolderType::Array(_) => {
                let (store_value, span) = match variable.get_value() {
                    Some((value, span)) => Ok((value, span)),
                    None => Err(runtime_error!(
                        foreach_statement.span,
                        RuntimeErrorKind::VariableHasNoValue,
                        format!(
                            "Kan geen 'voor elke' lus uitvoeren voor een ongeïnitialiseerde variabele"
                        )
                    )),
                }?;

                match store_value {
                    StoreValue::Array(elements) => {
                        for (index, element) in elements.iter().enumerate() {
                            self.store.declare(
                                foreach_statement.element_binding.clone(),
                                Variable {
                                    declared_at: foreach_statement.span,
                                    type_: element.into(),
                                    value: Some((StoreValue::Literal(element.clone()), *span)),
                                    is_const: true,
                                },
                                *span,
                            )?;

                            if foreach_statement.index_binding.is_some() {
                                self.store.declare(
                                    foreach_statement.index_binding.as_ref().unwrap().clone(),
                                    Variable {
                                        declared_at: foreach_statement.span,
                                        type_: PolderType::Integer,
                                        value: Some((
                                            StoreValue::Literal(Literal::Integer(index as i64)),
                                            foreach_statement.span,
                                        )),
                                        is_const: true,
                                    },
                                    *span,
                                )?;
                            }

                            foreach_statement.statements.iter().try_for_each(|stmt| {
                                self.interpret_statement(stmt.to_owned(), io)
                            })?;

                            self.store.delete(&foreach_statement.element_binding);

                            if foreach_statement.index_binding.is_some() {
                                self.store
                                    .delete(&foreach_statement.index_binding.as_ref().unwrap())
                            }
                        }
                    }
                    _ => unreachable!(),
                }

                Ok(())
            }
            _ => Err(runtime_error!(
                foreach_statement.span,
                RuntimeErrorKind::MismatchedType,
                foreach_statement.list_name,
                format!("'Voor elke' is niet geldig voor een {}", variable.type_)
            )),
        }
    }
}
