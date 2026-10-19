use crate::lang::compiler::{
    CompilerError, CompilerResult, ErrorCode,
    iter::unshift::UnshiftExpect,
    lexer::token::{ARITH_EXPR, ARTICLES, TokenStream, TokenType},
    parser::{
        expression::{BinaryExpression, Expression, ExpressionType::Binary},
        statement::BinaryOp,
        types::is_correct_article,
    },
};

pub(super) fn binary(tokens: &mut TokenStream) -> CompilerResult<Expression> {
    let article = tokens.unshift_expect_any(ARTICLES)?;
    let operator = tokens.unshift_expect_any(ARITH_EXPR)?;
    if !is_correct_article(&article.type_, &operator.type_) {
        return Err(CompilerError {
            kind: ErrorCode::WrongArticleForType,
            message: format!("Lekker genederlandst"),
            span_message: format!("Dat is geen correct Nederlands"),
            span: article.span.clone(),
        });
    }
    
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
