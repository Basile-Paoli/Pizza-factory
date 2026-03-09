mod token;
mod parser;

fn main() {
    use crate::token::Tokenizer;

    let input = "Margherita =
    MakeDough
    -> AddBase(base_type=tomato)
    -> [AddCheese(amount=2), AddBasil(leaves=3)]
    -> Bake(duration=5)
    -> AddPepperoni(slices=1)^12
    -> AddOliveOil";
    let mut t = Tokenizer::new(input);
    t.print_tokens();

    let mut p = Parser::new(t.parse());
    p.parse_recipe();
}