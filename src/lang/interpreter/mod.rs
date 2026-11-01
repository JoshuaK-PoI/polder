mod intrinsic;
mod methods;
mod runtime_error;
mod store;

use crate::{
    io::Io,
    lang::{
        compiler::{
            Span,
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

#[hotpath::measure_all]
impl Interpreter {
    pub(crate) fn new() -> Interpreter {
        Interpreter {
            store: Store::new(),
        }
    }
    pub(crate) fn run(&mut self, io: &mut Io, ast: &Vec<Statement>) -> Result<(), PolderError> {
        let mut ast = ast.iter().peekable();

        while let Some(stmt) = ast.unshift() {
            self.interpret_statement(stmt, io)?;
        }

        Ok(())
    }

    fn interpret_statement(&mut self, stmt: &Statement, io: &mut Io) -> Result<(), RuntimeError> {
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
            Statement::Comment => Ok(()), /* no-op */
        }
    }

    fn interpret_variable_declaration(
        &mut self,
        declaration: &VariableDeclarationStatement,
    ) -> Result<(), RuntimeError> {
        let (store_value, expression_type) = match &declaration.initializer {
            Some(expr) => {
                let expr_result = self.interpret_expression(expr)?;
                let expression_result_type: PolderType = (&expr_result).into();
                (Some(expr_result), Some(expression_result_type))
            }
            None => (None, None),
        };

        self.store.declare(
            &declaration.identifier.value,
            Variable {
                value: store_value,
                type_: declaration.type_
                    .as_ref()
                    .or(expression_type.as_ref())
                    .expect(format!("Interpreter: Tried to declare variable {} without a type (none in either the declaration or the initializer expression", declaration.identifier.value).as_str())
                    .clone(),
            },
        );
        Ok(())
    }

    fn interpret_variable_assignment(
        &mut self,
        assignment: &VariableAssignmentStatement,
    ) -> Result<(), RuntimeError> {
        let literal = self.interpret_expression(&assignment.expression)?;
        self.store.write(
            &assignment.identifier.value,
            (literal, assignment.expression.span),
            assignment.expression.span,
        )?;
        Ok(())
    }

    fn interpret_expression(&self, expression: &Expression) -> Result<Literal, RuntimeError> {
        match &expression.expression {
            ExpressionType::Literal(lit) => Ok(lit.clone()),
            ExpressionType::Unary(un) => self.interpret_unary_expression(un),
            ExpressionType::Binary(bin) => self.interpret_binary_expression(bin, expression.span),
            ExpressionType::Identifier(ident) => {
                Ok(self.store.get(&ident.value).value.clone().unwrap())
            }
            ExpressionType::ArrayDecl(arr) => self.interpret_array(arr),
            ExpressionType::MethodCall(method) => {
                self.interpret_method_call(method, expression.span)
            }
        }
    }

    fn interpret_unary_expression(
        &self,
        expression: &UnaryExpression,
    ) -> Result<Literal, RuntimeError> {
        let store_value = self.interpret_expression(&expression.expr)?;

        match expression.op {
            UnaryOp::Increment => {
                self.check_numeric(&store_value, &expression.expr)?;
                Ok(store_value + Literal::Integer(1))
            }
            UnaryOp::Decrement => {
                self.check_numeric(&store_value, &expression.expr)?;
                Ok(store_value - Literal::Integer(1))
            }
            UnaryOp::Negation => Ok(!store_value),
        }
    }

    fn interpret_binary_expression(
        &self,
        expression: &BinaryExpression,
        span: Span,
    ) -> Result<Literal, RuntimeError> {
        let lhs = self.interpret_expression(&expression.lhs)?;
        let rhs = self.interpret_expression(&expression.rhs)?;

        if expression.op == BinaryOp::Equal {
            return self.interpret_equality_expression(lhs, rhs, span);
        }
        self.check_numeric(&lhs, &expression.lhs)?;
        self.check_numeric(&rhs, &expression.rhs)?;

        Ok(match expression.op {
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
            _ => unreachable!(),
        }?)
    }

    fn interpret_equality_expression(
        &self,
        lhs: Literal,
        rhs: Literal,
        span: Span,
    ) -> Result<Literal, RuntimeError> {
        Ok(match lhs {
            Literal::Vec(lhs) => match rhs {
                Literal::Vec(rhs) => {
                    if lhs.len() != rhs.len() {
                        return Ok(Literal::Boolean(false));
                    }

                    for (index, element) in lhs.iter().enumerate() {
                        let rhs_elem = rhs.get(index);
                        if rhs_elem.is_none() {
                            return Ok(Literal::Boolean(false));
                        }
                        let rhs_elem = rhs_elem.unwrap();

                        if *element != *rhs_elem {
                            return Ok(Literal::Boolean(false));
                        }
                    }

                    Ok(Literal::Boolean(true))
                }
                rhs => Err(runtime_error!(
                    span,
                    RuntimeErrorKind::InvalidComparison,
                    "".to_string(),
                    format!(
                        "Incompatibele types aan de linker- en rechterkant van de vergelijking ({}, {})",
                        lhs.get(0).unwrap().get_type(),
                        rhs.get_type()
                    )
                )),
            },
            lhs => match rhs {
                Literal::Vec(_) => Err(runtime_error!(
                    span,
                    RuntimeErrorKind::InvalidComparison,
                    "".to_string(),
                    format!(
                        "Incompatibele types aan de linker- en rechterkant van de vergelijking ({}, {})",
                        lhs.get_type(),
                        rhs.get_type()
                    )
                )),
                rhs => Ok(Literal::Boolean(lhs == rhs)),
            },
        }?)
    }

    fn interpret_intrinsic(
        &mut self,
        intrinsic: &IntrinsicStatement,
        io: &mut Io,
    ) -> Result<(), RuntimeError> {
        match intrinsic.intrinsic {
            Intrinsics::Print | Intrinsics::Write => {
                let store_value: Vec<Literal> = intrinsic
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

                let value = intrinsic::readline(span, io)?;
                self.store.write_input(identifier.value, value, span)?;
                Ok(())
            }
        }
    }

    fn interpret_method_call(
        &self,
        method_call_expression: &MethodCallExpression,
        span: Span,
    ) -> Result<Literal, RuntimeError> {
        match &method_call_expression.class.expression {
            ExpressionType::Identifier(identifier) => {
                let mut arguments = Vec::new();

                for argument in &method_call_expression.arguments {
                    arguments.push((self.interpret_expression(argument)?, argument.span));
                }

                methods::call_method(
                    self.store.get(&identifier.value),
                    &identifier.value,
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

    fn interpret_array(&self, array: &Vec<Expression>) -> Result<Literal, RuntimeError> {
        let mut result = Vec::new();
        for expression in array {
            result.push(self.interpret_expression(expression)?)
        }

        Ok(Literal::Vec(result))
    }

    fn interpret_if_statement(
        &mut self,
        statement: &IfStatement,
        io: &mut Io,
    ) -> Result<(), RuntimeError> {
        let condition = self.interpret_expression(&statement.predicate)?;

        if self.is_truthy(&condition) {
            self.store.up_scope();
            statement
                .then_statements
                .iter()
                .try_for_each(|stmt| self.interpret_statement(stmt, io))?;
            self.store.dn_scope();
        } else {
            self.store.up_scope();
            statement
                .else_statements
                .iter()
                .try_for_each(|stmt| self.interpret_statement(stmt, io))?;
            self.store.dn_scope();
        }

        Ok(())
    }

    fn interpret_while_statement(
        &mut self,
        statement: &WhileStatement,
        io: &mut Io,
    ) -> Result<(), RuntimeError> {
        let mut condition = self.interpret_expression(&statement.predicate)?;

        self.store.up_scope();
        while self.is_truthy(&condition) {
            statement
                .then_statements
                .iter()
                .try_for_each(|stmt| self.interpret_statement(stmt, io))?;
            condition = self.interpret_expression(&statement.predicate)?;
            self.store.clean_scope();
        }
        self.store.dn_scope();

        Ok(())
    }

    fn is_truthy(&self, condition: &Literal) -> bool {
        match condition {
            Literal::Boolean(b) => *b,
            Literal::Char(c) => *c != '\0',
            Literal::Float(f) => *f != 0f64,
            Literal::Integer(i) => *i != 0,
            Literal::String(s) => !s.is_empty(),
            Literal::Vec(v) => !v.is_empty(),
        }
    }

    fn check_numeric(&self, store_value: &Literal, expr: &Expression) -> Result<(), RuntimeError> {
        if [PolderType::Integer, PolderType::Float].contains(&(store_value).into()) {
            Ok(())
        } else {
            Err(runtime_error!(
                expr.span,
                RuntimeErrorKind::CannotCalculateNonNumeric,
                format!(""),
                format!("Waarde '{}' is een {}", store_value, store_value.get_type())
            ))
        }
    }

    fn interpret_foreach_statement(
        &mut self,
        foreach_statement: &ForeachStatement,
        io: &mut Io<'_>,
    ) -> Result<(), RuntimeError> {
        let variable = self.store.get(foreach_statement.list_name.as_str()).clone();

        match variable.type_ {
            PolderType::Array(_) => {
                let store_value = match variable.get_value() {
                    Some(value) => Ok(value),
                    None => Err(runtime_error!(
                        foreach_statement.span,
                        RuntimeErrorKind::VariableHasNoValue,
                        format!(
                            "Kan geen 'voor elke' lus uitvoeren voor een ongeïnitialiseerde variabele"
                        )
                    )),
                }?;

                match store_value {
                    Literal::Vec(elements) => {
                        for (index, element) in elements.iter().enumerate() {
                            self.store.up_scope();
                            self.store.declare(
                                &foreach_statement.element_binding,
                                Variable {
                                    type_: element.into(),
                                    value: Some(element.clone()),
                                },
                            );

                            if foreach_statement.index_binding.is_some() {
                                self.store.declare(
                                    foreach_statement.index_binding.as_ref().unwrap(),
                                    Variable {
                                        type_: PolderType::Integer,
                                        value: Some(Literal::Integer(index as i64)),
                                    },
                                );
                            }

                            foreach_statement
                                .statements
                                .iter()
                                .try_for_each(|stmt| self.interpret_statement(stmt, io))?;

                            self.store.dn_scope();
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
