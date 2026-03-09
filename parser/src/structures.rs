

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
}

#[derive(Debug, Clone, Default)]
pub struct Recipe {
    pub name: String,
    pub steps: Vec<Step>,
}
