use crate::lang::{
    compiler::{
        CompilerResult,
        parser::{
            expression::{Expression, ExpressionType, literal::Literal},
            types::PolderType,
        },
    },
    typechecker::{Typecheck, Typechecker},
};

impl Typecheck for Expression {
    type Value = PolderType;
    fn typecheck(&self, typechecker: &mut Typechecker) -> CompilerResult<Self::Value> {
        match &self.expression {
            ExpressionType::Literal(literal) => visit_literal(literal),
            expr => todo!("Expression visited {:?}", expr),
        }
    }
}

fn visit_literal(literal: &Literal) -> CompilerResult<PolderType> {
    Ok(literal.into())
}
