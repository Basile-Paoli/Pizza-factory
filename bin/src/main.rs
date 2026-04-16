//! Point d'entrée du système Pizza Factory.
//!
//! # Utilisation
//!
//! Démarrer un agent :
//! ```text
//! pizza-factory agent --port 8001 --capabilities MakeDough,Bake --recipes recettes.recipes
//! pizza-factory agent --port 8002 --capabilities AddCheese,AddBasil --bootstrap 127.0.0.1:8001
//! ```
//!
//! Utiliser le client :
//! ```text
//! pizza-factory client --agent 127.0.0.1:8001 order Margherita
//! pizza-factory client --agent 127.0.0.1:8001 list-recipes
//! pizza-factory client --agent 127.0.0.1:8001 get-recipe Margherita
//! ```

use agent::gossip::{start_gossip, LocalSkills};
use agent::production::{start_production_server, AgentContext};
use clap::{Parser, Subcommand};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;

// ─── CLI ──────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "pizza-factory",
    about = "Système distribué de fabrication de pizzas"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Agent {
        #[arg(short, long, default_value = "8001")]
        port: u16,

        #[arg(long, default_value = "127.0.0.1")]
        ip: String,


        #[arg(short, long, value_delimiter = ',', default_value = "")]
        capabilities: Vec<String>,

        #[arg(short, long)]
        recipes: Option<String>,

        #[arg(short, long)]
        bootstrap: Option<SocketAddr>,
    },

    Client {
        #[arg(short, long)]
        agent: SocketAddr,

        #[command(subcommand)]
        action: ClientAction,
    },
}

#[derive(Subcommand)]
enum ClientAction {

    Order {
        recipe: String,
    },
    ListRecipes,
    GetRecipe {
        name: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Agent { port, ip, capabilities, recipes, bootstrap } => {
            run_agent(port, ip, capabilities, recipes, bootstrap);
        }
        Commands::Client { agent, action } => {
            run_client(agent, action);
        }
    }
}


fn run_agent(
    port: u16,
    ip: String,
    capabilities: Vec<String>,
    recipes_path: Option<String>,
    bootstrap: Option<SocketAddr>,
) {
    let addr: SocketAddr = format!("{}:{}", ip, port)
        .parse()
        .expect("Adresse invalide (exemple valide : 127.0.0.1:8001)");

    let caps: Vec<String> = capabilities.into_iter().filter(|c| !c.is_empty()).collect();
    let capabilities_set: HashSet<String> = caps.iter().cloned().collect();

    let recipe_store = load_recipes(recipes_path.as_deref());
    let recipe_names: Vec<String> = recipe_store.keys().cloned().collect();

    println!("=== Agent Pizza Factory ===");
    println!("Adresse     : {}", addr);
    println!("Capabilities: {:?}", caps);
    println!("Recettes    : {:?}", recipe_names);
    if let Some(b) = bootstrap {
        println!("Bootstrap   : {}", b);
    }
    println!();

    let local_skills = LocalSkills { capabilities: caps, recipes: recipe_names };
    let (_gossip_cmd, gossip_handle) = start_gossip(addr, local_skills, bootstrap)
        .expect("Impossible de démarrer le gossip UDP");

    let ctx = AgentContext::new(addr, capabilities_set, recipe_store, gossip_handle);

    start_production_server(ctx).expect("Impossible de démarrer le serveur TCP");

    println!("Agent démarré. Appuyez sur Ctrl+C pour arrêter.");

    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}

