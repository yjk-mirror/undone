use crate::lexer::{tokenize, LexError, Token};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Receiver {
    Player,    // w
    MaleNpc,   // m
    FemaleNpc, // f
    Scene,     // scene
    GameData,  // gd
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Str(String),
    Int(i64),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub receiver: Receiver,
    pub method: String,
    pub args: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Call(Call),
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    Ne(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    Le(Box<Expr>, Box<Expr>),
    Ge(Box<Expr>, Box<Expr>),
    Lit(Value),
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("lex error: {0}")]
    Lex(#[from] LexError),
    #[error("unexpected token {0:?} at position {1}")]
    Unexpected(Token, usize),
    #[error("unknown receiver '{0}'")]
    UnknownReceiver(String),
    #[error("expected ')' to close argument list")]
    UnclosedArgs,
    #[error("empty expression")]
    Empty,
    #[error("expression nesting exceeds maximum depth ({max_depth})")]
    MaxDepthExceeded { max_depth: usize },
}

const MAX_EXPR_DEPTH: usize = 128;

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) -> &Token {
        let t = &self.tokens[self.pos];
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        t
    }

    fn expect(&mut self, expected: &Token) -> Result<(), ParseError> {
        if self.peek() == expected {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::Unexpected(self.peek().clone(), self.pos))
        }
    }

    fn parse_expr(&mut self, depth: usize) -> Result<Expr, ParseError> {
        self.parse_or(depth)
    }

    fn parse_or(&mut self, depth: usize) -> Result<Expr, ParseError> {
        let mut left = self.parse_and(depth)?;
        while self.peek() == &Token::Or {
            self.advance();
            let right = self.parse_and(depth)?;
            left = Expr::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_and(&mut self, depth: usize) -> Result<Expr, ParseError> {
        let mut left = self.parse_not(depth)?;
        while self.peek() == &Token::And {
            self.advance();
            let right = self.parse_not(depth)?;
            left = Expr::And(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_not(&mut self, depth: usize) -> Result<Expr, ParseError> {
        if depth >= MAX_EXPR_DEPTH {
            return Err(ParseError::MaxDepthExceeded {
                max_depth: MAX_EXPR_DEPTH,
            });
        }
        if self.peek() == &Token::Bang {
            self.advance();
            let inner = self.parse_not(depth + 1)?;
            return Ok(Expr::Not(Box::new(inner)));
        }
        self.parse_compare(depth)
    }

    fn parse_compare(&mut self, depth: usize) -> Result<Expr, ParseError> {
        let left = self.parse_primary(depth)?;
        let op = match self.peek() {
            Token::Eq => {
                self.advance();
                Some("==")
            }
            Token::Ne => {
                self.advance();
                Some("!=")
            }
            Token::Lt => {
                self.advance();
                Some("<")
            }
            Token::Gt => {
                self.advance();
                Some(">")
            }
            Token::Le => {
                self.advance();
                Some("<=")
            }
            Token::Ge => {
                self.advance();
                Some(">=")
            }
            _ => None,
        };
        if let Some(op) = op {
            let right = self.parse_primary(depth)?;
            Ok(match op {
                "==" => Expr::Eq(Box::new(left), Box::new(right)),
                "!=" => Expr::Ne(Box::new(left), Box::new(right)),
                "<" => Expr::Lt(Box::new(left), Box::new(right)),
                ">" => Expr::Gt(Box::new(left), Box::new(right)),
                "<=" => Expr::Le(Box::new(left), Box::new(right)),
                ">=" => Expr::Ge(Box::new(left), Box::new(right)),
                _ => unreachable!(),
            })
        } else {
            Ok(left)
        }
    }

    fn parse_primary(&mut self, depth: usize) -> Result<Expr, ParseError> {
        match self.peek().clone() {
            Token::StringLit(s) => {
                self.advance();
                Ok(Expr::Lit(Value::Str(s)))
            }
            Token::IntLit(n) => {
                self.advance();
                Ok(Expr::Lit(Value::Int(n)))
            }
            Token::BoolLit(b) => {
                self.advance();
                Ok(Expr::Lit(Value::Bool(b)))
            }
            Token::Ident(name) => {
                self.advance();
                // Must be receiver.method(args)
                self.expect(&Token::Dot)?;
                let receiver = match name.as_str() {
                    "w" => Receiver::Player,
                    "m" => Receiver::MaleNpc,
                    "f" => Receiver::FemaleNpc,
                    "scene" => Receiver::Scene,
                    "gd" => Receiver::GameData,
                    other => return Err(ParseError::UnknownReceiver(other.to_string())),
                };
                let method = if let Token::Ident(m) = self.advance().clone() {
                    m
                } else {
                    return Err(ParseError::Unexpected(self.peek().clone(), self.pos));
                };
                self.expect(&Token::LParen)?;
                let mut args = Vec::new();
                while self.peek() != &Token::RParen {
                    if !args.is_empty() {
                        self.expect(&Token::Comma)?;
                    }
                    let arg = match self.advance().clone() {
                        Token::StringLit(s) => Value::Str(s),
                        Token::IntLit(n) => Value::Int(n),
                        Token::BoolLit(b) => Value::Bool(b),
                        Token::Eof => return Err(ParseError::UnclosedArgs),
                        other => return Err(ParseError::Unexpected(other, self.pos)),
                    };
                    args.push(arg);
                }
                self.expect(&Token::RParen)?;
                Ok(Expr::Call(Call {
                    receiver,
                    method,
                    args,
                }))
            }
            Token::LParen => {
                if depth >= MAX_EXPR_DEPTH {
                    return Err(ParseError::MaxDepthExceeded {
                        max_depth: MAX_EXPR_DEPTH,
                    });
                }
                self.advance();
                let inner = self.parse_expr(depth + 1)?;
                self.expect(&Token::RParen)?;
                Ok(inner)
            }
            other => Err(ParseError::Unexpected(other, self.pos)),
        }
    }
}

pub fn parse(src: &str) -> Result<Expr, ParseError> {
    let tokens = tokenize(src)?;
    if tokens == vec![Token::Eof] {
        return Err(ParseError::Empty);
    }
    let mut parser = Parser::new(tokens);
    let expr = parser.parse_expr(0)?;
    if parser.peek() != &Token::Eof {
        return Err(ParseError::Unexpected(parser.peek().clone(), parser.pos));
    }
    Ok(expr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_method_call() {
        let expr = parse("w.hasTrait(\"SHY\")").unwrap();
        assert_eq!(
            expr,
            Expr::Call(Call {
                receiver: Receiver::Player,
                method: "hasTrait".into(),
                args: vec![Value::Str("SHY".into())],
            })
        );
    }

    #[test]
    fn parses_negation() {
        let expr = parse("!w.hasTrait('POSH')").unwrap();
        assert!(matches!(expr, Expr::Not(_)));
    }

    #[test]
    fn parses_and() {
        let expr = parse("w.hasTrait('SHY') && !m.isPartner()").unwrap();
        assert!(matches!(expr, Expr::And(_, _)));
    }

    #[test]
    fn parses_comparison() {
        let expr = parse("w.getSkill('FITNESS') > 20").unwrap();
        assert!(matches!(expr, Expr::Gt(_, _)));
    }

    #[test]
    fn parses_complex_condition() {
        let src = "m.hasTrait('SLEAZY') && !w.hasTrait('BLOCK_ROUGH') || gd.week() > 2";
        assert!(parse(src).is_ok());
    }

    #[test]
    fn errors_on_unknown_receiver() {
        assert!(parse("x.something()").is_err());
    }

    #[test]
    fn errors_on_empty() {
        assert!(parse("").is_err());
    }

    #[test]
    fn errors_on_trailing_tokens() {
        let result = parse("w.hasTrait('SHY') w.hasTrait('POSH')");
        assert!(matches!(
            result,
            Err(ParseError::Unexpected(Token::Ident(_), _))
        ));
    }

    #[test]
    fn errors_when_not_chain_exceeds_max_depth() {
        let expr = format!("{}true", "!".repeat(MAX_EXPR_DEPTH + 1));
        let result = parse(&expr);
        assert!(matches!(result, Err(ParseError::MaxDepthExceeded { .. })));
    }

    #[test]
    fn errors_when_paren_nesting_exceeds_max_depth() {
        let expr = format!(
            "{}true{}",
            "(".repeat(MAX_EXPR_DEPTH + 1),
            ")".repeat(MAX_EXPR_DEPTH + 1)
        );
        let result = parse(&expr);
        assert!(matches!(result, Err(ParseError::MaxDepthExceeded { .. })));
    }
}
