use super::lexer::token::{Token, TokenType};
use crate::lang::compiler::{CompilerError, ErrorCode, Span, iter::unshift::Unshift};

pub(crate) fn try_parse_numeric_literal(from: &String, span: Span) -> Option<Token> {
    match parse_numeric_literal(from, span) {
        Ok(token) => Some(token),
        Err(_) => None,
    }
}

pub(crate) fn parse_numeric_literal(from: &String, span: Span) -> Result<Token, CompilerError> {
    let chars = from
        .split("")
        .filter(|c| !c.is_empty())
        .collect::<Vec<&str>>();

    let mut output = Vec::new();

    let mut tmp = "".to_string();

    for char in chars {
        tmp += char;
        if WORD_TO_NUMBER.get(tmp.as_str()).is_some() {
            output.push(tmp.clone());
            tmp.clear();
        }
    }

    // If at the end we still have characters in the buffer,
    // something couldn't be matched and the number is likely invalid
    if tmp.len() > 0 {
        return Err(CompilerError {
            kind: ErrorCode::InvalidNumericLiteral,
            message: format!("Invalid numeric literal {}", from),
            span_message: format!("Invalid numeric literal"),
            span,
        });
    }

    let mut acc: usize = 0;
    let mut output = output.iter_mut().peekable();

    while let Some(next) = output.unshift() {
        match next.as_str() {
            "nul" | "een" | "één" | "twee" | "drie" | "vier" | "vijf" | "zes" | "zeven"
            | "acht" | "negen" | "tien" | "elf" | "twaalf" | "dertien" | "veertien" | "twintig"
            | "dertig" | "veertig" | "tachtig" | "honderd" | "duizend" => {
                check_next_part_correct_grammar(next, output.peek(), &from, span)?;

                let value = WORD_TO_NUMBER.get(next.as_str()).unwrap();
                if *value == 1000 {
                    if acc > 999 {
                        // We have already multiplied the number by 1000 before,
                        // so it should not be possible to have "duizend" in the number again
                        return Err(CompilerError {
                            kind: ErrorCode::InvalidNumericLiteral,
                            message: format!("Invalid numeric literal {}", from),
                            span_message: format!("Invalid numeric literal"),
                            span,
                        });
                    }
                    if acc == 0 {
                        acc = 1;
                    }
                    acc *= 1000;
                } else if *value == 100 {
                    // @TODO - Check the validity of the number literal here
                    // This is a bit more tricky, since it is possible to have multiple
                    // hundreds in a string (e.g. if a 100_000 came before)

                    if acc == 0 {
                        acc = 1;
                    }

                    acc += (acc % 1000) * 100 - (acc % 1000);
                } else if output.peek().is_some() && **(output.peek().unwrap()) == "tig".to_string()
                {
                    output.unshift(); // Move the "tig" out of the vector
                    acc += *value * 10;
                } else {
                    acc += *value;
                }

                Ok(())
            }
            "en" | "ën" => {
                check_next_part_correct_grammar(next, output.peek(), &from, span)?;
                Ok(())
            }

            c => Err(CompilerError {
                kind: ErrorCode::InvalidNumericLiteral,
                message: format!("Invalid numeric literal {}", from),
                span_message: format!(
                    "Invalid numeric literal: {} Not registered as number token fragment",
                    c
                ),
                span,
            }),
        }?;
    }

    Ok(Token {
        type_: TokenType::IntegerLiteral,
        value: acc.to_string(),
        span,
    })
}

fn check_next_part_correct_grammar(
    cur: &String,
    next: Option<&&mut String>,
    from: &String,
    span: Span,
) -> Result<(), CompilerError> {
    if match cur.as_str() {
        "één" => next.is_none() || next.is_some() && (**(next.unwrap()) == "en".to_string()),
        "twee" | "drie" => {
            next.is_none()
                || next.is_some()
                    && (**(next.unwrap()) == "ën".to_string()
                        || **(next.unwrap()) == "honderd".to_string()
                        || **(next.unwrap()) == "duizend".to_string())
        }
        "vier" => {
            next.is_none()
                || next.is_some()
                    && (**(next.unwrap()) == "en".to_string()
                        || **(next.unwrap()) == "honderd".to_string()
                        || **(next.unwrap()) == "duizend".to_string())
        }
        "vijf" | "zes" | "zeven" | "negen" => {
            next.is_none()
                || next.is_some()
                    && (**(next.unwrap()) == "en".to_string()
                        || **(next.unwrap()) == "tien".to_string()
                        || **(next.unwrap()) == "honderd".to_string()
                        || **(next.unwrap()) == "duizend".to_string()
                        || **(next.unwrap()) == "tig".to_string())
        }
        "acht" => {
            next.is_none()
                || next.is_some()
                    && (**(next.unwrap()) == "en".to_string()
                        || **(next.unwrap()) == "tien".to_string()
                        || **(next.unwrap()) == "honderd".to_string()
                        || **(next.unwrap()) == "duizend".to_string())
        }
        "tien" | "twintig" | "dertig" | "veertig" | "tachtig" => {
            next.is_none() || next.is_some() && (**(next.unwrap()) == "duizend".to_string())
        }
        "elf" | "twaalf" | "dertien" | "veertien" => {
            next.is_none()
                || next.is_some()
                    && (**(next.unwrap()) == "honderd".to_string()
                        || **(next.unwrap()) == "duizend".to_string())
        }
        "en" | "ën" => {
            next.is_some()
                && (**(next.unwrap()) == "twintig".to_string()
                    || **(next.unwrap()) == "dertig".to_string()
                    || **(next.unwrap()) == "veertig".to_string()
                    || **(next.unwrap()) == "vijf".to_string()
                    || **(next.unwrap()) == "zes".to_string()
                    || **(next.unwrap()) == "zeven".to_string()
                    || **(next.unwrap()) == "tachtig".to_string()
                    || **(next.unwrap()) == "negen".to_string())
        }
        "honderd" => {
            next.is_none()
                || next.is_some()
                    && (**(next.unwrap()) != "en".to_string()
                        && **(next.unwrap()) != "ën".to_string()
                        && **(next.unwrap()) != "tig".to_string()
                        && **(next.unwrap()) != "honderd".to_string())
        }
        "duizend" => {
            next.is_none()
                || next.is_some()
                    && (**(next.unwrap()) != "en".to_string()
                        && **(next.unwrap()) != "ën".to_string()
                        && **(next.unwrap()) != "tig".to_string()
                        && **(next.unwrap()) != "duizend".to_string())
        }
        _ => next.is_none(),
    } {
        Ok(())
    } else {
        Err(CompilerError {
            kind: ErrorCode::InvalidNumericLiteral,
            message: format!("Invalid numeric literal {}", cur),
            span_message: format!("Invalid numeric literal '{}' in {}", from, cur),
            span,
        })
    }
}

