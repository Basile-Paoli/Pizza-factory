use crate::structures::{BaseType, Recipe, Step};
use crate::token::Token;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

#[derive(Debug, Clone)]
pub enum ParamValue {
    Number(u32),
    String(String),
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse_recipe(&mut self) -> Result<Vec<Recipe>, String> {
        let mut recipes: Vec<Recipe> = Vec::new();

        while self.pos < self.tokens.len() {
            let rec = self.parse()?;
            recipes.push(rec);
        }
        Ok(recipes)
    }

    fn expect_next_token(&mut self, expected: Token) -> Result<Token, String> {
        match self.tokens.get(self.pos) {
            Some(token) if token == &expected => {
                self.pos += 1;
                Ok(token.clone())
            }
            Some(found) => Err(format!("Expected {:?}, found {:?}", expected, found)),
            None => Err("Fin des tokens (next_token)".to_string()),
        }
    }

    fn expect_next_token_array(&mut self, expected: &[Token]) -> Result<Token, String> {
        if let Some(token) = self.tokens.get(self.pos) {
            if expected.contains(token) {
                self.pos += 1;
                Ok(token.clone())
            } else {
                Err(format!("Expected one of {:?}, found {:?}", expected, token))
            }
        } else {
            Err("Fin des tokens (expect next token array)".to_string())
        }
    }

    fn expect_next_string(&mut self) -> Result<String, String> {
        match self.tokens.get(self.pos) {
            Some(Token::String(s)) => {
                self.pos += 1;
                Ok(s.clone())
            }
            Some(found) => Err(format!("Expected String, found {:?}", found)),
            None => Err("Fin des tokens (expect next string)".to_string()),
        }
    }

    fn expect_next_number(&mut self) -> Result<u32, String> {
        match self.tokens.get(self.pos) {
            Some(Token::Number(s)) => {
                self.pos += 1;
                Ok(s.clone())
            }
            Some(found) => Err(format!("Expected Number, found {:?}", found)),
            None => Err("Fin des tokens (expect next number)".to_string()),
        }
    }

    fn assign_to_step(
        &mut self,
        step_name: String,
        step_param_name: String,
        step_param_value: Option<ParamValue>,
    ) -> Result<Step, String> {
        return match (step_name.as_str(), step_param_name.as_str(), &step_param_value) {
            ("AddBase", "base_type", &Some(ParamValue::String(ref s))) => {
                let base_type = match s.as_str() {
                    "tomato" | "Tomato" => BaseType::Tomato,
                    "cream" | "Cream" => BaseType::Cream,
                    _ => return Err(format!("Invalid base_type '{}'", s)),
                };
                Ok(Step::AddBase { base_type })
            },
            ("AddMushrooms", "amount", &Some(ParamValue::Number(n))) => Ok(Step::AddMushrooms { amount: n }),
            ("AddCheese", "amount", &Some(ParamValue::Number(n))) => Ok(Step::AddCheese { amount: n }),
            ("AddPepperoni", "slices", &Some(ParamValue::Number(n))) => Ok(Step::AddPepperoni { slices: n }),
            ("AddGarlic", "cloves", &Some(ParamValue::Number(n))) => Ok(Step::AddGarlic { cloves: n }),
            ("AddOregano", "amount", &Some(ParamValue::Number(n))) => Ok(Step::AddOregano { amount: n }),
            ("AddBasil", "leaves", &Some(ParamValue::Number(n))) => Ok(Step::AddBasil { leaves: n }),
            ("Bake", "duration", &Some(ParamValue::Number(n))) => Ok(Step::Bake { duration: n }),

            ("MakeDough", "", &None) => Ok(Step::MakeDough),
            ("AddOliveOil", "", &None) => Ok(Step::AddOliveOil),

            (_, param, &Some(_)) if !param.is_empty() => Err(format!("{} ne prend pas de paramètres", step_name)),
            (_, "", &None) => Err(format!("{} requires parameters", step_name)),
            _ => Err(format!("Unknown combination: step='{}' param='{}' value={:?}",
                             step_name, step_param_name, step_param_value)),
        }

    }

    fn print_current_token(&mut self) {
        match self.tokens.get(self.pos) {
            Some(token) => println!("Token actuel [pos {}]: {:?}", self.pos, token),
            None => println!("Fin des tokens (pos: {})", self.pos),
        }
    }

    fn parse(&mut self) -> Result<Recipe, String> {
        let mut recipe: Recipe;

        // Name
        // =
        // MakeDough
        // ->

        let name = self.expect_next_string()?;
        if self.tokens[self.pos] != Token::Equals {
            return Err(format!("Expected \"=\" after the pizza name"));
        }
        recipe = Recipe {
            name: name,
            ..Default::default()
        };

        self.expect_next_token(Token::Equals)?;

        let step = self.parse_step()?;
        recipe.steps.push(step);

        while true {
            if self.pos >= self.tokens.len() {
                break;
            }
            self.expect_next_token(Token::Arrow)?;

            // Parsing [String, String()]
            if let Token::LBracket = self.tokens[self.pos] {
                self.pos += 1;

                loop {
                    recipe.steps.push(self.parse_step()?);

                    match self.expect_next_token_array(&[Token::Comma, Token::RBracket]) {
                        Ok(token) => match token {
                            Token::Comma => continue,
                            Token::RBracket => break,
                            _ => unreachable!(),
                        },
                        Err(e) => {
                            return Err(format!("Expected ',' or ']' in array, got: {}", e));
                        }
                    }
                }
            } else {
                recipe.steps.push(self.parse_step()?);
            }

            if self.expect_next_token(Token::Caret).is_ok() {
                let _caret_value = self.expect_next_number()?;
                //step.repeat = caret_value
            }

        }
        Ok(recipe)
    }

    fn parse_step(&mut self) -> Result<Step, String> {
        self.print_current_token();
        let step_name = self.expect_next_string()?;
        let mut step_param_name = "".to_string();
        let mut step_param_value: Option<ParamValue> = None;


        if self.expect_next_token(Token::LParen).is_ok() {
            step_param_name = self.expect_next_string()?;

            self.expect_next_token(Token::Equals)?;

            step_param_value = match self.tokens.get(self.pos) {
                Some(Token::Number(n)) => {
                    self.pos += 1;
                    Some(ParamValue::Number(*n))
                }
                Some(Token::String(s)) => {
                    self.pos += 1;
                    Some(ParamValue::String(s.clone()))
                }
                Some(found) => {
                    return Err(format!("Expected Number/String, found {:?}", found));
                }
                None => return Err("Fin des tokens (expect next param value)".to_string()),
            };

            if self.expect_next_token(Token::RParen).is_err() {
                return Err(format!("Expected ')', got {:?}", self.tokens[self.pos]));
            }
        }

        let step = self.assign_to_step(step_name, step_param_name, step_param_value)?;

        Ok(step)
    }
}
