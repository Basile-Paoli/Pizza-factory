

#[derive(Debug, Clone)]
pub enum BaseType {
    Tomato,
    Cream,
}

#[derive(Debug, Clone)]
pub enum Step {
    MakeDough,
    AddBase { base_type: BaseType },
    AddMushrooms { amount: u32 },
    AddCheese { amount: u32 },
    AddPepperoni { slices: u32 },
    AddGarlic { cloves: u32 },
    AddOregano { amount: u32 },
    AddBasil { leaves: u32 },
    AddOliveOil,
    Bake { duration: u32 }
}

#[derive(Debug, Clone, Default)]
pub struct Recipe {
    pub name: String,
    pub steps: Vec<Step>,
}

impl Recipe {

    fn print_step(&self,step: &Step) {
        match step {
            Step::MakeDough => println!("🥖 MakeDough"),
            Step::AddBase { base_type } => println!("🍕 AddBase({base_type:?})"),
            Step::AddMushrooms { amount } => println!("🍄 AddMushrooms(amount: {amount})"),
            Step::AddCheese { amount } => println!("🧀 AddCheese(amount: {amount})"),
            Step::AddPepperoni { slices } => println!("🌭 AddPepperoni(slices: {slices})"),
            Step::AddGarlic { cloves } => println!("🧄 AddGarlic(cloves: {cloves})"),
            Step::AddOregano { amount } => println!("🌿 AddOregano(amount: {amount})"),
            Step::AddBasil { leaves } => println!("🌱 AddBasil(leaves: {leaves})"),
            Step::AddOliveOil => println!("🛢️ AddOliveOil"),
            _ => {}
        }
    }
    pub fn print_recipe(&self) {
        println!("Recette: {:?}", self.name);
        for (i, step) in self.steps.iter().enumerate() {
            print!("  {}. ", i + 1);
            self.print_step(step);
        }
    }
}
