#[cfg(test)]
mod tests {
    use crate::{PizzaParser, Recipe};

    #[test]
    fn test_funghi() {
        let input = "Funghi =
    MakeDough
    -> AddBase(base_type=tomato)
    -> AddMushrooms(amount=3)
    -> AddCheese(amount=2)
    -> Bake(duration=6)
    -> AddOliveOil";

        let recipes = PizzaParser::parse(input).unwrap();
        let recipe = &recipes[0];
        recipe.print_recipe();
        assert_eq!(recipe.name, "Funghi");
        assert_eq!(recipe.steps.len(), 6);  // 6 steps attendus
    }

    #[test]
    fn test_pepperoni() {
        let input = "Pepperoni =
    MakeDough
    -> AddBase(base_type=tomato)
    -> AddCheese(amount=2)
    -> AddPepperoni(slices=12)
    -> Bake(duration=6)";

        let recipes = PizzaParser::parse(input).unwrap();
        assert_eq!(recipes[0].name, "Pepperoni");
    }

    #[test]
    fn test_marinara() {
        let input = "Marinara =
    MakeDough
    -> AddBase(base_type=tomato)
    -> AddGarlic(cloves=2)
    -> AddOregano(amount=1)
    -> Bake(duration=5)
    -> AddOliveOil";

        let recipes = PizzaParser::parse(input).unwrap();
        assert_eq!(recipes[0].name, "Marinara");
    }

    #[test]
    fn test_margarita() {
        let input = "Margherita =
    MakeDough
    -> AddBase(base_type=tomato)
    -> [AddCheese(amount=2), AddBasil(leaves=3)]
    -> Bake(duration=5)
    -> AddOliveOil";

        let recipes = PizzaParser::parse(input).unwrap();
        let recipe = &recipes[0];
        recipe.print_recipe();
        assert_eq!(recipe.steps.len(), 5);  // 1 MakeDough + 1 Multiple + 1 Bake + 1 OliveOil
    }

    #[test]
    fn test_quattro_formaggi() {
        let input = "QuattroFormaggi =
    MakeDough
    -> AddBase(base_type=cream)
    -> AddCheese(amount=1)^4
    -> Bake(duration=6)
    -> AddOliveOil";

        let recipes = PizzaParser::parse(input).unwrap();
        let recipe = &recipes[0];
        recipe.print_recipe();
    }

    #[test]
    fn test_custom() {
        let input = "Custom =
    MakeDough
    -> AddBase(base_type=cream)
    -> [AddCheese(amount=1)^4, AddBasil(leaves=1)^4]
    -> Bake(duration=6)
    -> AddOliveOil";

        let recipes = PizzaParser::parse(input).unwrap();
        assert_eq!(recipes[0].name, "Custom");
    }
}

fn main() {}