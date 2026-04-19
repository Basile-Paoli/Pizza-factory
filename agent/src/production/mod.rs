//! Serveur TCP de production — cœur de la chaîne de fabrication des pizzas.
//!
//! Ce module démarre un [`TcpListener`] sur le port de l'agent et gère chaque
//! connexion entrante dans un thread dédié.

mod actions;
mod handler;

pub use handler::{AgentContext, handle_connection};

use std::net::TcpListener;
use std::sync::Arc;
use std::thread;

/// Démarre le serveur TCP de production en arrière-plan.
///
/// Pour chaque connexion entrante, un nouveau thread est créé pour la traiter.
/// Le contexte [`AgentContext`] est partagé entre tous ces threads via `Arc`.
///
/// # Erreurs
///
/// Retourne une erreur si le port TCP est déjà occupé ou si le bind échoue.
pub fn start_production_server(
    ctx: Arc<AgentContext>,
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(ctx.addr)?;
    eprintln!("[agent] Serveur de production TCP démarré sur {}", ctx.addr);

    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(conn) => {
                    let ctx_clone = Arc::clone(&ctx);
                    thread::spawn(move || {
                        handle_connection(conn, ctx_clone);
                    });
                }
                Err(e) => {
                    eprintln!("[agent] Erreur d'acceptation de connexion: {}", e);
                }
            }
        }
    });

    Ok(())
}
