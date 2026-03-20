#[cfg(test)]
mod tests {
    use crate::{PizzaParser, Step, Steps};

    #[test]
    fn test_funghi() {
        let input = "Funghi =
    MakeDough
    -> AddBase(base_type=tomato)
    -> AddMushrooms(amount=3)
    -> AddCheese(amount=2)
    -> Bake(duration=6)
    -> AddOliveOil
    Funghi2 =
    MakeDough
    -> AddBase(base_type=tomato)
    -> AddMushrooms(amount=3)
    -> AddCheese(amount=2)
    -> Bake(duration=6)
    -> AddOliveOil";

        let recipes = PizzaParser::parse(input).unwrap();
        assert_eq!(recipes.len(), 2);

        let recipe = &recipes[0];
        recipe.print_recipe();
        assert_eq!(recipe.name, "Funghi");
        assert_eq!(recipe.steps.len(), 6);

        let recipe = &recipes[1];
        recipe.print_recipe();
        assert_eq!(recipe.name, "Funghi2");
        assert_eq!(recipe.steps.len(), 6);
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
        assert_eq!(recipes.len(), 1);

        let recipe = &recipes[0];
        assert_eq!(recipe.name, "Pepperoni");
        assert_eq!(recipe.steps.len(), 5);
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
        assert_eq!(recipes.len(), 1);

        let recipe = &recipes[0];
        assert_eq!(recipe.name, "Marinara");
        assert_eq!(recipe.steps.len(), 6);

        if let Steps::Single(Step::AddOregano { amount, repeat }) = &recipe.steps[3] {
            assert_eq!(*amount, 1);
        } else {
            panic!("Expected Steps::Single(Step::AddOregano) at index 3");
        }
    }

    #[test]
    fn test_margarita() {
        let input = "Margherita =
    MakeDough
    -> AddBase(base_type=tomato)
    -> [AddCheese(amount=2), AddBasil(leaves=3)^2]
    -> Bake(duration=5)
    -> AddOliveOil";

        let recipes = PizzaParser::parse(input).unwrap();
        assert_eq!(recipes.len(), 1);

        let recipe = &recipes[0];
        recipe.print_recipe();
        assert_eq!(recipe.name, "Margherita");
        assert_eq!(recipe.steps.len(), 5);

        assert!(matches!(recipe.steps[2], Steps::Multiple(_)));

        if let Steps::Multiple(steps) = &recipe.steps[2] {
            assert_eq!(steps.len(), 2);

            if let Step::AddCheese { amount, repeat } = &steps[0] {
                assert_eq!(*amount, 2);
                assert_eq!(*repeat, 1);
            } else {
                panic!("Expected AddCheese at Multiple[0]");
            }

            if let Step::AddBasil { leaves, repeat } = &steps[1] {
                assert_eq!(*leaves, 3);
                assert_eq!(*repeat, 2);
            } else {
                panic!("Expected AddBasil at Multiple[1]");
            }
        }


        if let Steps::Single(Step::Bake { duration }) = &recipe.steps[3] {
            assert_eq!(*duration, 5);
        } else {
            panic!("Expected Steps::Single(Step::Bake) at index 3");
        }
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
        assert_eq!(recipes.len(), 1);

        let recipe = &recipes[0];
        recipe.print_recipe();
        assert_eq!(recipe.name, "QuattroFormaggi");

        if let Steps::Single(Step::AddCheese { amount, repeat }) = &recipe.steps[2] {
            assert_eq!(*amount, 1);
            assert_eq!(*repeat, 4);
        } else {
            panic!("Expected Steps::Single(Step::AddCheese) at index 3 with duration 6 and repeat 4");
        }

        if let Steps::Single(Step::Bake { duration }) = &recipe.steps[3] {
            assert_eq!(*duration, 6);
        } else {
            panic!("Expected Steps::Single(Step::Bake) at index 3");
        }
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
        assert_eq!(recipes.len(), 1);

        let recipe = &recipes[0];
        assert_eq!(recipe.name, "Custom");
        assert!(matches!(recipe.steps[2], Steps::Multiple(_)));

        if let Steps::Single(Step::Bake { duration }) = &recipe.steps[3] {
            assert_eq!(*duration, 6);
        } else {
            panic!("Expected Steps::Single(Step::Bake) at index 3");
        }
    }
}