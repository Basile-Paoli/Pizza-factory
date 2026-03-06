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

struct Tokenizer<'a> {
    input: &'a str,
    pos: usize,
}