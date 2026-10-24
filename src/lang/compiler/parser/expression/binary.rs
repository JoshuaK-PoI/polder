use crate::lang::compiler::{
    CompilerError, CompilerResult, ErrorCode,
    lexer::token::{ARITH_EXPR, ARTICLES, TokenType},
    parser::{
        Parser,
        expression::{BinaryExpression, Expression, ExpressionType::Binary},
        statement::BinaryOp,
        types::is_correct_article,
    },
};

impl Parser {
    pub(super) fn binary_prefix(&mut self) -> CompilerResult<Expression> {
        let article = self.expect_any_of(ARTICLES)?;
        let operator = self.expect_any_of(ARITH_EXPR)?;

        if !is_correct_article(&article, &operator) {
            return Err(CompilerError {
                kind: ErrorCode::WrongArticleForType,
                message: format!("Lekker genederlandst"),
                span_message: format!("Dat is geen correct Nederlands"),
                span: article.span.clone(),
            });
        }

        self.expect(TokenType::KWVan)?;

        let lhs = self.binary()?;
        self.expect(TokenType::KWEn)?;
        let rhs = self.binary()?;
        let end = rhs.span.1;

        Ok(Expression {
            expression: Binary(BinaryExpression {
                op: match operator.type_ {
                    TokenType::KWSom => BinaryOp::Add,
                    TokenType::KWVerschil => BinaryOp::Subtract,
                    TokenType::KWProduct => BinaryOp::Multiply,
                    TokenType::KWQuotient => BinaryOp::Divide,
                    _ => unreachable!(),
                },
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
            span: (article.span.0, end),
        })
    }
}
