use crate::structures::Recipe;
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
            self.pos += 1;
        }
        Ok(recipes)

    }

    fn expect_next_token(&mut self, expected: Token) -> Result<Token, String>{
        match self.tokens.get(self.pos) {
            Some(token) if token == &expected => {
                self.pos += 1;
                Ok(token.clone())
            }
            Some(found) => Err(format!("Attendu {:?}, trouvé {:?}", expected, found)),
            None => Err("Fin des tokens".to_string()),
        }
    }

    fn parse(&mut self) -> Result<Recipe, String>{
        let mut recipe: Recipe;
        let name = match self.tokens.get(self.pos) {
            Some(Token::String(s)) => {  // Déstructure pour récupérer &str ou String
                self.pos += 1;
                s.clone()  // ou *s si &str, selon le type de Token::String
            }
            Some(other) => return Err(format!("Attendu String, trouvé {:?}", other)),
            None => return Err("Fin des tokens".to_string()),
        };

        let mut recipe = Recipe { name, ..Default::default() };

        self.expect_next_token(Token::Equals);

        //=
        // String
        // ->
        // [ ?
        // String
        // (String=Number)?
        // ^Number ?
        // ] ?

        Ok(recipe)
    }
}