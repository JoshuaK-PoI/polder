use crate::lang::compiler::{
    CompilerResult,
    iter::unshift::UnshiftExpect,
    lexer::token::{ARITH_EXPR, ARTICLES, TokenStream, TokenType},
    parser::{
        expression::{BinaryExpression, Expression, ExpressionType::Binary},
        statement::BinaryOp,
    },
};

pub(super) fn binary(tokens: &mut TokenStream) -> CompilerResult<Expression> {
    let article = tokens.unshift_expect_any(ARTICLES)?;
    let operator = tokens.unshift_expect_any(ARITH_EXPR)?;

    tokens.unshift_expect(TokenType::KWVan)?;

    let lhs = Expression::parse(tokens)?;
    tokens.unshift_expect(TokenType::KWEn)?;
    let rhs = Expression::parse(tokens)?;
    let end = rhs.span.end;

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
        span: article.span.start..end,
    })
}
