use crate::{TaggedSocketAddr, TaggedUuid};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub name: String,
    pub params: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload {
    pub order_id: TaggedUuid,
    pub order_timestamp: u64,
    pub delivery_host: TaggedSocketAddr,
    pub action_index: usize,
    pub action_sequence: Vec<Action>,
    pub content: String,
    pub updates: Vec<Update>,
}

impl Payload {
    pub fn now_micros() -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("L'horloge système est avant 1970")
            .as_micros() as u64
    }

    pub fn to_result_string(&self) -> String {
        serde_json::to_string(&serde_json::json!({
            "order_id": self.order_id.0.to_string(),
            "order_timestamp": self.order_timestamp,
            "delivery_host": self.delivery_host.0.to_string(),
            "action_index": self.action_index,
            "action_sequence": self.action_sequence,
            "content": self.content,
            "updates": self.updates,
        }))
        .expect("Failed to serialize Payload to JSON")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Update {
    Action {
        action: Action,
        timestamp: u64,
    },
    Forward {
        to: TaggedSocketAddr,
        timestamp: u64,
    },
    Deliver {
        timestamp: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TcpMessage {
    Order {
        recipe_name: String,
    },

    OrderDeclined {
        message: String,
    },

    ListRecipes {},

    GetRecipe {
        recipe_name: String,
    },

    OrderReceipt {
        order_id: TaggedUuid,
    },

    CompletedOrder {
        recipe_name: String,
        result: String,
    },

    RecipeListAnswer {
        recipes: HashMap<String, RecipeStatus>,
    },

    RecipeAnswer {
        recipe: String,
    },

    ProcessPayload {
        payload: Payload,
    },

    Deliver {
        payload: Payload,
        error: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipeStatus {
    Local(LocalRecipeStatus),
    Remote { host: TaggedSocketAddr },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalRecipeStatus {
    pub missing_actions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn payload_to_result_string_is_valid() {
        let payload = Payload {
            order_id: Uuid::nil().into(),
            order_timestamp: 12345,
            delivery_host: "127.0.0.1:8001"
                .parse::<std::net::SocketAddr>()
                .unwrap()
                .into(),
            action_index: 2,
            action_sequence: vec![
                Action {
                    name: "MakeDough".into(),
                    params: HashMap::new(),
                },
                Action {
                    name: "AddBase".into(),
                    params: [("base_type".to_string(), "tomato".to_string())].into(),
                },
            ],
            content: "Dough\nBase(tomato)\n".into(),
            updates: vec![Update::Action {
                action: Action {
                    name: "MakeDough".into(),
                    params: HashMap::new(),
                },
                timestamp: 100,
            }],
        };
        let s = payload.to_result_string();
        assert!(s.contains("\"order_timestamp\":12345"));
        assert!(s.contains("\"action_index\":2"));
        assert!(s.contains("MakeDough"));
    }
}
