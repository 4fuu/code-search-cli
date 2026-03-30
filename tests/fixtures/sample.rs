pub mod utils;

pub const MAX_SIZE: usize = 1024;
static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Parser {
    input: String,
    position: usize,
}

#[derive(Debug)]
pub enum Error {
    InvalidSyntax(String),
    UnexpectedEof,
}

pub trait Parseable {
    fn parse(input: &str) -> Result<Self> where Self: Sized;
    fn validate(&self) -> bool;
}

impl Parser {
    pub fn new(input: &str) -> Self {
        Parser {
            input: input.to_string(),
            position: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        while self.position < self.input.len() {
            tokens.push(self.next_token()?);
        }
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token> {
        todo!()
    }
}

impl Parseable for Parser {
    fn parse(input: &str) -> Result<Self> {
        Ok(Parser::new(input))
    }

    fn validate(&self) -> bool {
        !self.input.is_empty()
    }
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
}

#[derive(Debug)]
pub enum TokenKind {
    Identifier,
    Number,
    Symbol,
}

pub fn tokenize(input: &str) -> Result<Vec<Token>> {
    let mut parser = Parser::new(input);
    parser.parse()
}
