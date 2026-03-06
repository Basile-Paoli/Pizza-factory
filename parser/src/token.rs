#[derive(Debug, Clone)]
pub enum Token {
    Name(String),      // Pizza Name
    Equals,
    Arrow,             // ->
    LParen, RParen,
    LBracket, RBracket,
    Comma,
    Caret,             // ^
    EqualsSign,
    Ident(String),     // MakeDough, AddCheese, tomato
    Number(u32),
    Newline,
}

pub struct Tokenizer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    pub fn parse(&mut self) -> Option<Token> {
        loop {
            self.skip_whitespace();
            print!("{}", self.input.as_bytes()[self.pos] as char);
            self.pos += 1
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input.as_bytes()[self.pos] as char == ' ' {
            self.pos += 1;
        }
    }
}