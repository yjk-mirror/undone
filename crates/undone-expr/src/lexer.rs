use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Identifiers and literals
    Ident(String),
    StringLit(String),
    IntLit(i64),
    BoolLit(bool),

    // Punctuation
    Dot,
    Comma,
    LParen,
    RParen,

    // Boolean operators
    Bang,
    And,
    Or,

    // Comparison operators
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,

    Eof,
}

#[derive(Debug, Error, Clone)]
pub enum LexError {
    #[error("unexpected character '{0}' at position {1}")]
    UnexpectedChar(char, usize),
    #[error("unterminated string at position {0}")]
    UnterminatedString(usize),
    #[error("integer literal overflows i64")]
    IntegerOverflow,
}

pub fn tokenize(src: &str) -> Result<Vec<Token>, LexError> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = src.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            ' ' | '\t' | '\n' | '\r' => {
                i += 1;
            }
            '.' => {
                tokens.push(Token::Dot);
                i += 1;
            }
            ',' => {
                tokens.push(Token::Comma);
                i += 1;
            }
            '(' => {
                tokens.push(Token::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            '!' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::Ne);
                    i += 2;
                } else {
                    tokens.push(Token::Bang);
                    i += 1;
                }
            }
            '&' if i + 1 < chars.len() && chars[i + 1] == '&' => {
                tokens.push(Token::And);
                i += 2;
            }
            '|' if i + 1 < chars.len() && chars[i + 1] == '|' => {
                tokens.push(Token::Or);
                i += 2;
            }
            '=' if i + 1 < chars.len() && chars[i + 1] == '=' => {
                tokens.push(Token::Eq);
                i += 2;
            }
            '<' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::Le);
                    i += 2;
                } else {
                    tokens.push(Token::Lt);
                    i += 1;
                }
            }
            '>' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::Ge);
                    i += 2;
                } else {
                    tokens.push(Token::Gt);
                    i += 1;
                }
            }
            '"' | '\'' => {
                let quote = chars[i];
                let start = i + 1;
                i += 1;
                while i < chars.len() && chars[i] != quote {
                    i += 1;
                }
                if i >= chars.len() {
                    return Err(LexError::UnterminatedString(start));
                }
                let s: String = chars[start..i].iter().collect();
                tokens.push(Token::StringLit(s));
                i += 1; // closing quote
            }
            c if c.is_ascii_digit() => {
                let start = i;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                let n: String = chars[start..i].iter().collect();
                let value = n.parse::<i64>().map_err(|_| LexError::IntegerOverflow)?;
                tokens.push(Token::IntLit(value));
            }
            c if c.is_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                let tok = match s.as_str() {
                    "true" => Token::BoolLit(true),
                    "false" => Token::BoolLit(false),
                    _ => Token::Ident(s),
                };
                tokens.push(tok);
            }
            c => return Err(LexError::UnexpectedChar(c, i)),
        }
    }

    tokens.push(Token::Eof);
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_method_call() {
        let toks = tokenize("w.hasTrait(\"SHY\")").unwrap();
        assert_eq!(toks[0], Token::Ident("w".into()));
        assert_eq!(toks[1], Token::Dot);
        assert_eq!(toks[2], Token::Ident("hasTrait".into()));
        assert_eq!(toks[3], Token::LParen);
        assert_eq!(toks[4], Token::StringLit("SHY".into()));
        assert_eq!(toks[5], Token::RParen);
    }

    #[test]
    fn tokenizes_boolean_operators() {
        let toks = tokenize("!a && b || c").unwrap();
        assert_eq!(toks[0], Token::Bang);
        assert_eq!(toks[2], Token::And);
        assert_eq!(toks[4], Token::Or);
    }

    #[test]
    fn tokenizes_comparison() {
        let toks = tokenize("x >= 20").unwrap();
        assert_eq!(toks[1], Token::Ge);
        assert_eq!(toks[2], Token::IntLit(20));
    }

    #[test]
    fn single_quote_strings() {
        let toks = tokenize("w.hasTrait('SHY')").unwrap();
        assert_eq!(toks[4], Token::StringLit("SHY".into()));
    }

    #[test]
    fn bool_literals() {
        let toks = tokenize("true && false").unwrap();
        assert_eq!(toks[0], Token::BoolLit(true));
        assert_eq!(toks[2], Token::BoolLit(false));
    }

    #[test]
    fn integer_overflow_returns_error() {
        let result = tokenize("99999999999999999999");
        assert!(
            matches!(result, Err(LexError::IntegerOverflow)),
            "expected IntegerOverflow error, got: {result:?}"
        );
    }
}
