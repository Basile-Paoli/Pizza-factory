use crate::structures::{Recipe, Step};
use crate::token::Token;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse_recipe(&mut self) ->Result<Vec<Recipe>, String> {
        let mut recipes: Vec<Recipe> = Vec::new();

        while self.pos < self.tokens.len(){
            let rec = self.parse()?;
            recipes.push(rec);
        }
        Ok(recipes)

    }

    fn expect_next_token(&mut self, expected: Token) -> Result<Token, String>{
        match self.tokens.get(self.pos) {
            Some(token) if token == &expected => {
                self.pos += 1;
                Ok(token.clone())
            }
            Some(found) => Err(format!("Expected {:?}, found {:?}", expected, found)),
            None => Err("Fin des tokens".to_string()),
        }
    }

    fn expect_next_string(&mut self) -> Result<String, String> {
        match self.tokens.get(self.pos) {
            Some(Token::String(s)) => {
                self.pos += 1;
                Ok(s.clone())
            }
            Some(found) => Err(format!("Expected String, found {:?}", found)),
            None => Err("Fin des tokens".to_string()),
        }
    }

    fn expect_next_number(&mut self) -> Result<u32, String> {
        match self.tokens.get(self.pos) {
            Some(Token::Number(s)) => {
                self.pos += 1;
                Ok(s.clone())
            }
            Some(found) => Err(format!("Expected Number, found {:?}", found)),
            None => Err("Fin des tokens".to_string()),
        }
    }

    fn parse(&mut self) -> Result<Recipe, String>{
        let mut recipe: Recipe;

        let name = self.expect_next_string()?;
        recipe = Recipe { name, ..Default::default() };

        //=
        // String
        // ->

        self.expect_next_token(Token::Equals)?;
        let dought = self.expect_next_string()?;
        if dought != "MakeDough" {
            return Err(format!("Unexpected step : {}", dought));
        }
        self.expect_next_token(Token::Arrow)?;

        recipe.steps.push(Step::MakeDough);
        // [ ?
        // String
        // (String=Number)?
        // ^Number ?
        // ,        <- Only if array
        // String
        // (String=Number)?
        // ^Number ?
        // ] ?

        Ok(recipe)
    }
}