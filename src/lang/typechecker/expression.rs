use crate::lang::{
    compiler::{
        CompilerError, CompilerResult, ErrorCode,
        lexer::token::Token,
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
            ExpressionType::Literal(literal) => self.literal(literal),
            ExpressionType::Identifier(identifier) => self.identifier(identifier, typechecker),
            ExpressionType::Type(type_) => Ok(type_.clone()),
            expr => todo!("Expression visited {:?}", expr),
        }
    }
}

impl Expression {
    fn literal(&self, literal: &Literal) -> CompilerResult<PolderType> {
        Ok(literal.into())
    }

    fn identifier(
        &self,
        identifier: &Token,
        typechecker: &mut Typechecker,
    ) -> CompilerResult<PolderType> {
        if let Some(var) = typechecker.find_identifier(&identifier.value) {
            if !var.initialized {
                return Err(CompilerError::new(
                    ErrorCode::UninitializedVariable,
                    format!("{} heeft nog geen waarde", identifier.value),
                    identifier.span,
                ));
            }

            Ok(var.type_.clone())
        } else {
            Err(CompilerError::new(
                ErrorCode::UndefinedVariable,
                format!("{} bestaat niet", identifier.value),
                self.span,
            ))
        }
    }
}
