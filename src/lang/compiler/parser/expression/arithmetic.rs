use crate::lang::compiler::{
    CompilerResult,
    iter::unshift::UnshiftExpect,
    lexer::token::{ARITH_EXPR, ARTICLES, TokenStream, TokenType},
    parser::{
        expression::{ArithmeticExpression, Expression, ExpressionType::Arithmetic},
        statement::ArithmeticOp,
    },
};

pub(super) fn arithmetic(tokens: &mut TokenStream) -> CompilerResult<Expression> {
    let article = tokens.unshift_expect_any(ARTICLES)?;
    let operator = tokens.unshift_expect_any(ARITH_EXPR)?;

    tokens.unshift_expect(TokenType::KWVan)?;

    let lhs = Expression::parse(tokens)?;
    tokens.unshift_expect(TokenType::KWEn)?;
    let rhs = Expression::parse(tokens)?;

    let end = tokens.unshift_expect(TokenType::KWZijn)?;

    Ok(Expression {
        expression: Arithmetic(ArithmeticExpression {
            op: match operator.type_ {
                TokenType::KWSom => ArithmeticOp::Add,
                _ => unreachable!(),
            },
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }),
        span: article.span.start..end.span.end,
    })
}
