//! Client TCP pour le système Pizza Factory.
//!
//! Fournit des fonctions pour communiquer avec un agent :
//! commander une pizza, lister les recettes, obtenir un DSL.

use shared::framing::{read_message, write_message, FramingError};
use shared::message::{RecipeStatus, TcpMessage};
use std::collections::HashMap;
use std::net::{SocketAddr, TcpStream};

/// Erreur du client.
#[derive(Debug)]
pub enum ClientError {
    Io(std::io::Error),
    Framing(FramingError),
    /// Le serveur a répondu avec un message inattendu
    Protocol(String),
    /// La commande a été refusée par l'agent (recette inconnue, invalide,
    /// ou action non réalisable par le cluster).
    Declined(String),
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::Io(e) => write!(f, "Erreur réseau: {}", e),
            ClientError::Framing(e) => write!(f, "Erreur de protocole: {}", e),
            ClientError::Protocol(s) => write!(f, "Erreur applicative: {}", s),
            ClientError::Declined(s) => write!(f, "Commande refusée: {}", s),
        }
    }
}

impl std::error::Error for ClientError {}

impl From<std::io::Error> for ClientError {
    fn from(e: std::io::Error) -> Self {
        ClientError::Io(e)
    }
}

impl From<FramingError> for ClientError {
    fn from(e: FramingError) -> Self {
        ClientError::Framing(e)
    }
}

/// Commander une pizza et attendre qu'elle soit prête.
///
/// **Protocole :**
/// 1. Envoie `order { recipe_name }` à l'agent
/// 2. Reçoit `order_receipt { order_id }` (accusé de réception immédiat)
/// 3. Reste connecté et attend `completed_order { recipe_name, result }`
///
/// **Retourne** la chaîne JSON décrivant la pizza produite.
///
/// # Exemple
///
/// ```no_run
/// use client::order_pizza;
/// let result = order_pizza("127.0.0.1:8001".parse().unwrap(), "Margherita").unwrap();
/// println!("{}", result);
/// ```
pub fn order_pizza(agent_addr: SocketAddr, recipe_name: &str) -> Result<String, ClientError> {
    let mut stream = TcpStream::connect(agent_addr)?;

    // Envoyer la commande
    write_message(&mut stream, &TcpMessage::Order {
        recipe_name: recipe_name.to_string(),
    })?;

    // Attendre l'accusé de réception (ou un refus immédiat)
    match read_message::<TcpMessage>(&mut stream)? {
        TcpMessage::OrderReceipt { order_id } => {
            eprintln!("[client] Commande acceptée, UUID: {:?}", order_id);
        }
        TcpMessage::OrderDeclined { message } => {
            return Err(ClientError::Declined(message));
        }
        other => {
            return Err(ClientError::Protocol(format!(
                "Attendu order_receipt, reçu: {:?}",
                other
            )));
        }
    }

    // Attendre la livraison finale (peut prendre du temps)
    match read_message::<TcpMessage>(&mut stream)? {
        TcpMessage::CompletedOrder { recipe_name: _, result } => Ok(result),
        other => Err(ClientError::Protocol(format!(
            "Attendu completed_order, reçu: {:?}",
            other
        ))),
    }
}

/// Demander la liste des recettes disponibles sur l'agent.
///
/// Retourne une map `nom_recette → statut` où le statut indique
/// les actions manquantes (vide si la recette est entièrement réalisable).
pub fn list_recipes(
    agent_addr: SocketAddr,
) -> Result<HashMap<String, RecipeStatus>, ClientError> {
    let mut stream = TcpStream::connect(agent_addr)?;
    write_message(&mut stream, &TcpMessage::ListRecipes {})?;

    match read_message::<TcpMessage>(&mut stream)? {
        TcpMessage::RecipeListAnswer { recipes } => Ok(recipes),
        other => Err(ClientError::Protocol(format!(
            "Attendu recipe_list_answer, reçu: {:?}",
            other
        ))),
    }
}

/// Obtenir la définition DSL d'une recette spécifique.
pub fn get_recipe(agent_addr: SocketAddr, recipe_name: &str) -> Result<String, ClientError> {
    let mut stream = TcpStream::connect(agent_addr)?;
    write_message(&mut stream, &TcpMessage::GetRecipe {
        recipe_name: recipe_name.to_string(),
    })?;

    match read_message::<TcpMessage>(&mut stream)? {
        TcpMessage::RecipeAnswer { recipe } => Ok(recipe),
        other => Err(ClientError::Protocol(format!(
            "Attendu recipe_answer, reçu: {:?}",
            other
        ))),
    }
}