fn run_client(agent_addr: SocketAddr, action: ClientAction) {
    match action {
        ClientAction::Order { recipe } => {
            println!("Commander '{}' auprès de {}...", recipe, agent_addr);
            match client::order_pizza(agent_addr, &recipe) {
                Ok(result) => {
                    println!("\nPizza prête !");
                    println!("{}", result);
                }
                Err(e) => {
                    eprintln!("Erreur : {}", e);
                    std::process::exit(1);
                }
            }
        }

        ClientAction::ListRecipes => {
            println!("Recettes disponibles sur {} :", agent_addr);
            match client::list_recipes(agent_addr) {
                Ok(recipes) => {
                    let mut names: Vec<_> = recipes.keys().cloned().collect();
                    names.sort();
                    for name in names {
                        let status = &recipes[&name];
                        let missing = &status.local.missing_actions;
                        if missing.is_empty() {
                            println!("  [OK] {}", name);
                        } else {
                            println!("  [--] {} (manque: {})", name, missing.join(", "));
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Erreur : {}", e);
                    std::process::exit(1);
                }
            }
        }

        ClientAction::GetRecipe { name } => {
            match client::get_recipe(agent_addr, &name) {
                Ok(recipe) => println!("{}", recipe),
                Err(e) => {
                    eprintln!("Erreur : {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}



/// Charge les recettes depuis un fichier .recipes et les retourne sous forme de
/// `HashMap<nom, dsl_texte>`.
///
/// Le fichier peut contenir plusieurs recettes séparées par des lignes vides.
/// Chaque recette est au format DSL Pizza Factory (voir `parser/example.recipes`).
fn load_recipes(path: Option<&str>) -> HashMap<String, String> {
    let path = match path {
        Some(p) => p,
        None => return HashMap::new(),
    };

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Impossible de lire '{}': {}", path, e);
            return HashMap::new();
        }
    };

    match parser::PizzaParser::parse(&content) {
        Ok(recipes) => {
            let mut store = HashMap::new();
            for recipe in recipes {
                let dsl = recipe_to_dsl_line(&recipe);
                eprintln!("[agent] Recette chargée: {} = {}", recipe.name, dsl);
                store.insert(recipe.name, dsl);
            }
            store
        }
        Err(e) => {
            eprintln!("Erreur de parsing du fichier '{}': {}", path, e);
            HashMap::new()
        }
    }
}

/// Reconstruit une représentation DSL en une seule ligne depuis une `Recipe` parsée.
///
/// Exemple : `Margherita = MakeDough -> AddBase(base_type=tomato) -> Bake(duration=5)`
fn recipe_to_dsl_line(recipe: &parser::Recipe) -> String {
    use parser::Steps;

    let steps: Vec<String> = recipe
        .steps
        .iter()
        .map(|s| match s {
            Steps::Single(step) => step_to_dsl(step),
            Steps::Multiple(ss) => {
                let inner: Vec<String> = ss.iter().map(step_to_dsl).collect();
                format!("[{}]", inner.join(", "))
            }
        })
        .collect();

    format!("{} = {}", recipe.name, steps.join(" -> "))
}

fn step_to_dsl(step: &parser::Step) -> String {
    use parser::{BaseType, Step};
    match step {
        Step::MakeDough => "MakeDough".into(),
        Step::AddBase { base_type } => format!(
            "AddBase(base_type={})",
            match base_type {
                BaseType::Tomato => "tomato",
                BaseType::Cream => "cream",
            }
        ),
        Step::AddCheese { amount, repeat } => fmt_repeat("AddCheese", &format!("amount={}", amount), *repeat),
        Step::AddMushrooms { amount, repeat } => fmt_repeat("AddMushrooms", &format!("amount={}", amount), *repeat),
        Step::AddPepperoni { slices, repeat } => fmt_repeat("AddPepperoni", &format!("slices={}", slices), *repeat),
        Step::AddGarlic { cloves, repeat } => fmt_repeat("AddGarlic", &format!("cloves={}", cloves), *repeat),
        Step::AddOregano { amount, repeat } => fmt_repeat("AddOregano", &format!("amount={}", amount), *repeat),
        Step::AddBasil { leaves, repeat } => fmt_repeat("AddBasil", &format!("leaves={}", leaves), *repeat),
        Step::AddOliveOil => "AddOliveOil".into(),
        Step::Bake { duration } => format!("Bake(duration={})", duration),
    }
}

fn fmt_repeat(name: &str, param: &str, repeat: u32) -> String {
    if repeat > 1 {
        format!("{}({})^{}", name, param, repeat)
    } else {
        format!("{}({})", name, param)
    }
}
