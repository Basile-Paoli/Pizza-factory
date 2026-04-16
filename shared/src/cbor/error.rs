use thiserror::Error;

#[derive(Debug, Error)]
pub enum CborError {
    #[error("tag CBOR attendu {expected}, reçu {got}")]
    WrongTag { expected: u64, got: u64 },

    #[error("valeur CBOR invalide : {context}")]
    InvalidValue { context: &'static str },

    #[error("adresse réseau invalide : {0}")]
    InvalidSocketAddr(#[from] std::net::AddrParseError),

    #[error("valeur CBOR non-tagguée reçue, un tag était attendu")]
    NotTagged,

    #[error("clé CBOR manquante dans la map : {key}")]
    MissingKey { key: i64 },

    #[error("erreur d'encodage CBOR : {0}")]
    EncodeError(String),

    #[error("erreur de décodage CBOR : {0}")]
    DecodeError(String),
}
