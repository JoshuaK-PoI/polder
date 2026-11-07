use crate::{
    compiler_error,
    lang::{
        compiler::{
            CompilerError, CompilerResult, ErrorCode,
            parser::{
                statement::{Statement, VariableAssignmentStatement, VariableDeclarationStatement},
                types::PolderType,
            },
        },
        typechecker::{Typecheck, Typechecker},
    },
};

impl Typecheck for Statement {
    type Value = ();

    fn typecheck(&self, typechecker: &mut Typechecker) -> CompilerResult<Self::Value> {
        match self {
            Statement::Intrinsic(intrinsic) => self.intrinsic(intrinsic, typechecker),
            Statement::VariableDeclaration(var_decl) => {
                self.variable_declaration(var_decl, typechecker)
            }
            Statement::VariableAssignment(var_assn) => {
                self.variable_assignment(var_assn, typechecker)
            }
            stmt => todo!("Statement {:?}", stmt),
        }
    }
}

impl Statement {
    fn variable_declaration(
        &self,
        var_decl: &VariableDeclarationStatement,
        typechecker: &mut Typechecker,
    ) -> CompilerResult<()> {
        let declared_type = match &var_decl.initializer {
            Some(initializer) => {
                let initialized_type = initializer.typecheck(typechecker)?;
                match &var_decl.type_ {
                    //
                    // TODO: This is a mess now. Checking array types should probably be factored out somehow;
                    //       maybe it is possible to make an array type compare on PolderType itself
                    //       but that should also gracefully handle None inner types.
                    Some(type_) => match type_ {
                        PolderType::Array(inner_type) => match initialized_type {
                            PolderType::Array(ref inner_initialized_type) => {
                                if inner_type.is_some()
                                    && inner_type.as_ref().unwrap()
                                        != inner_initialized_type.as_ref().unwrap()
                                {
                                    return compiler_error!(
                                        IncompatibleTypes @ var_decl.identifier.span =>
                                        format!(
                                            "Het type van {} is niet compatibel met het type van de initiele waarde.",
                                            var_decl.identifier.value
                                        );
                                        span_message: format!("`{}` is een {}", var_decl.identifier.value, type_);
                                        @ initializer.span => format!("Het resultaat van deze expressie is een {}", initialized_type)
                                    );
                                } else {
                                    initialized_type.clone()
                                }
                            }
                            _ => {
                                return compiler_error!(
                                    IncompatibleTypes @ var_decl.identifier.span =>
                                    format!(
                                        "Het type van {} is niet compatibel met het type van de initiele waarde.",
                                        var_decl.identifier.value
                                    );
                                    span_message: format!("`{}` is een {}", var_decl.identifier.value, type_);
                                    @ initializer.span => format!("Het resultaat van deze expressie is een {}", initialized_type)
                                );
                            }
                        },
                        type_ => {
                            if type_ == &initialized_type {
                                Ok(type_.clone())
                            } else {
                                compiler_error!(
                                    IncompatibleTypes @ var_decl.identifier.span =>
                                    format!(
                                        "Het type van {} is niet compatibel met het type van de initiele waarde.",
                                        var_decl.identifier.value
                                    );
                                    span_message: format!("`{}` is een {}", var_decl.identifier.value, type_);
                                    @ initializer.span => format!("Het resultaat van deze expressie is een {}", initialized_type)
                                )
                            }
                        }?
                    },
                    None => initialized_type,
                }
            }
            None => var_decl
                .type_
                .as_ref()
                .expect(
                    format!(
                        "Variable {} has no type and no initializer; \
                        this should have been caught in the variable declaration statement parser",
                        var_decl.identifier.value
                    )
                    .as_str(),
                )
                .clone(),
        };

        typechecker.declare(
            var_decl.identifier.value.clone(),
            var_decl.identifier.span,
            declared_type,
            var_decl.is_const,
            var_decl.initializer.is_some(),
        )?;

        Ok(())
    }

    fn variable_assignment(
        &self,
        var_assn: &VariableAssignmentStatement,
        typechecker: &mut Typechecker,
    ) -> CompilerResult<()> {
        let assn_type = var_assn.expression.typecheck(typechecker)?;

        if let Some(var) = typechecker.find_identifier_mut(&var_assn.identifier.value) {
            if var.immutable {
                return compiler_error!(
                    ConstVariable @ var_assn.expression.span =>
                    format!("Kan constante `{}` geen nieuwe waarde toewijzen.", &var_assn.identifier.value)
                );
            }

            // @TODO: When we support functions and closures, variables may be declared after their assignment.
            // This should be handled here; the `used_at` struct field may help with this - we can
            // create a two-way check where the assigned expression type sets the initial variable type,
            // and if that doesn't match the later declaration we can use that field to direct the error
            // to the appropriate assignment
            if var.type_ != assn_type {
                return compiler_error!(
                    MismatchedType @ var_assn.expression.span =>
                    format!("Een {} kan niet toegewezen worden aan {} {} `{}`", assn_type, var.type_.article(), var.type_, var_assn.identifier.value);
                    @ var.declared_at => format!("{} is hier gedefinieerd als {}", var_assn.identifier.value, var.type_)
                );
            }

            var.initialized = true;
            var.modified = true;
            var.used_at.push(var_assn.identifier.span);

            Ok(())
        } else {
            // TODO: Like the comment above, when declaration and assignment are out of order,
            // we should handle that here
            compiler_error!(
                UndefinedVariable @ var_assn.identifier.span =>
                format!("{} bestaat niet", var_assn.identifier.value)
            )
        }
    }
}
