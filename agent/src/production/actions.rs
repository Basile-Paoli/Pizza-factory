//! Exécution des actions et conversion recette → séquence d'actions.

use parser::{BaseType, Step, Steps};
use shared::message::Action;
use std::collections::HashMap;

/// Convertit les étapes d'une recette parsée en séquence d'actions pour le protocole TCP.
///
/// Une étape `Single` avec `repeat=3` devient 3 actions identiques.
/// Une étape `Multiple([A, B])` devient les actions A puis B (dans l'ordre).
pub fn recipe_to_action_sequence(steps: &[Steps]) -> Vec<Action> {
    steps.iter().flat_map(expand_steps).collect()
}

fn expand_steps(steps: &Steps) -> Vec<Action> {
    match steps {
        Steps::Single(step) => {
            let repeat = repeat_of(step).max(1);
            let action = step_to_action(step);
            (0..repeat).map(|_| action.clone()).collect()
        }
        Steps::Multiple(ss) => {
            let mut out = Vec::new();
            for s in ss {
                let repeat = repeat_of(s).max(1);
                let action = step_to_action(s);
                for _ in 0..repeat {
                    out.push(action.clone());
                }
            }
            out
        }
    }
}

fn repeat_of(step: &Step) -> u32 {
    match step {
        Step::AddMushrooms { repeat, .. } => *repeat,
        Step::AddCheese { repeat, .. } => *repeat,
        Step::AddPepperoni { repeat, .. } => *repeat,
        Step::AddGarlic { repeat, .. } => *repeat,
        Step::AddOregano { repeat, .. } => *repeat,
        Step::AddBasil { repeat, .. } => *repeat,
        _ => 1,
    }
}

fn step_to_action(step: &Step) -> Action {
    let mut params: HashMap<String, String> = HashMap::new();
    let name = match step {
        Step::MakeDough => "MakeDough",
        Step::AddBase { base_type } => {
            params.insert(
                "base_type".into(),
                match base_type {
                    BaseType::Tomato => "tomato".into(),
                    BaseType::Cream => "cream".into(),
                },
            );
            "AddBase"
        }
        Step::AddCheese { amount, .. } => {
            params.insert("amount".into(), amount.to_string());
            "AddCheese"
        }
        Step::AddMushrooms { amount, .. } => {
            params.insert("amount".into(), amount.to_string());
            "AddMushrooms"
        }
        Step::AddPepperoni { slices, .. } => {
            params.insert("slices".into(), slices.to_string());
            "AddPepperoni"
        }
        Step::AddGarlic { cloves, .. } => {
            params.insert("cloves".into(), cloves.to_string());
            "AddGarlic"
        }
        Step::AddOregano { amount, .. } => {
            params.insert("amount".into(), amount.to_string());
            "AddOregano"
        }
        Step::AddBasil { leaves, .. } => {
            params.insert("leaves".into(), leaves.to_string());
            "AddBasil"
        }
        Step::AddOliveOil => "AddOliveOil",
        Step::Bake { duration } => {
            params.insert("duration".into(), duration.to_string());
            "Bake"
        }
    };
    Action { name: name.into(), params }
}

/// Exécute une action et retourne la contribution textuelle à la pizza.
///
/// Retourne `Err` si l'action est inconnue ou si un paramètre obligatoire est absent.
pub fn execute_action(action: &Action) -> Result<String, String> {
    match action.name.as_str() {
        "MakeDough" => Ok("Dough\n".into()),
        "AddBase" => {
            let base = action
                .params
                .get("base_type")
                .ok_or("Paramètre 'base_type' manquant pour AddBase")?;
            Ok(format!("Base({})\n", base))
        }
        "AddCheese" => {
            let n = action.params.get("amount").ok_or("Paramètre 'amount' manquant pour AddCheese")?;
            Ok(format!("Cheese x{}\n", n))
        }
        "AddMushrooms" => {
            let n = action.params.get("amount").ok_or("Paramètre 'amount' manquant pour AddMushrooms")?;
            Ok(format!("Mushrooms x{}\n", n))
        }
        "AddPepperoni" => {
            let n = action.params.get("slices").ok_or("Paramètre 'slices' manquant pour AddPepperoni")?;
            Ok(format!("Pepperoni x{}\n", n))
        }
        "AddGarlic" => {
            let n = action.params.get("cloves").ok_or("Paramètre 'cloves' manquant pour AddGarlic")?;
            Ok(format!("Garlic x{}\n", n))
        }
        "AddOregano" => {
            let n = action.params.get("amount").ok_or("Paramètre 'amount' manquant pour AddOregano")?;
            Ok(format!("Oregano x{}\n", n))
        }
        "AddBasil" => {
            let n = action.params.get("leaves").ok_or("Paramètre 'leaves' manquant pour AddBasil")?;
            Ok(format!("Basil x{}\n", n))
        }
        "AddOliveOil" => Ok("Olive Oil\n".into()),
        "Bake" => {
            let d = action.params.get("duration").ok_or("Paramètre 'duration' manquant pour Bake")?;
            Ok(format!("Baked ({} min)\n", d))
        }
        name => Err(format!("Action inconnue : '{}'", name)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execute_make_dough() {
        let a = Action { name: "MakeDough".into(), params: HashMap::new() };
        assert_eq!(execute_action(&a).unwrap(), "Dough\n");
    }

    #[test]
    fn execute_add_cheese() {
        let a = Action {
            name: "AddCheese".into(),
            params: [("amount".into(), "2".into())].into(),
        };
        assert_eq!(execute_action(&a).unwrap(), "Cheese x2\n");
    }

    #[test]
    fn execute_bake() {
        let a = Action {
            name: "Bake".into(),
            params: [("duration".into(), "5".into())].into(),
        };
        assert_eq!(execute_action(&a).unwrap(), "Baked (5 min)\n");
    }

    #[test]
    fn execute_unknown_returns_err() {
        let a = Action { name: "FlyToMoon".into(), params: HashMap::new() };
        assert!(execute_action(&a).is_err());
    }

    #[test]
    fn quattro_formaggi_expands_repeat() {
        // AddCheese(amount=1)^4 → 4 actions AddCheese
        use parser::{PizzaParser, Steps};
        let recipe = PizzaParser::parse_single(
            "QuattroFormaggi = MakeDough -> AddBase(base_type=cream) -> AddCheese(amount=1)^4 -> Bake(duration=6)",
        )
        .unwrap();
        let actions = recipe_to_action_sequence(&recipe.steps);
        let cheese_count = actions.iter().filter(|a| a.name == "AddCheese").count();
        assert_eq!(cheese_count, 4);
    }
}
