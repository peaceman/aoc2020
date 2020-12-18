use clap::Clap;
use core::fmt::Alignment::Left;
use std::error::Error as StdError;
use std::fmt::Pointer;
use std::fs::File;
use std::hint::unreachable_unchecked;
use std::io::{BufRead, BufReader};

#[derive(Clap)]
struct Opts {
    input: String,
}

/// Adapted http://www.craftinginterpreters.com/
fn main() -> Result<(), Box<dyn StdError>> {
    let opts = Opts::parse();
    let reader = File::open(opts.input).map(BufReader::new)?;

    let result = reader
        .lines()
        .filter_map(|l| l.ok())
        .map(|l| Scanner::new(&l).scan())
        .map(|tokens| Parser::new(tokens).parse())
        .map(|expr| evaluate_expression(&expr))
        .sum::<i64>();

    println!("result: {}", result);

    Ok(())
}

#[derive(Debug, PartialEq, Clone)]
enum Token {
    LeftParen,
    RightParen,
    Minus,
    Plus,
    Slash,
    Star,
    Number(u64),
    EndOfFile,
}

#[derive(Debug)]
enum Expr {
    Binary {
        left: Box<Self>,
        operator: Token,
        right: Box<Self>,
    },
    Unary {
        operator: Token,
        right: Box<Self>,
    },
    Grouping(Box<Self>),
    Literal(Token),
}

#[derive(Default)]
struct Scanner {
    input: Vec<char>,
    start: usize,
    current: usize,
    tokens: Vec<Token>,
}

impl Scanner {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect::<Vec<_>>(),
            ..Self::default()
        }
    }

    pub fn scan(mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token::EndOfFile);
        self.tokens
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.input.len()
    }

    fn scan_token(&mut self) {
        use Token::*;

        let c = self.advance();

        match c {
            '(' => self.add_token(LeftParen),
            ')' => self.add_token(RightParen),
            '-' => self.add_token(Minus),
            '+' => self.add_token(Plus),
            '*' => self.add_token(Star),
            '/' => self.add_token(Slash),
            c if c.is_whitespace() => {}
            c if c.is_ascii_digit() => self.number(),
            _ => {}
        }
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.input[self.current - 1]
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.input[self.current]
        }
    }

    fn add_token(&mut self, token: Token) {
        self.tokens.push(token);
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        let s = self.input[self.start..self.current]
            .iter()
            .collect::<String>();

        self.add_token(Token::Number(s.parse::<u64>().unwrap()))
    }
}

#[derive(Default)]
struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            ..Self::default()
        }
    }

    pub fn parse(mut self) -> Expr {
        self.expression()
    }

    fn match_token(&mut self, check: impl Fn(&Token) -> bool) -> bool {
        if self.check_token(check) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check_token(&self, check: impl Fn(&Token) -> bool) -> bool {
        if self.is_at_end() {
            false
        } else {
            check(self.peek())
        }
    }

    fn consume_token(&mut self, check: impl Fn(&Token) -> bool, error_msg: &'static str) {
        if self.check_token(check) {
            self.advance();
        } else {
            panic!(error_msg);
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Token::EndOfFile)
    }

    fn expression(&mut self) -> Expr {
        self.factor()
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.unary();

        while self.match_token(|t| matches!(t, Token::Minus | Token::Plus)) {
            let operator = self.previous().clone();
            let right = self.unary();

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        expr
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.term();

        while self.match_token(|t| matches!(t, Token::Slash | Token::Star)) {
            let operator = self.previous().clone();
            let right = self.term();

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        expr
    }

    fn unary(&mut self) -> Expr {
        if self.match_token(|t| matches!(t, Token::Minus)) {
            let operator = self.previous().clone();
            let right = self.unary();

            Expr::Unary {
                operator,
                right: Box::new(right),
            }
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Expr {
        if self.match_token(|t| matches!(t, Token::Number(_))) {
            Expr::Literal(self.previous().clone())
        } else if self.match_token(|t| matches!(t, Token::LeftParen)) {
            let expr = self.expression();
            self.consume_token(
                |t| matches!(t, Token::RightParen),
                "Expect ')' after expression.",
            );
            Expr::Grouping(Box::new(expr))
        } else {
            panic!("Expect expression. {:?}", self.peek())
        }
    }
}

fn evaluate_expression(expr: &Expr) -> i64 {
    match expr {
        Expr::Literal(Token::Number(n)) => *n as i64,
        Expr::Grouping(expr) => evaluate_expression(expr.as_ref()),
        Expr::Unary { operator, right } => {
            let right = evaluate_expression(right.as_ref());

            match operator {
                Token::Minus => -right,
                _ => unreachable!(),
            }
        }
        Expr::Binary {
            left,
            operator,
            right,
        } => {
            let left = evaluate_expression(left.as_ref());
            let right = evaluate_expression(right.as_ref());

            match operator {
                Token::Minus => left - right,
                Token::Plus => left + right,
                Token::Star => left * right,
                Token::Slash => left / right,
                _ => unreachable!(),
            }
        }
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{evaluate_expression, Parser, Scanner, Token};

    #[test]
    fn test_scanner() {
        let input = "2 * 3 + (4 * 5)";
        let mut scanner = Scanner::new(input);
        let tokens = scanner.scan();

        assert_eq!(
            vec![
                Token::Number(2),
                Token::Star,
                Token::Number(3),
                Token::Plus,
                Token::LeftParen,
                Token::Number(4),
                Token::Star,
                Token::Number(5),
                Token::RightParen,
                Token::EndOfFile
            ],
            tokens
        );
    }

    #[test]
    fn test_eval() {
        let data = &[
            ("1 + (2 * 3) + (4 * (5 + 6))", 51),
            ("2 * 3 + (4 * 5)", 46),
            ("5 + (8 * 3 + 9 + 3 * 4 * 3)", 1445),
            ("5 * 9 * (7 * 3 * 3 + 9 * 3 + (8 + 6 * 4))", 669060),
            ("((2 + 4 * 9) * (6 + 9 * 8 + 6) + 6) + 2 + 4 * 2", 23340),
        ];

        for (input, result) in data {
            assert_eq!(
                *result,
                evaluate_expression(&Parser::new(Scanner::new(input).scan()).parse())
            );
        }
    }
}
