//! Handlers TCP du serveur de production.
//!
//! Chaque connexion TCP entrante est traitée dans un thread séparé.
//! Le handler lit un seul message, l'identifie, et appelle la fonction appropriée.

use crate::gossip::GossipHandle;
use parser::PizzaParser;
use shared::framing::{read_message, write_message};
use shared::message::{LocalRecipeStatus, Payload, RecipeStatus, TcpMessage, Update};
use std::collections::{HashMap, HashSet};
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, RwLock};
use uuid::Uuid;

use super::actions::{execute_action, recipe_to_action_sequence};

// ─── Contexte partagé ──────────────────────────────────────────────────────

/// Contexte global de l'agent, partagé entre tous les threads de production.
///
/// Contient tout ce dont les handlers ont besoin : adresse, capacités,
/// recettes, accès au gossip, et table des commandes en attente.
pub struct AgentContext {
    /// Adresse TCP de cet agent (ex: "127.0.0.1:8001")
    pub addr: SocketAddr,
    /// Noms des actions que cet agent sait exécuter (ex: {"MakeDough", "Bake"})
    pub capabilities: RwLock<HashSet<String>>,
    /// Recettes connues localement : nom → texte DSL
    pub recipe_store: Arc<HashMap<String, String>>,
    /// Interface vers l'état gossip pour trouver des pairs
    pub gossip: GossipHandle,
    /// Commandes en attente de livraison.
    /// Clé : UUID de la commande.
    /// Valeur : (nom de la recette, canal pour recevoir le résultat).
    pub pending_orders: Arc<Mutex<HashMap<Uuid, (String, Sender<Result<Payload, String>>)>>>,
}

impl AgentContext {
    /// Crée un nouveau contexte agent et le place dans un `Arc` (pointeur partagé).
    pub fn new(
        addr: SocketAddr,
        capabilities: HashSet<String>,
        recipe_store: HashMap<String, String>,
        gossip: GossipHandle,
    ) -> Arc<Self> {
        Arc::new(Self {
            addr,
            capabilities: RwLock::new(capabilities),
            recipe_store: Arc::new(recipe_store),
            gossip,
            pending_orders: Arc::new(Mutex::new(HashMap::new())),
        })
    }
}

// ─── Dispatcher principal ──────────────────────────────────────────────────

/// Gère une connexion TCP entrante : lit le message et l'envoie au bon handler.
pub fn handle_connection(mut stream: TcpStream, ctx: Arc<AgentContext>) {
    match read_message::<TcpMessage>(&mut stream) {
        Ok(msg) => match msg {
            TcpMessage::Order { recipe_name } => handle_order(stream, ctx, recipe_name),
            TcpMessage::ProcessPayload { payload } => handle_process_payload(ctx, payload),
            TcpMessage::Deliver { payload, error } => handle_deliver(ctx, payload, error),
            TcpMessage::ListRecipes {} => handle_list_recipes(stream, ctx),
            TcpMessage::GetRecipe { recipe_name } => handle_get_recipe(stream, ctx, recipe_name),
            other => eprintln!("[agent] Message inattendu: {:?}", other),
        },
        Err(e) => eprintln!("[agent] Erreur de lecture: {}", e),
    }
}

// ─── Handler : order ──────────────────────────────────────────────────────

