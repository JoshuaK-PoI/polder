use crate::lang::{
    compiler::{
        CompilerError, CompilerResult, ErrorCode,
        parser::{
            expression::ExpressionType,
            intrinsic::Intrinsics,
            statement::{IntrinsicStatement, Statement},
        },
    },
    typechecker::{Typecheck, Typechecker},
};

impl Statement {
    pub(super) fn intrinsic(
        &self,
        intrinsic: &IntrinsicStatement,
        typechecker: &mut Typechecker,
    ) -> CompilerResult<()> {
        for expression in &intrinsic.args {
            match expression.typecheck(typechecker) {
                Err(err)
                    if err.kind == ErrorCode::UninitializedVariable
                        && intrinsic.intrinsic == Intrinsics::Readline =>
                {
                    Ok(())
                }
                res => res.map(|_| ()),
            }?;
        }

        if intrinsic.intrinsic == Intrinsics::Readline {
            self.check_readline_intrinsic(intrinsic, typechecker)?;
        }

        Ok(())
    }

    pub fn check_readline_intrinsic(
        &self,
        intrinsic: &IntrinsicStatement,
        typechecker: &mut Typechecker,
    ) -> CompilerResult<()> {
        let identifier = match intrinsic.args.get(0) {
            None => Err(CompilerError::new(
                ErrorCode::InvalidStatement,
                format!(
                    "Verwachtte een variabelenaam om de waarde in in te lezen, maar vond niets."
                ),
                intrinsic.span,
            )),
            Some(arg) => match &arg.expression {
                ExpressionType::Identifier(ident) => Ok(ident),
                _ => Err(CompilerError::new(
                    ErrorCode::InvalidStatement,
                    format!(
                        "Een waarde kan alleen ingelezen worden in een variabele. Deze expressie is niet geldig hier."
                    ),
                    arg.span,
                )),
            },
        }?;

        let var = match typechecker.find_identifier_mut(&identifier.value) {
            Some(var) => var,
            None => panic!(
                "Could not find variable {}, this should have been caught by the typechecker checking the argument expression",
                identifier.value
            ),
        };

        if var.is_const {
            return Err(CompilerError::new(
                ErrorCode::ConstVariable,
                format!("Het is niet mogelijk om in een constante waarde in te lezen."),
                identifier.span
            ).with_info(var.loc, format!("{} was hier gedefinieerd als constante.", identifier.value))
        .with_help(format!("Verwijder `voorgoed` uit de declaratie van de variabele om hierin in te kunnen lezen.")));
        }

        // @TODO:
        // When we support warnings, probably good to warn against modifying a variable that was already modified here
        var.initialized = true;
        var.modified = true;

        Ok(())
    }
}
