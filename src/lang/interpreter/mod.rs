use crate::{
    lang::compiler::{
        Compiler, CompilerError,
        expression::ExpressionType,
        intrinsic::Intrinsics,
        statement::{IntrinsicStatement, SpannedStatement, StatementType},
    },
    logger::ErrorLogger,
};

pub(crate) struct Interpreter<'a> {
    input: &'a str,
    filename: &'a str,
    error_logger: ErrorLogger<'a>,
}

impl Interpreter<'_> {
    pub(crate) fn new<'a>(input: &'a str, filename: &'a str) -> Interpreter<'a> {
        let error_logger = ErrorLogger::new(input, filename);
        Interpreter {
            input,
            filename,
            error_logger,
        }
    }

    pub(crate) fn run(&mut self) -> Result<String, String> {
        let mut compiler = Compiler::new(&self.input, &self.filename);
        let mut ast = compiler.compile().unwrap_or_else(|err| {
            self.error_logger.report(&err);
            panic!("Error compiling file: {}", err.error_message);
        });

        let mut output = Vec::<String>::new();

        while let Some(stmt) = ast.next() {
            let stmt = self.interpret_stmt(stmt).map_err(|err| {
                self.error_logger.report(&err);
                return format!("Error interpreting program: {}", err.error_message);
            })?;

            if stmt.is_some() {
                output.push(stmt.unwrap());
            }
        }

        Ok(output.join("\n").to_string())
    }

    fn interpret_stmt(&self, stmt: SpannedStatement) -> Result<Option<String>, CompilerError> {
        match stmt.0 {
            StatementType::Intrinsic(intrinsic) => self.interpret_intrinsic(intrinsic),
            StatementType::VariableDeclaration(declaratino) => todo!(),
        }
    }

    fn interpret_intrinsic(
        &self,
        intrinsic: IntrinsicStatement,
    ) -> Result<Option<String>, CompilerError> {
        let mut output = Vec::<String>::new();
        match intrinsic.intrinsic {
            Intrinsics::Print => intrinsic.args.iter().for_each(|arg| match arg.expression {
                ExpressionType::StringExpr(ref content) => output.push(content.text.clone()),
                ExpressionType::Identifier(ref content) => todo!(),
            }),
        };

        Ok(Some(output.join("\n").to_string()))
    }
}
