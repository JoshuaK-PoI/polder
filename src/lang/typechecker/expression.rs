use crate::{
    compiler_error,
    lang::{
        compiler::{
            CompilerError, CompilerResult, ErrorCode,
            lexer::token::Token,
            parser::{
                expression::{
                    BinaryExpression, Expression, ExpressionType, MethodCallExpression,
                    UnaryExpression, literal::Literal,
                },
                statement::{BinaryOp, UnaryOp},
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
            ExpressionType::MethodCall(method) => self.method_call(method, typechecker),
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
                    format!("`{}` heeft nog geen waarde", identifier.value)
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
        let expr_type = unary.expr.typecheck(typechecker)?;

        match &unary.op {
            UnaryOp::Decrement | UnaryOp::Increment => {
                if ![PolderType::Integer, PolderType::Float].contains(&expr_type) {
                    return compiler_error!(
                        MismatchedType @ unary.expr.span => format!("Het resultaat van deze expressie is hier niet toegestaan");
                        span_message: format!("Deze expressie resulteert in een {}, verwachtte een {} of {}", expr_type, PolderType::Integer, PolderType::Float)
                    );
                }
            }
            UnaryOp::Negation => {
                if ![PolderType::Boolean].contains(&expr_type) {
                    return compiler_error!(
                        MismatchedType @ unary.expr.span => format!("Het resultaat van deze expressie is hier niet toegestaan");
                        span_message: format!("Deze expressie resulteert in een {}, verwachtte een {}", expr_type, PolderType::Boolean)
                    );
                }
            }
        };

        Ok(expr_type)
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

        let expected: &[PolderType] = match binary.op {
            BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::Divide
            | BinaryOp::Multiply
            | BinaryOp::Modulo
            | BinaryOp::Greater
            | BinaryOp::Less => &[PolderType::Float, PolderType::Integer],

            BinaryOp::Equal => &[
                PolderType::Boolean,
                PolderType::Char,
                PolderType::Float,
                PolderType::Integer,
                PolderType::String,
            ],
        };

        if !expected.contains(&lhs) {
            let mut expected = expected
                .into_iter()
                .map(|t| t.to_string())
                .collect::<Vec<String>>()
                .join(",");

            expected
                .rfind(",")
                .map(|i| expected.replace_range(i..(i + 1), "of"));

            return compiler_error!(
                MismatchedType @ binary.lhs.span => format!("Het resultaat van dit type is niet geldig hier");
                span_message: format!("Verwachtte een {} hier, maar vond een {}", expected, lhs)
            );
        }

        Ok(match binary.op {
            BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::Divide
            | BinaryOp::Multiply
            | BinaryOp::Modulo => lhs,
            BinaryOp::Greater | BinaryOp::Less | BinaryOp::Equal => PolderType::Boolean,
        })
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

    fn method_call(
        &self,
        method: &MethodCallExpression,
        typechecker: &mut Typechecker,
    ) -> CompilerResult<PolderType> {
        let ident = match &method.class.expression {
            ExpressionType::Identifier(ident) => Ok(ident),
            expr => compiler_error!(
                InvalidExpression @ method.class.span => format!("Verwachtte hier een identificatie, maar vond een {}.", expr)
            ),
        }?;
        
        let class_type = if let Some(var) = typechecker.find_identifier_mut(&ident.value) {
            if method.method_name == "Voeg toe".to_string() {
                var.initialized = true;
            }
            Ok(var.type_.clone())
        } else {
            compiler_error!(
                UndefinedVariable @ method.class.span => format!("`{}` bestaat niet.", ident.value)
            )
        }?;

        if method.method_name == "Voeg toe" {
            for arg in &method.arguments {
                let type_ = arg.typecheck(typechecker)?;
                let atype = match &type_ {
                    PolderType::Array(Some(inner)) => inner,
                    typ => typ
                };
                
                let ctype = match &class_type {
                    PolderType::Array(Some(inner)) => inner,
                    typ => typ
                };

                if atype != ctype {
                    return compiler_error!(
                        MismatchedType @ arg.span => format!("Kan deze expressie niet toevoegen");
                        span_message: format!("`{}` is een {}, maar deze expressie resulteert in een {}", ident.value, class_type, type_)
                    )
                }

            }
        }

        for arg in &method.arguments {
            arg.typecheck(typechecker)?;
        }

        // @TODO: Because methods are loosely defined and we don't support user functions yet,
        // there is not a lot to typecheck on them right now
        // When we implement functions we can add stricter type checking to make sure the arguments and
        // return type are correct, but for the current builtin methods this is not needed
        Ok(class_type)
    }
}
