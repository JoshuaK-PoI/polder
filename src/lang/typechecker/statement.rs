use log::debug;

use crate::lang::{
    compiler::{
        CompilerError, CompilerResult, ErrorCode,
        parser::statement::{
            IntrinsicStatement, Statement, VariableAssignmentStatement,
            VariableDeclarationStatement,
        },
    },
    typechecker::{Typecheck, Typechecker},
};

impl Typecheck for Statement {
    type Value = ();

    fn typecheck(&self, typechecker: &mut Typechecker) -> CompilerResult<Self::Value> {
        debug!("{:?}", self);
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
    fn intrinsic(
        &self,
        intrinsic: &IntrinsicStatement,
        typechecker: &mut Typechecker,
    ) -> CompilerResult<()> {
        for expression in &intrinsic.args {
            expression.typecheck(typechecker)?;
        }

        Ok(())
    }

    fn variable_declaration(
        &self,
        var_decl: &VariableDeclarationStatement,
        typechecker: &mut Typechecker,
    ) -> CompilerResult<()> {
        let declared_type = match &var_decl.initializer {
            Some(initializer) => initializer.typecheck(typechecker)?,
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
            false,
        )?;

        Ok(())
    }

    fn variable_assignment(
        &self,
        var_assn: &VariableAssignmentStatement,
        typechecker: &mut Typechecker,
    ) -> CompilerResult<()> {
        let assn_type = var_assn.expression.typecheck(typechecker)?;

        if let Some(var) = typechecker.find_identifier(&var_assn.identifier.value) {
            if var.is_const {
                return Err(CompilerError::new(
                    ErrorCode::ConstVariable,
                    format!("{} kan niet worden aangepast", &var_assn.identifier.value),
                    var_assn.expression.span,
                ));
            }

            typechecker
                .typecheck_against_var(var, assn_type, var_assn.expression.span)
                .map_err(|err| {
                    err.with_info(
                        var.loc,
                        format!(
                            "{} is hier gedefinieerd als {}",
                            var_assn.identifier.value, var.type_
                        ),
                    )
                })
        } else {
            Err(CompilerError::new(
                ErrorCode::UndefinedVariable,
                format!("{} bestaat niet", var_assn.identifier.value),
                var_assn.identifier.span,
            ))
        }
    }
}