/// Gère une commande client `order`.
///
/// **Flux :**
/// 1. Cherche la recette dans le store local
/// 2. Parse la recette → séquence d'actions
/// 3. Envoie `order_receipt` au client (sur la même connexion)
/// 4. Lance la chaîne de production (envoie `process_payload` à soi-même)
/// 5. Attend le `deliver` final (bloquant)
/// 6. Envoie `completed_order` au client
///
/// La connexion avec le client reste **ouverte** pendant toute la production.
fn handle_order(mut stream: TcpStream, ctx: Arc<AgentContext>, recipe_name: String) {
    // 1. Chercher la recette (localement, puis chez un pair)
    let recipe_dsl = match ctx.recipe_store.get(&recipe_name) {
        Some(dsl) => dsl.clone(),
        None => match fetch_recipe_from_peer(&ctx, &recipe_name) {
            Some(dsl) => dsl,
            None => {
                let reason = format!("Recette inconnue: '{}'", recipe_name);
                eprintln!("[agent] {}", reason);
                decline_order(&mut stream, reason);
                return;
            }
        },
    };

    // 2. Parser la recette en séquence d'actions
    let action_sequence = match PizzaParser::parse_single(&recipe_dsl) {
        Ok(recipe) => recipe_to_action_sequence(&recipe.steps),
        Err(e) => {
            let reason = format!("Erreur de parsing de '{}': {}", recipe_name, e);
            eprintln!("[agent] {}", reason);
            decline_order(&mut stream, reason);
            return;
        }
    };

    // 2bis. Vérifier que toutes les actions sont réalisables par le cluster
    let mut all_caps: HashSet<String> = ctx
        .capabilities
        .read()
        .expect("lock capabilities empoisonné")
        .clone();
    all_caps.extend(ctx.gossip.get_all_peer_capabilities());
    let missing: Vec<String> = action_sequence
        .iter()
        .map(|a| a.name.clone())
        .filter(|n| !all_caps.contains(n))
        .collect();
    if !missing.is_empty() {
        let reason = format!(
            "Actions non réalisables pour '{}': {}",
            recipe_name,
            missing.join(", ")
        );
        eprintln!("[agent] {}", reason);
        decline_order(&mut stream, reason);
        return;
    }

    // 3. Créer le payload initial
    let order_id = Uuid::new_v4();
    let payload = Payload {
        order_id: order_id.into(),
        order_timestamp: Payload::now_micros(),
        delivery_host: ctx.addr.into(),
        action_index: 0,
        action_sequence,
        content: String::new(),
        updates: Vec::new(),
    };

    // 4. Accuser réception au client
    if let Err(e) = write_message(
        &mut stream,
        &TcpMessage::OrderReceipt {
            order_id: order_id.into(),
        },
    ) {
        eprintln!("[agent] Erreur envoi order_receipt: {}", e);
        return;
    }
    eprintln!(
        "[agent] Commande {} acceptée (recette: {})",
        order_id, recipe_name
    );

    // 5. Créer un canal d'attente et enregistrer la commande
    let (tx, rx) = std::sync::mpsc::channel();
    ctx.pending_orders
        .lock()
        .expect("lock pending_orders empoisonné")
        .insert(order_id, (recipe_name.clone(), tx));

    // 6. Démarrer la chaîne de production (process_payload → soi-même)
    send_process_payload(ctx.addr, payload);

    // 7. Attendre la livraison (bloquant — le thread reste ici jusqu'au résultat)
    match rx.recv() {
        Ok(Ok(final_payload)) => {
            let result = final_payload.to_result_string();
            let msg = TcpMessage::CompletedOrder {
                recipe_name,
                result,
            };
            if let Err(e) = write_message(&mut stream, &msg) {
                eprintln!("[agent] Erreur envoi completed_order: {}", e);
            } else {
                eprintln!("[agent] Commande {} livrée", order_id);
            }
        }
        Ok(Err(err)) => {
            eprintln!("[agent] Erreur de production pour {}: {}", order_id, err);
            let msg = TcpMessage::CompletedOrder {
                recipe_name,
                result: format!("{{\"error\":{:?}}}", err),
            };
            let _ = write_message(&mut stream, &msg);
        }
        Err(e) => {
            eprintln!("[agent] Canal de livraison fermé pour {}: {}", order_id, e);
        }
    }
}

// ─── Handler : process_payload ────────────────────────────────────────────