static WORD_TO_NUMBER: phf::Map<&'static str, usize> = phf::phf_map! {
  // Compounding words are in this dictionary so they can be matched when
  // number fragments are parsed
  // They are not valid as standalone fragments; this is validated in check_next_part_correct_grammar
  "en" => 0,
  "ën" => 0,
  "tig" => 10,

  "nul" => 0,
  "één" => 1,
  "twee" => 2,
  "drie" => 3,
  "vier" => 4,
  "vijf" => 5,
  "zes" => 6,
  "zeven" => 7,
  "acht" => 8,
  "negen" => 9,

  "tien" => 10,
  "elf" => 11,
  "twaalf" => 12,
  "dertien" => 13,
  "veertien" => 14,

  "twintig" => 20,
  "dertig" => 30,
  "veertig" => 40,
  "tachtig" => 80,

  "honderd" => 100,

  "duizend" => 1_000,

  "miljoen" => 1_000_000,

  "miljard" => 1_000_000_000,
};

#[cfg(test)]
mod tests {
    use crate::lang::compiler::lexer::token::{Token, TokenType};
    use crate::lang::compiler::numeric_literal::parse_numeric_literal;

    macro_rules! assert_number_converted {
        ($literal:expr, $numerical:tt) => {
            let lit = parse_numeric_literal(&String::from($literal), (0, 0));

            if lit.is_err() {
                eprintln!("{}", format!("{:?}", lit.as_ref().err().unwrap()));
                assert!(false);
            }

            assert_eq!(
                lit.unwrap(),
                Token {
                    type_: TokenType::IntegerLiteral,
                    value: format!("{}", $numerical),
                    span: (0, 0)
                }
            );
        };
    }

    #[test]
    fn can_parse_simple_number() {
        assert_number_converted!("nul", 0);
        assert_number_converted!("zeven", 7);
        assert_number_converted!("vijftig", 50);
        assert_number_converted!("vierhonderd", 400);
        assert_number_converted!("vierduizend", 4000);
    }

    #[test]
    fn can_parse_complex_number() {
        assert_number_converted!("vijftien", 15);
        assert_number_converted!("tweeëntwintig", 22);
        assert_number_converted!("achtenvijftig", 58);
        assert_number_converted!("zesennegentig", 96);
        assert_number_converted!("honderdveertig", 140);
        assert_number_converted!("honderdtachtig", 180);
        assert_number_converted!("twaalfhonderd", 1200);
        assert_number_converted!("duizendtweehonderd", 1200);
    }

    #[test]
    fn can_parse_compound_number() {
        assert_number_converted!("zevenenveertig", 47);
        assert_number_converted!("achtentachtig", 88);
        assert_number_converted!("driehonderdvierenveertig", 344);
        assert_number_converted!("achthonderdachtentachtig", 888);
        assert_number_converted!("tweeduizend", 2000);
        assert_number_converted!("tweeduizenddriehonderd", 2300);
        assert_number_converted!("tweeduizenddriehonderdvierenveertig", 2344);
        assert_number_converted!("vijfduizendvierhonderdachtennegentig", 5498);
        assert_number_converted!("duizenddriehonderdvierenveertig", 1344);
        assert_number_converted!("zestienduizenddriehonderdvierenveertig", 16344);
        assert_number_converted!("tachtigduizendachthonderdachtentachtig", 80888);
        assert_number_converted!("achtentachtigduizendachthonderdachtentachtig", 88888);
        assert_number_converted!("tweeëntwintigduizendtweehonderdtweeëntwintig", 22222);
    }
}
