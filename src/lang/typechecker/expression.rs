use crate::{
    compiler_error,
    lang::{
        compiler::{
            CompilerError, CompilerResult, ErrorCode,
            lexer::token::Token,
            parser::{
                expression::{
                    BinaryExpression, Expression, ExpressionType, UnaryExpression, literal::Literal,
                },
                types::PolderType,
            },
        },
        typechecker::{Typecheck, Typechecker},
    },
};

impl Typecheck for Expression {
    type Value = PolderType;
    fn typecheck(&self, typechecker: &mut Typechecker) -> CompilerResult<Self::Value> {
        match &self.expression {
            ExpressionType::Literal(literal) => self.literal(literal),
            ExpressionType::Identifier(identifier) => self.identifier(identifier, typechecker),
            ExpressionType::Binary(binary) => self.binary(binary, typechecker),
            ExpressionType::Unary(unary) => self.unary(unary, typechecker),
            ExpressionType::ArrayDecl(arr) => self.array_decl(arr, typechecker),
            expr => unimplemented!("Typecheck expression {:?}", expr),
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
                return compiler_error!(
                    UninitializedVariable @ identifier.span =>
                    format!("{} heeft nog geen waarde", identifier.value)
                );
            }

            Ok(var.type_.clone())
        } else {
            compiler_error!(
                UndefinedVariable @ self.span =>
                format!("`{}` is niet gedefinieerd.", identifier.value)
            )
        }
    }

    fn unary(
        &self,
        unary: &UnaryExpression,
        typechecker: &mut Typechecker,
    ) -> CompilerResult<PolderType> {
        unary.expr.typecheck(typechecker)
    }

    fn binary(
        &self,
        binary: &BinaryExpression,
        typechecker: &mut Typechecker,
    ) -> CompilerResult<PolderType> {
        let lhs = binary.lhs.typecheck(typechecker)?;
        let rhs = binary.rhs.typecheck(typechecker)?;

        if lhs != rhs {
            return compiler_error!(
                MismatchedType @ binary.lhs.span =>
                format!("De linker- en rechterkant van deze expressie hebben verschillende types");
                span_message: format!("Deze kant resulteert in een {}", lhs);
                @ binary.rhs.span => format!("Deze kant resulteert in een {}", rhs)
            );
        }

        Ok(lhs)
    }

    fn array_decl(
        &self,
        arr: &Vec<Expression>,
        typechecker: &mut Typechecker,
    ) -> CompilerResult<PolderType> {
        let mut initialized_type = None;
        for expr in arr {
            let type_ = expr.typecheck(typechecker)?;
            match initialized_type {
                Some(ref initialized_type) => {
                    if initialized_type != &type_ {
                        return compiler_error!(MismatchedType @ expr.span =>
                            format!("Niet alle expressies in de lijst hebben hetzelfde type");
                            span_message: format!("Verwachtte een expressie die resulteert in een {}, maar deze expressie resulteert in een {}", initialized_type, type_)
                        );
                    }
                }
                None => initialized_type = Some(type_),
            }
        }

        if initialized_type.is_none() {
            panic!("After array declaration, no type was set (is the array empty?)");
        }
        Ok(PolderType::Array(Some(Box::new(initialized_type.unwrap()))))
    }
}