/// Gère un message `process_payload` (chaîne de production inter-agents).
///
/// **Algorithme :**
/// - Si l'agent sait exécuter l'action courante :
///   - Exécute l'action → ajoute le résultat à `content`
///   - Incrémente `action_index`
///   - Si toutes les actions sont faites → envoie `deliver` au `delivery_host`
///   - Sinon → envoie `process_payload` à soi-même (nouvelle connexion)
/// - Sinon :
///   - Cherche un pair qui sait faire cette action (via le gossip)
///   - Envoie `process_payload` au pair
///   - Si aucun pair → envoie `deliver` avec une erreur
fn handle_process_payload(ctx: Arc<AgentContext>, mut payload: Payload) {
    if payload.action_index >= payload.action_sequence.len() {
        // Cas dégénéré : toutes les actions déjà faites
        send_deliver(payload.delivery_host.0, payload, None);
        return;
    }

    let action = payload.action_sequence[payload.action_index].clone();
    eprintln!(
        "[agent] process_payload action[{:?}]: '{}' (index {}/{})",
        payload.order_id,
        action.name,
        payload.action_index,
        payload.action_sequence.len()
    );

    let can_handle_locally = ctx
        .capabilities
        .read()
        .expect("lock capabilities empoisonné")
        .contains(&action.name);
    if can_handle_locally {
        // L'agent sait faire cette action
        match execute_action(&action) {
            Ok(contribution) => {
                payload.content.push_str(&contribution);
                payload.updates.push(Update::Action {
                    action: action.clone(),
                    timestamp: Payload::now_micros(),
                });
                payload.action_index += 1;

                if payload.action_index >= payload.action_sequence.len() {
                    // Chaîne terminée → livrer
                    payload.updates.push(Update::Forward {
                        to: payload.delivery_host,
                        timestamp: Payload::now_micros(),
                    });
                    send_deliver(payload.delivery_host.0, payload, None);
                } else {
                    // Encore des actions → se renvoyer à soi-même
                    send_process_payload(ctx.addr, payload);
                }
            }
            Err(err) => {
                send_deliver(payload.delivery_host.0, payload, Some(err));
            }
        }
    } else {
        // L'agent ne sait pas faire cette action → chercher un pair
        match ctx.gossip.find_peer_for_action(&action.name) {
            Some(peer_addr) => {
                eprintln!("[agent] Transfert '{}' → {}", action.name, peer_addr);
                payload.updates.push(Update::Forward {
                    to: peer_addr.into(),
                    timestamp: Payload::now_micros(),
                });
                send_process_payload(peer_addr, payload);
            }
            None => {
                let err = format!("Aucun agent ne sait faire '{}'", action.name);
                eprintln!("[agent] {}", err);
                send_deliver(payload.delivery_host.0, payload, Some(err));
            }
        }
    }
}

// ─── Handler : deliver ────────────────────────────────────────────────────

/// Gère un message `deliver` (livraison finale de la pizza).
///
/// Cherche la commande en attente et envoie le résultat via le canal mpsc.
/// Le thread du handler `order` reçoit ce résultat et envoie `completed_order` au client.
fn handle_deliver(ctx: Arc<AgentContext>, mut payload: Payload, error: Option<String>) {
    let order_id = payload.order_id.0;

    // Marquer la livraison dans le journal
    payload.updates.push(Update::Deliver {
        timestamp: Payload::now_micros(),
    });

    let entry = ctx
        .pending_orders
        .lock()
        .expect("lock pending_orders empoisonné")
        .remove(&order_id);

    match entry {
        Some((_, tx)) => {
            let result = match error {
                None => Ok(payload),
                Some(e) => Err(e),
            };
            if let Err(e) = tx.send(result) {
                eprintln!("[agent] Canal fermé pour {}: {}", order_id, e);
            }
        }
        None => {
            eprintln!("[agent] deliver pour commande inconnue: {}", order_id);
        }
    }
}

// ─── Handler : list_recipes ───────────────────────────────────────────────

/// Gère une demande `list_recipes`.
///
/// Pour chaque recette locale, calcule les `missing_actions` :
/// actions que ni cet agent ni aucun pair ne sait exécuter.
fn handle_list_recipes(mut stream: TcpStream, ctx: Arc<AgentContext>) {
    // Combiner les capabilities locales et celles des pairs
    let mut all_caps: HashSet<String> = ctx
        .capabilities
        .read()
        .expect("lock capabilities empoisonné")
        .clone();
    all_caps.extend(ctx.gossip.get_all_peer_capabilities());

    let mut recipes = HashMap::new();
    for (name, dsl) in ctx.recipe_store.iter() {
        let missing = match PizzaParser::parse_single(dsl) {
            Ok(recipe) => {
                let actions = recipe_to_action_sequence(&recipe.steps);
                actions
                    .iter()
                    .filter(|a| !all_caps.contains(&a.name))
                    .map(|a| a.name.clone())
                    .collect::<Vec<_>>()
            }
            Err(_) => vec![],
        };
        recipes.insert(
            name.clone(),
            RecipeStatus::Local(LocalRecipeStatus {
                missing_actions: missing,
            }),
        );
    }

    for (name, host) in ctx.gossip.get_all_peer_recipes() {
        recipes
            .entry(name)
            .or_insert_with(|| RecipeStatus::Remote { host: host.into() });
    }

    let resp = TcpMessage::RecipeListAnswer { recipes };
    if let Err(e) = write_message(&mut stream, &resp) {
        eprintln!("[agent] Erreur envoi recipe_list_answer: {}", e);
    }
}

