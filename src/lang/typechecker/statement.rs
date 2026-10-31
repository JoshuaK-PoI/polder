use log::debug;

use crate::lang::{
    compiler::{
        CompilerResult,
        parser::statement::{IntrinsicStatement, Statement, VariableDeclarationStatement},
    },
    typechecker::{Typecheck, Typechecker},
};

impl Typecheck for Statement {
    type Value = ();

    fn typecheck(&self, typechecker: &mut Typechecker) -> CompilerResult<Self::Value> {
        debug!("{:?}", self);
        match self {
            Statement::Intrinsic(intrinsic) => typecheck_intrinsic(intrinsic, typechecker),
            Statement::VariableDeclaration(var_decl) => {
                typecheck_variable_declaration(var_decl, typechecker)
            }
            stmt => todo!("Statement {:?}", stmt),
        }
    }
}

fn typecheck_intrinsic(
    intrinsic: &IntrinsicStatement,
    typechecker: &mut Typechecker,
) -> CompilerResult<()> {
    for expression in &intrinsic.args {
        expression.typecheck(typechecker)?;
    }

    Ok(())
}

fn typecheck_variable_declaration(
    var_decl: &VariableDeclarationStatement,
    typechecker: &mut Typechecker,
) -> CompilerResult<()> {
    if typechecker.scopes.is_empty() {
        return Ok(());
    }

    let declared_type = var_decl.value_expression.typecheck(typechecker);

    typechecker.declare(
        var_decl.identifier.value,
        var_decl.identifier.span,
        type_,
        var_decl.is_const,
        initialized,
    )?;

    Ok(())
}
