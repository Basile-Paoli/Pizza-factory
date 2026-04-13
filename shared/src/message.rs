use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;
use crate::{TaggedSocketAddr, TaggedUuid};

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
        let seq: Vec<String> = self.action_sequence.iter().map(action_to_json).collect();
        let upd: Vec<String> = self.updates.iter().map(update_to_json).collect();

        format!(
            "{{\"order_id\":{{\"value\":\"{}\",\"tag\":37}},\
             \"order_timestamp\":{},\
             \"delivery_host\":{{\"value\":\"{}\",\"tag\":260}},\
             \"action_index\":{},\
             \"action_sequence\":[{}],\
             \"content\":{},\
             \"updates\":[{}]}}",
            self.order_id.0,
            self.order_timestamp,
            self.delivery_host.0,
            self.action_index,
            seq.join(","),
            json_string(&self.content),
            upd.join(","),
        )
    }
}

fn action_to_json(a: &Action) -> String {
    format!(
        "{{\"name\":{},\"params\":{}}}",
        json_string(&a.name),
        params_to_json(&a.params)
    )
}

fn params_to_json(params: &HashMap<String, String>) -> String {
    if params.is_empty() {
        return "{}".to_string();
    }
    let pairs: Vec<String> = params
        .iter()
        .map(|(k, v)| format!("{}:{}", json_string(k), json_string(v)))
        .collect();
    format!("{{{}}}", pairs.join(","))
}

fn update_to_json(u: &Update) -> String {
    match u {
        Update::Action { action, timestamp } => format!(
            "{{\"Action\":{{\"action\":{},\"timestamp\":{}}}}}",
            action_to_json(action),
            timestamp
        ),
        Update::Forward { to, timestamp } => format!(
            "{{\"Forward\":{{\"to\":{{\"value\":\"{}\",\"tag\":260}},\"timestamp\":{}}}}}",
            to.0, timestamp
        ),
        Update::Deliver { timestamp } => {
            format!("{{\"Deliver\":{{\"timestamp\":{}}}}}", timestamp)
        }
    }
}

fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Update {
    Action { action: Action, timestamp: u64 },
    Forward { to: TaggedSocketAddr, timestamp: u64 },
    Deliver { timestamp: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TcpMessage {

    Order { recipe_name: String },

    ListRecipes {},

    GetRecipe { recipe_name: String },

    OrderReceipt { order_id: TaggedUuid },

    CompletedOrder { recipe_name: String, result: String },

    RecipeListAnswer { recipes: HashMap<String, RecipeStatus> },

    RecipeAnswer { recipe: String },

    ProcessPayload { payload: Payload },

    Deliver {
        payload: Payload,
        error: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeStatus {
    pub local: LocalRecipeStatus,
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
            delivery_host: "127.0.0.1:8001".parse::<std::net::SocketAddr>().unwrap().into(),
            action_index: 2,
            action_sequence: vec![
                Action { name: "MakeDough".into(), params: HashMap::new() },
                Action {
                    name: "AddBase".into(),
                    params: [("base_type".to_string(), "tomato".to_string())].into(),
                },
            ],
            content: "Dough\nBase(tomato)\n".into(),
            updates: vec![
                Update::Action {
                    action: Action { name: "MakeDough".into(), params: HashMap::new() },
                    timestamp: 100,
                },
            ],
        };
        let s = payload.to_result_string();
        assert!(s.contains("\"order_timestamp\":12345"));
        assert!(s.contains("\"action_index\":2"));
        assert!(s.contains("MakeDough"));
    }
}
