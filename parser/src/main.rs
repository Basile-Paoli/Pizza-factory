mod token;

fn main() {
    use crate::token::Tokenizer;

    let input = "Funghi =
    MakeDough
    -> AddBase(base_type=tomato)
    -> AddMushrooms(amount=3)
    -> AddCheese(amount=2)
    -> Bake(duration=6)
    -> AddOliveOil";
    let mut t = Tokenizer::new(input);
    t.print_tokens();
}