// ─── Handler : get_recipe ─────────────────────────────────────────────────

/// Gère une demande `get_recipe` : retourne le DSL de la recette demandée.
fn handle_get_recipe(mut stream: TcpStream, ctx: Arc<AgentContext>, recipe_name: String) {
    match ctx.recipe_store.get(&recipe_name) {
        Some(dsl) => {
            let resp = TcpMessage::RecipeAnswer {
                recipe: dsl.clone(),
            };
            if let Err(e) = write_message(&mut stream, &resp) {
                eprintln!("[agent] Erreur envoi recipe_answer: {}", e);
            }
        }
        None => {
            eprintln!("[agent] get_recipe: recette inconnue '{}'", recipe_name);
        }
    }
}

// ─── Fonctions d'envoi (nouvelles connexions TCP) ─────────────────────────

/// Envoie `OrderDeclined` au client sur la connexion en cours.
fn decline_order(stream: &mut TcpStream, message: String) {
    let msg = TcpMessage::OrderDeclined { message };
    if let Err(e) = write_message(stream, &msg) {
        eprintln!("[agent] Erreur envoi order_declined: {}", e);
    }
}

/// Demande le DSL d'une recette à un pair qui l'annonce via le gossip.
///
/// Retourne `None` si aucun pair ne connaît la recette ou si l'échange TCP échoue.
fn fetch_recipe_from_peer(ctx: &AgentContext, recipe_name: &str) -> Option<String> {
    let peer_addr = ctx.gossip.find_peer_for_recipe(recipe_name)?;
    eprintln!(
        "[agent] Recette '{}' absente localement, demande à {}",
        recipe_name, peer_addr
    );
    let mut stream = match TcpStream::connect(peer_addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "[agent] Connexion impossible vers {} (get_recipe): {}",
                peer_addr, e
            );
            return None;
        }
    };
    let req = TcpMessage::GetRecipe {
        recipe_name: recipe_name.to_string(),
    };
    if let Err(e) = write_message(&mut stream, &req) {
        eprintln!("[agent] Erreur envoi get_recipe → {}: {}", peer_addr, e);
        return None;
    }
    match read_message::<TcpMessage>(&mut stream) {
        Ok(TcpMessage::RecipeAnswer { recipe }) => Some(recipe),
        Ok(other) => {
            eprintln!(
                "[agent] Réponse inattendue à get_recipe depuis {}: {:?}",
                peer_addr, other
            );
            None
        }
        Err(e) => {
            eprintln!(
                "[agent] Erreur lecture recipe_answer depuis {}: {}",
                peer_addr, e
            );
            None
        }
    }
}

/// Ouvre une nouvelle connexion TCP et envoie un `process_payload`.
///
/// Chaque action de la chaîne crée une nouvelle connexion — c'est le protocole.
fn send_process_payload(addr: SocketAddr, payload: Payload) {
    std::thread::spawn(move || match TcpStream::connect(addr) {
        Ok(mut stream) => {
            let msg = TcpMessage::ProcessPayload { payload };
            if let Err(e) = write_message(&mut stream, &msg) {
                eprintln!("[agent] Erreur envoi process_payload → {}: {}", addr, e);
            }
        }
        Err(e) => {
            eprintln!(
                "[agent] Connexion impossible vers {} (process_payload): {}",
                addr, e
            );
        }
    });
}

/// Ouvre une nouvelle connexion TCP et envoie un `deliver`.
fn send_deliver(addr: SocketAddr, payload: Payload, error: Option<String>) {
    std::thread::spawn(move || match TcpStream::connect(addr) {
        Ok(mut stream) => {
            let msg = TcpMessage::Deliver { payload, error };
            if let Err(e) = write_message(&mut stream, &msg) {
                eprintln!("[agent] Erreur envoi deliver → {}: {}", addr, e);
            }
        }
        Err(e) => {
            eprintln!(
                "[agent] Connexion impossible vers {} (deliver): {}",
                addr, e
            );
        }
    });
}
