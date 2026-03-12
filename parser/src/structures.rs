use crate::parser::Parser;
use crate::token::Tokenizer;

pub struct PizzaParser;

impl PizzaParser {
    pub fn parse(input: &str) -> Result<Vec<Recipe>, String> {
        // 1. Tokenize
        let mut tokenizer = Tokenizer::new(input);
        let tokens = tokenizer.parse();

        // 2. Parse
        let mut parser = Parser::new(tokens);
        parser.parse_recipe()
    }
}

#[derive(Debug, Clone)]
pub enum BaseType {
    Tomato,
    Cream,
}

#[derive(Debug, Clone)]
pub enum Step {
    MakeDough,
    AddBase { base_type: BaseType },
    AddMushrooms { amount: u32, repeat: u32 },
    AddCheese { amount: u32, repeat: u32 },
    AddPepperoni { slices: u32, repeat: u32 },
    AddGarlic { cloves: u32, repeat: u32 },
    AddOregano { amount: u32, repeat: u32 },
    AddBasil { leaves: u32, repeat: u32 },
    AddOliveOil,
    Bake { duration: u32 }
}

#[derive(Debug, Clone)]
pub enum Steps {
    Single(Step),
    Multiple(Vec<Step>)
}

#[derive(Debug, Clone, Default)]
pub struct Recipe {
    pub name: String,
    pub steps: Vec<Steps>,
}

impl Recipe {

    fn print_step(&self, step: &Step){
        match step {
            Step::MakeDough => println!("MakeDough"),
            Step::AddBase { base_type } => println!("AddBase({base_type:?})"),
            Step::Bake { duration } => println!("Bake({duration:?})"),
            Step::AddMushrooms { amount, repeat } => println!("AddMushrooms(amount: {amount})^{repeat:?}"),
            Step::AddCheese { amount, repeat } => println!("AddCheese(amount: {amount})^{repeat:?}"),
            Step::AddPepperoni { slices, repeat } => println!("AddPepperoni(slices: {slices})^{repeat:?}"),
            Step::AddGarlic { cloves, repeat } => println!("AddGarlic(cloves: {cloves})^{repeat:?}"),
            Step::AddOregano { amount, repeat } => println!("AddOregano(amount: {amount})^{repeat:?}"),
            Step::AddBasil { leaves, repeat } => println!("AddBasil(leaves: {leaves})^{repeat:?}"),
            Step::AddOliveOil => println!("🛢️ AddOliveOil"),
            _ => {println!("Unknow printing step")}
        }
    }
    fn print_steps(&self, steps: &Steps) {

        match steps {
            Steps::Single(step) => self.print_step(step),
            Steps::Multiple(steps) => {
                for step in steps {
                    self.print_step(step);
                }
            },
        }
    }
    pub fn print_recipe(&self) {
        println!("Recette: {:?}", self.name);
        for (i, step) in self.steps.iter().enumerate() {
            print!("  {}. ", i + 1);
            self.print_steps(step);
        }
    }
}
