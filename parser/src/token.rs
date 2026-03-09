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

    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        let input_byte = self.input.as_bytes();

        if self.pos >= self.input.len() {
            return None;
        }

        let ch = input_byte[self.pos] as char;
        match ch {
            '=' => { self.pos += 1; return Some(Token::Equals) }
            '-' => { self.pos += 2; return Some(Token::Arrow) } // only "->" stat with "-"
            '(' => { self.pos += 1; return Some(Token::LParen) }
            ')' => { self.pos += 1; return Some(Token::RParen) }
            '[' => { self.pos += 1; return Some(Token::LBracket) }
            ']' => { self.pos += 1; return Some(Token::RBracket) }
            ',' => { self.pos += 1; return Some(Token::Comma) }
            '^' => { self.pos += 1; return Some(Token::Caret) }
            _ => {}
        }

        
        self.pos += 1;
        Some(Token::Name("1".to_string()))
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
                Token::Name(name) => println!("  Name: \"{}\"", name),
                Token::Equals => println!("  Equals"),
                Token::Arrow => println!("  Arrow (->)"),
                Token::LParen => println!("  LParen ("),
                Token::RParen => println!("  RParen )"),
                Token::LBracket => println!("  LBracket ["),
                Token::RBracket => println!("  RBracket ]"),
                Token::Comma => println!("  Comma ,"),
                Token::Caret => println!("  Caret ^"),
                Token::Ident(ident) => println!("  Ident: {}", ident),
                Token::Number(num) => println!("  Number: {}", num),
                Token::Newline => println!("  Newline"),
                _ => {}
            }
        }
    }
}