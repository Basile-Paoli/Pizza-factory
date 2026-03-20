#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    String(String),
    Equals,
    Arrow,             // ->
    LParen, RParen,
    LBracket, RBracket,
    Comma,
    Caret,             // ^
    Number(u32),
}

pub struct Tokenizer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        let input_byte = self.input.as_bytes();

        if self.pos >= self.input.len() {
            return None;
        }

        let ch = input_byte[self.pos] as char;
        match ch {
            '=' => { self.pos += 1; return Some(Token::Equals) }
            '-' if self.pos + 1 < self.input.len() && input_byte[self.pos + 1] as char == '>' => { self.pos += 2; return Some(Token::Arrow) }
            '(' => { self.pos += 1; return Some(Token::LParen) }
            ')' => { self.pos += 1; return Some(Token::RParen) }
            '[' => { self.pos += 1; return Some(Token::LBracket) }
            ']' => { self.pos += 1; return Some(Token::RBracket) }
            ',' => { self.pos += 1; return Some(Token::Comma) }
            '^' => { self.pos += 1; return Some(Token::Caret) }
            _ => {}
        }

        let start = self.pos;

        if ch.is_ascii_digit() {
            let mut end = self.pos;
            while end < self.input.len() && (input_byte[end] as char).is_ascii_digit() {
                end += 1;
            }
            let num: u32 = self.input[start..end].parse().unwrap();
            self.pos = end;
            return Some(Token::Number(num));
        }

        if ch.is_ascii_alphabetic(){

            let mut end = self.pos;
            while end < self.input.len() && ( (input_byte[end] as char).is_ascii_alphabetic() || (input_byte[end] as char) == '_' ) {
                end +=1;
            }

            let string = self.input[start..end].to_string();
            self.pos = end;
            return Some(Token::String(string))
        }

        None
    }

    pub fn parse(&mut self) -> Vec<Token>{
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token() {
            tokens.push(token);
        }
        tokens
    }

    fn skip_whitespace(&mut self) {
        let bytes = self.input.as_bytes();
        while self.pos < bytes.len() && bytes[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    pub fn print_tokens(&mut self) {
        println!("Tokens:");
        while let Some(token) = self.next_token() {
            match &token {
                Token::String(name) => println!("  String: \"{}\"", name),
                Token::Equals => println!("  Equals"),
                Token::Arrow => println!("  Arrow (->)"),
                Token::LParen => println!("  LParen ("),
                Token::RParen => println!("  RParen )"),
                Token::LBracket => println!("  LBracket ["),
                Token::RBracket => println!("  RBracket ]"),
                Token::Comma => println!("  Comma ,"),
                Token::Caret => println!("  Caret ^"),
                Token::Number(num) => println!("  Number: {}", num),
                _ => {}
            }
        }
    }
}