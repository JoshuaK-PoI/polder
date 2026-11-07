use serde::Serialize;

use crate::{
    compiler_error,
    lang::compiler::{
        CompilerError, CompilerResult, ErrorCode, Span,
        lexer::token::{KEYWORDS, LITERALS, Token, TokenType},
        parser::{
            Parser, ParserMode,
            expression::{
                BinaryExpression, Expression, ExpressionType, MethodCallExpression,
                UnaryExpression, literal::literal,
            },
            intrinsic::Intrinsics,
            types::PolderType,
        },
    },
};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) enum Statement {
    VariableDeclaration(VariableDeclarationStatement),
    VariableAssignment(VariableAssignmentStatement),
    Intrinsic(IntrinsicStatement),
    If(IfStatement),
    While(WhileStatement),
    Foreach(ForeachStatement),
    Comment,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct IntrinsicStatement {
    pub(crate) intrinsic: Intrinsics,
    pub(crate) args: Vec<Expression>,
    pub(crate) span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct VariableDeclarationStatement {
    pub(crate) identifier: Token,
    pub(crate) type_: Option<PolderType>,
    pub(crate) initializer: Option<Expression>,
    pub(crate) is_const: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct VariableAssignmentStatement {
    pub(crate) identifier: Token,
    pub(crate) expression: Expression,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct IfStatement {
    pub(crate) predicate: Expression,
    pub(crate) then_statements: Vec<Statement>,
    pub(crate) else_statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct WhileStatement {
    pub(crate) predicate: Expression,
    pub(crate) then_statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct ForeachStatement {
    pub(crate) list_name: String,
    pub(crate) element_binding: String,
    pub(crate) index_binding: Option<String>,
    pub(crate) statements: Vec<Statement>,
    pub(crate) span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Less,
    Greater,
    Equal,
}

impl Into<BinaryOp> for TokenType {
    fn into(self) -> BinaryOp {
        match self {
            TokenType::KWGroter => BinaryOp::Greater,
            TokenType::KWKleiner => BinaryOp::Less,
            TokenType::KWGelijk => BinaryOp::Equal,
            _ => panic!("Binary op not implemented for token type {}", self),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) enum UnaryOp {
    Increment,
    Decrement,
    Negation,
}

impl Parser {
    pub(crate) fn stmt(&mut self) -> CompilerResult<Statement> {
        if self.peek().is_none() {
            return compiler_error!(
                NoTokensLeft @ self.get_nth(0).span =>
                format!("Verwachtte hier het begin van een zin, maar vond het einde van het bestand.")
            );
        }

        let stmt = match self.peek().unwrap().type_ {
            TokenType::KWDruk => self.intrinsic(Intrinsics::Print),
            TokenType::KWSchrijf => self.intrinsic(Intrinsics::Write),
            TokenType::KWMaak => self.variable_assignment(),
            TokenType::KWLaat => self.variable_declaration(),
            TokenType::KWLees => self.intrinsic(Intrinsics::Readline),
            TokenType::KWTel => self.arith_add(),
            TokenType::KWTrek => self.arith_subtract(),
            TokenType::KWVermenigvuldig => self.arith_multiply(),
            TokenType::KWDeel => self.arith_divide(),
            TokenType::KWVerhoog => self.arith_increment(),
            TokenType::KWVerlaag => self.arith_decrement(),
            TokenType::KWAls => self.r#if(),
            TokenType::KWZolang => self.r#while(),
            TokenType::KWVoeg => self.method(),
            TokenType::KWVoor => self.foreach(),
            TokenType::Comment => {
                self.expect(TokenType::Comment)?;
                Ok(Statement::Comment)
            }
            TokenType::KWKlaar => {
                let start = self.expect(TokenType::KWKlaar)?.span.0;
                self.expect(TokenType::KWIs)?;
                let end = self.expect(TokenType::KWKees)?;

                compiler_error!(
                    InvalidStatement @ (start, end.span.1) => format!("Kees was nog niet eens begonnen");
                    span_message: format!("Er is hier geen blok om af te sluiten.")
                )
            }
            TokenType::KWZo | TokenType::KWAnders => compiler_error!(
                InvalidStatement @ self.peek().unwrap().span =>
                format!(
                    "'{}' hoort bij een 'Als'",
                    if self.peek().unwrap().type_ == TokenType::KWZo {
                        format!("Zo niet")
                    } else {
                        format!("Anders")
                    }
                );
                span_message: format!(
                "Er staat geen 'Als' vóór '{}'",
                if self.peek().unwrap().type_ == TokenType::KWZo {
                    format!("Zo niet")
                } else {
                    format!("Anders")
                }
            )),
            _ => compiler_error!(
                InvalidStatement @ self.peek().unwrap().span =>
                format!("Ik weet niet wat ik met `{}` moet; dit is geen geldig begin van een zin.", self.peek().unwrap().value)
            ),
        }?;

        if self.mode != ParserMode::InThenStatement && stmt != Statement::Comment {
            self.expect(TokenType::FullStop).map_err(|_| {
                CompilerError::new(
                    ErrorCode::StatementNotTerminated,
                    format!("Een zin eindigt met een punt."),
                    self.get_nth(0).span,
                )
                .with_span_message(format!("Deze zin is niet beëindigd."))
            })?;
        }

        Ok(stmt)
    }

    pub fn intrinsic(&mut self, intrinsic_name: Intrinsics) -> CompilerResult<Statement> {
        let start = self.expect(intrinsic_name.expected_start_token())?.span.0;
        if self.mode == ParserMode::InThenStatement {
            self.expect(TokenType::KWDan)?;
        }
        let arguments = vec![self.expression()?];
        let end = self.expect(intrinsic_name.expected_end_token())?.span.1;

        Ok(Statement::Intrinsic(IntrinsicStatement {
            intrinsic: intrinsic_name,
            args: arguments,
            span: (start, end),
        }))
    }

    pub fn variable_assignment(&mut self) -> CompilerResult<Statement> {
        self.expect(TokenType::KWMaak)?;
        if self.mode == ParserMode::InThenStatement {
            self.expect(TokenType::KWDan)?;
        }

        self.expect(TokenType::KWVan)?;
        let identifier = self.expect(TokenType::Identifier)?;
        let expression = self.expression()?;

        Ok(Statement::VariableAssignment(VariableAssignmentStatement {
            identifier,
            expression,
        }))
    }

    pub fn variable_declaration(&mut self) -> CompilerResult<Statement> {
        if self.mode == ParserMode::InThenStatement {
            return compiler_error!(
                InvalidThenStatement @ self.next().span =>
                format!("Hier kan geen variabele gemaakt worden");
                help: format!(
                    "Een variabele die je hier zou maken, is niet toegankelijk buiten de 'Als'- zin."
                )
            );
        }

        self.expect(TokenType::KWLaat)?;
        let identifier = self.get_identifier()?;
        let is_const = self.next_is_then_unshift(TokenType::KWConst);

        if self.next_is(TokenType::KWZijn) {
            return compiler_error!(
                CannotInferVariableType @ identifier.span =>
                format!("Wat is `{}` dan?", identifier.value);
                span_message: format!("Je moet een type of een waarde toewijzen aan `{}`.",identifier.value)
            );
        }

        let type_ = self.try_get_type()?;
        let initializer = if !self.next_is(TokenType::KWZijn) {
            Some(self.expression()?)
        } else {
            None
        };

        if is_const && initializer.is_none() {
            return compiler_error!(
                ConstDeclarationMustHaveValue @ identifier.span =>
                format!("Je hebt geen waarde toegewezen aan `{}`, wat verplicht is als je een constante maakt.", identifier.value)
            );
        }

        if type_.is_none() && initializer.is_none() {
            unreachable!(
                "In variable declaration for {}: It should not be possible to have no initializer and no type, but this was the case.",
                identifier.value
            );
        }

        self.expect(TokenType::KWZijn)?;

        Ok(Statement::VariableDeclaration(
            VariableDeclarationStatement {
                identifier,
                type_,
                initializer,
                is_const,
            },
        ))
    }

    pub(super) fn get_identifier(&mut self) -> CompilerResult<Token> {
        match self.peek() {
            Some(token) if token.type_ == TokenType::Identifier => Ok(self.next()),
            Some(token) if KEYWORDS.contains_key(&token.value) => compiler_error!(
                InvalidIdentifier @ token.span =>
                format!(
                    "'{}' is een gereserveerd woord en is niet bruikbaar als naam voor een variabele.",
                    token.value
                )
            ),
            Some(token) => compiler_error!(
                InvalidIdentifier @ token.span =>
                format!(
                    "Een variabelenaam moet een woord zijn, '{}' is hier niet geldig.",
                    token.value,
                )
            ),
            None => compiler_error!(
                NoTokensLeft @ self.get_nth(0).span =>
                format!("Geef een naam voor de variabele op.")
            ),
        }
    }

    fn arith_add(&mut self) -> CompilerResult<Statement> {
        self.expect(TokenType::KWTel)?;
        if self.mode == ParserMode::InThenStatement {
            self.expect(TokenType::KWDan)?;
        }

        let lhs = self.expression()?;
        self.expect(TokenType::KWBij)?;
        let rhs = self.expect(TokenType::Identifier)?.clone();

        self.expect(TokenType::KWOp)?;
        Ok(self.variable_assignment_binary(BinaryOp::Add, rhs, lhs))
    }

    fn arith_subtract(&mut self) -> CompilerResult<Statement> {
        self.expect(TokenType::KWTrek)?;
        if self.mode == ParserMode::InThenStatement {
            self.expect(TokenType::KWDan)?;
        }

        let lhs = self.expression()?;
        self.expect(TokenType::KWVan)?;
        let rhs = self.expect(TokenType::Identifier)?.clone();
        self.expect(TokenType::KWAf)?;

        Ok(self.variable_assignment_binary(BinaryOp::Subtract, rhs, lhs))
    }

    fn arith_multiply(&mut self) ->CompilerResult<Statement> {
        self.expect(TokenType::KWVermenigvuldig)?;
        if self.mode == ParserMode::InThenStatement {
            self.expect(TokenType::KWDan)?;
        }

        let lhs = self.expect(TokenType::Identifier)?.clone();
        self.expect(TokenType::KWMet)?;
        let rhs = self.expression()?;

        Ok(self.variable_assignment_binary(BinaryOp::Multiply, lhs, rhs))
    }

    fn arith_divide(&mut self) -> CompilerResult<Statement> {
        self.expect(TokenType::KWDeel)?;
        if self.mode == ParserMode::InThenStatement {
            self.expect(TokenType::KWDan)?;
        }

        let lhs = self.expect(TokenType::Identifier)?.clone();
        self.expect(TokenType::KWDoor)?;
        let rhs = self.expression()?;

        Ok(self.variable_assignment_binary(BinaryOp::Divide, lhs, rhs))
    }

    fn arith_increment(&mut self) -> CompilerResult<Statement> {
        let start = self.expect(TokenType::KWVerhoog)?;
        if self.mode == ParserMode::InThenStatement {
            self.expect(TokenType::KWDan)?;
        }

        let ident = self.expect(TokenType::Identifier)?.clone();
        let span = ident.span;
        let ident_expr = Expression::make(ExpressionType::Identifier(ident.clone()), span);
        Ok(self.variable_assignment_unary(start, UnaryOp::Increment, ident, ident_expr))
    }

    fn arith_decrement(&mut self) -> CompilerResult<Statement> {
        let start = self.expect(TokenType::KWVerlaag)?;
        if self.mode == ParserMode::InThenStatement {
            self.expect(TokenType::KWDan)?;
        }

        let ident = self.expect(TokenType::Identifier)?.clone();
        let span = ident.span;
        let ident_expr = Expression::make(ExpressionType::Identifier(ident.clone()), span);

        Ok(self.variable_assignment_unary(start, UnaryOp::Decrement, ident, ident_expr))
    }

    fn r#if(&mut self) -> CompilerResult<Statement> {
        if self.mode == ParserMode::InThenStatement {
            return compiler_error!(
                InvalidThenStatement @ self.next().span =>
                format!("Hier kan niet nogmaals een 'Als' voorkomen")
            );
        }

        let start = self.expect(TokenType::KWAls)?.clone();

        // @TODO: For now, we only support equality/gt/lt expressions
        // When we support AND / OR, we may get more than 1 expression - in that case we will
        // need to loop until we hit the comma / colon.
        let mut check_expression = self.expression()?;

        if self.next_is_any_of(LITERALS) {
            check_expression = self.create_shorthand_if_statement(check_expression)?;
        }

        let expect_block = self.next_is_then_unshift(TokenType::Colon);
        if !expect_block {
            self.expect(TokenType::Comma).map_err(|err| {
                err.with_span_message(format!(
                    "Verwachtte een literal, komma of dubbele punt hier."
                ))
            })?;
        }

        let mut then_statements = Vec::new();

        if expect_block {
            while self.peek().is_some_and(|token| {
                ![TokenType::KWZo, TokenType::KWAnders, TokenType::KWKlaar].contains(&token.type_)
            }) {
                then_statements.push(self.stmt()?);
            }
        } else {
            self.mode = ParserMode::InThenStatement;
            then_statements.push(self.stmt()?);
            self.mode = ParserMode::Normal;
        }

        self.check_block_empty(&then_statements, &start, &check_expression)?;

        let else_statements =
            self.parse_else_statements(expect_block, &start, &check_expression)?;

        Ok(Statement::If(IfStatement {
            predicate: check_expression,
            then_statements,
            else_statements,
        }))
    }

    fn create_shorthand_if_statement(
        &mut self,
        check_expression: Expression,
    ) -> CompilerResult<Expression> {
        let lit = self.expect_any_of(LITERALS)?;
        self.expect(TokenType::KWIs)?;

        let span = (check_expression.span.0, lit.span.1);

        Ok(Expression {
            expression: ExpressionType::Binary(BinaryExpression {
                op: BinaryOp::Equal,
                lhs: Box::new(check_expression),
                rhs: Box::new(literal(lit)?),
            }),
            span,
        })
    }

    fn parse_else_statements(
        &mut self,
        expect_block: bool,
        start: &Token,
        check_expression: &Expression,
    ) -> CompilerResult<Vec<Statement>> {
        let mut else_statements = Vec::new();

        if expect_block {
            if self.next_is(TokenType::KWZo) && self.next_nth_is_any_of(2, &[TokenType::KWNiet]) {
                return compiler_error!(
                    InvalidStatement @ (self.next().span.0, self.get_nth(1).span.1) =>
                    format!("'Zo niet' hoort bij een korte 'Als'-zin");
                    span_message: format!("Gebruik hier 'Anders:' om het blok voort te zetten.")
                );
            }

            if self.next_is_then_unshift(TokenType::KWKlaar) {
                self.expect(TokenType::KWIs)?;
                self.expect(TokenType::KWKees)?;
            } else if self.next_is_then_unshift(TokenType::KWAnders) {
                if self.next_is_then_unshift(TokenType::Comma) {
                    else_statements.push(self.r#if()?);
                } else {
                    self.expect(TokenType::Colon)?;

                    while self
                        .peek()
                        .is_some_and(|token| token.type_ != TokenType::KWKlaar)
                    {
                        else_statements.push(self.stmt()?);
                        if self.next_is(TokenType::KWAnders) {
                            return compiler_error!(
                                InvalidStatement @ self.next().span =>
                                format!("Dit als-blok heeft al een 'Anders'.")
                            );
                        }
                    }

                    self.expect(TokenType::KWKlaar)?;
                    self.expect(TokenType::KWIs)?;
                    self.expect(TokenType::KWKees)?;
                }
            } else {
                return compiler_error!(
                    InvalidStatement @ (start.span.0, check_expression.span.1) =>
                    format!("Kees is nog niet klaar");
                    span_message: format!("Dit blok wordt nergens afgesloten met 'Klaar is Kees.'")
                );
            }
        } else {
            if self.next_is(TokenType::FullStop)
                && self.next_nth_is_any_of(2, &[TokenType::KWAnders])
            {
                return compiler_error!(
                    InvalidStatement @ self.get_nth(2).span =>
                    format!("Anders wát?");
                    span_message: format!("'Anders' hoort bij een blok. Gebruik 'Zo niet,' na een korte 'Als'-zin.")
                );
            }

            if self.next_is(TokenType::FullStop) && self.next_nth_is_any_of(2, &[TokenType::KWZo]) {
                self.expect(TokenType::FullStop)?;
                self.expect(TokenType::KWZo)?;
                self.expect(TokenType::KWNiet)?;
                self.expect(TokenType::Comma)?;

                if self.next_is(TokenType::KWAls) {
                    else_statements.push(self.r#if()?);
                } else {
                    self.mode = ParserMode::InThenStatement;
                    else_statements.push(self.stmt()?);
                    self.mode = ParserMode::Normal;
                }
            }
        }

        Ok(else_statements)
    }

    fn r#while(&mut self) -> CompilerResult<Statement> {
        if self.mode == ParserMode::InThenStatement {
            return compiler_error!(
                InvalidThenStatement @ self.next().span =>
                format!("Hier kan geen 'Zolang' voorkomen");
                span_message: format!("Een 'Zolang' kan niet het resultaat van een 'Als' zijn.")
            );
        }

        let start = self.expect(TokenType::KWZolang)?;

        let predicate = self.expression()?;

        self.expect(TokenType::Colon)?;

        let mut then_statements = Vec::new();

        while self
            .peek()
            .is_some_and(|token| token.type_ != TokenType::KWKlaar)
        {
            then_statements.push(self.stmt()?);
        }

        self.check_block_empty(&then_statements, &start, &predicate)?;

        if self.peek().is_none() {
            return compiler_error!(
                InvalidStatement @ (start.span.0, predicate.span.1) =>
                format!("Kees is nog niet klaar");
                span_message: format!("Dit blok wordt nergens afgesloten met 'Klaar is Kees.'")
            );
        }
        self.expect(TokenType::KWKlaar)?;
        self.expect(TokenType::KWIs)?;
        self.expect(TokenType::KWKees)?;

        Ok(Statement::While(WhileStatement {
            predicate,
            then_statements,
        }))
    }

    fn foreach(&mut self) -> CompilerResult<Statement> {
        let start = self.expect(TokenType::KWVoor)?.span.0;
        self.expect(TokenType::KWElke)?;

        let element_binding = self.expect(TokenType::Identifier)?.value;
        let index_binding = if self.next_is_then_unshift(TokenType::KWEn) {
            Some(self.expect(TokenType::Identifier)?.value)
        } else {
            None
        };

        self.expect(TokenType::KWIn)?;
        let list = self.expect(TokenType::Identifier)?;
        let list_name = list.value;
        let end = list.span.1;

        self.expect(TokenType::Colon)?;

        let mut statements = Vec::new();
        while !self.next_is(TokenType::KWKlaar) {
            statements.push(self.stmt()?);
        }

        self.expect(TokenType::KWKlaar)?;
        self.expect(TokenType::KWIs)?;
        self.expect(TokenType::KWKees)?;

        Ok(Statement::Foreach(ForeachStatement {
            list_name,
            element_binding,
            index_binding,
            statements,
            span: (start, end),
        }))
    }

    fn check_block_empty(
        &self,
        statements: &Vec<Statement>,
        start: &Token,
        check_expression: &Expression,
    ) -> CompilerResult<()> {
        if statements.is_empty() {
            compiler_error!(
                InvalidStatement @ (start.span.0, check_expression.span.1) =>
                format!("Er valt niks te doen");
                span_message: format!("Dit blok is leeg.")
            )
        } else {
            Ok(())
        }
    }

    fn variable_assignment_unary(
        &self,
        start: Token,
        op: UnaryOp,
        target: Token,
        expr: Expression,
    ) -> Statement {
        let span = (start.span.0, expr.span.1);
        Statement::VariableAssignment(VariableAssignmentStatement {
            identifier: target,
            expression: Expression {
                expression: ExpressionType::Unary(UnaryExpression {
                    op,
                    expr: Box::new(expr),
                }),
                span,
            },
        })
    }

    fn variable_assignment_binary(
        &self,
        op: BinaryOp,
        target: Token,
        expr: Expression,
    ) -> Statement {
        let expr_span = expr.span;
        let lhs_span = target.span;
        Statement::VariableAssignment(VariableAssignmentStatement {
            identifier: target.clone(),
            expression: Expression {
                expression: ExpressionType::Binary(BinaryExpression {
                    op,
                    lhs: Box::new(Expression {
                        expression: ExpressionType::Identifier(target),
                        span: lhs_span,
                    }),
                    rhs: Box::new(expr),
                }),
                span: expr_span,
            },
        })
    }

    fn method(&mut self) -> CompilerResult<Statement> {
        match self.peek().unwrap().type_ {
            TokenType::KWVoeg => {
                let start = self.expect(TokenType::KWVoeg)?.span.0;
                if self.mode == ParserMode::InThenStatement {
                    self.expect(TokenType::KWDan)?;
                }

                let mut arguments: Vec<Expression> = Vec::new();
                let mut multipart = false;

                loop {
                    arguments.push(self.expression()?);

                    if self.next_is_then_unshift(TokenType::Comma) {
                        multipart = true;
                        continue;
                    }

                    if self.next_is(TokenType::KWAan) {
                        if multipart {
                            return compiler_error!(
                                InvalidExpression @ self.peek().unwrap().span =>
                                format!(
                                    "Verwachtte een expressie, komma, of 'en' hier, maar vond '{}'",
                                    self.peek().unwrap().type_
                                );
                                span_message: format!("Mogelijk ben je 'en' vergeten vóór het laatste element.")
                            );
                        }
                        break;
                    }

                    if self.next_is_then_unshift(TokenType::KWEn) {
                        arguments.push(self.expression()?);
                        break;
                    }

                    return compiler_error!(
                        InvalidExpression @ self.peek().unwrap().span =>
                        format!(
                            "Verwachtte een expressie, komma, of 'en' hier, maar vond '{}'",
                            self.peek().unwrap().type_
                        );
                        span_message: format!("Mogelijk ben je 'en' vergeten vóór het laatste element.")
                    );
                }

                self.expect(TokenType::KWAan)?;
                let identifier = self.expect(TokenType::Identifier)?;
                let identifier_span = identifier.span;
                let end = self.expect(TokenType::KWToe)?.span.1;

                Ok(Statement::VariableAssignment(VariableAssignmentStatement {
                    identifier: identifier.clone(),
                    expression: Expression {
                        span: (start, end),
                        expression: ExpressionType::MethodCall(MethodCallExpression {
                            method_name: "Voeg toe".to_string(),
                            arguments,
                            class: Box::new(Expression {
                                expression: ExpressionType::Identifier(identifier),
                                span: identifier_span,
                            }),
                        }),
                    },
                }))
            }
            _ => unimplemented!("Method statement for {:?}", self.peek().unwrap()),
        }
    }
}
