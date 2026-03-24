# Module CBOR - Documentation

## `shared/src/cbor/mod.rs`

### Module CBOR - types tagues du protocole Pizza-Factory

Ce module regroupe tout ce qui concerne la serialisation/deserialisation
 des valeurs CBOR speciales du protocole (tags `37`, `260`, `1001`).

### Sous-modules

| Sous-module   | Role                                              |
|---------------|---------------------------------------------------|
| `types`       | Structs Rust typees (`TaggedUuid`, etc.)          |
| `error`       | Type d'erreur `CborError`                         |
| `decode`      | Fonctions `decode_*` : `Value` -> struct          |
| `encode`      | Fonctions `encode_*` : struct -> `Value`          |
| `macros`      | Macros `cbor_encode_tagged!`, etc.                |

### Usage rapide

```rust,ignore
use shared::cbor::{
    decode::{decode_timestamp, from_bytes},
    encode::{encode_timestamp, to_bytes},
    types::TaggedTimestamp,
};

let ts = TaggedTimestamp::new(1769778234, 895985);
let bytes = to_bytes(&encode_timestamp(&ts)).unwrap();
let decoded = decode_timestamp(from_bytes(&bytes).unwrap()).unwrap();
assert_eq!(ts, decoded);
```

## `shared/src/cbor/types.rs`

### Types CBOR tagues du protocole Pizza-Factory

Ce module definit les types Rust correspondant aux valeurs CBOR tagguees
 utilisees dans le protocole (UDP gossip + TCP production).

### Correspondance tag -> type

| Tag CBOR | Rust type            | Exemple                                  |
|----------|----------------------|------------------------------------------|
| `37`     | `TaggedUuid`         | `"299defcb-c217-40e7-9030-af8debf647c6"` |
| `260`    | `TaggedSocketAddr`   | `"127.0.0.1:8001"`                       |
| `1001`   | `TaggedTimestamp`    | `{ "1": 1769778234, "-6": 895985 }`     |

### Structure CBOR sur le fil

Chaque valeur tagguee est encodee comme :

```text
tag(<N>, <valeur>)
```

où `<valeur>` depend du type (voir doc de chaque struct).

### Constantes de tags CBOR

- `TAG_UUID`: Tag CBOR pour un UUID RFC 4122 (16 octets bruts).
- `TAG_SOCKET_ADDR`: Tag CBOR pour une adresse reseau (`ip:port`) encodee en UTF-8.
- `TAG_TIMESTAMP`: Tag CBOR pour un timestamp micro-secondes (map avec cles `1` et `-6`).

### `TaggedUuid` (tag 37)

UUID RFC 4122 transporte avec le tag CBOR `37`.

#### Format CBOR sur le fil

```text
tag(37, bstr(16 octets big-endian))
```

#### Exemple JSON (representation protocole)

```json
{ "value": "299defcb-c217-40e7-9030-af8debf647c6", "tag": 37 }
```

### `TaggedSocketAddr` (tag 260)

Adresse reseau (`ip:port`) transportee avec le tag CBOR `260`.

#### Format CBOR sur le fil

```text
tag(260, tstr("127.0.0.1:8001"))
```

#### Exemple JSON (representation protocole)

```json
{ "value": "127.0.0.1:8001", "tag": 260 }
```

### `TaggedTimestamp` (tag 1001)

Timestamp haute-resolution transporte avec le tag CBOR `1001`.

La valeur interne est une **map CBOR** avec deux cles entieres :

| Cle CBOR | Champ Rust      | Signification                         |
|----------|-----------------|---------------------------------------|
| `1`      | `seconds`       | Secondes depuis l'epoque Unix         |
| `-6`     | `microseconds`  | Microsecondes (0 - 999 999)           |

#### Format CBOR sur le fil

```text
tag(1001, {1: <i64>, -6: <i64>})
```

#### Exemple JSON (representation protocole)

```json
{
  "value": { "1": 1769778234, "-6": 895985 },
  "tag": 1001
}
```

#### Exemple de valeur totale en microsecondes

```text
total_us = seconds * 1_000_000 + microseconds
```

Champs documentes :
- `seconds`: Secondes depuis l'epoque Unix (cle CBOR `1`).
- `microseconds`: Microsecondes (cle CBOR `-6`), dans l'intervalle [0, 999_999].

Methodes documentees :
- `to_micros`: Convertit en microsecondes totales depuis l'epoque Unix.
- `from_micros`: Construit depuis un nombre de microsecondes depuis l'epoque Unix.

## `shared/src/cbor/macros.rs`

### Macros utilitaires pour la serialisation/deserialisation CBOR tagguee

Ces macros generent le code boilerplate pour encoder et decoder un type
 Rust vers/depuis une valeur `ciborium::Value` enveloppee dans un tag CBOR.

### Macros disponibles

| Macro                   | Role                                                      |
|-------------------------|-----------------------------------------------------------|
| `cbor_encode_tagged!`   | Emballe une `ciborium::Value` dans `tag(N, value)`        |
| `cbor_decode_check_tag!`| Verifie que le tag est correct et extrait la valeur interne|
| `cbor_map_get_int!`     | Extrait une valeur d'une map CBOR par cle entiere         |

### `cbor_encode_tagged!`

Emballe une valeur `ciborium::Value` dans un tag CBOR.

```rust,ignore
let tagged = cbor_encode_tagged!(TAG_UUID, ciborium::Value::Bytes(bytes));
// -> ciborium::Value::Tag(37, Box::new(ciborium::Value::Bytes(bytes)))
```

### `cbor_decode_check_tag!`

Verifie qu'une `ciborium::Value` est un tag avec la valeur attendue,
 puis retourne la valeur interne par destructuration.

Retourne `Err(CborError::NotTagged)` si la valeur n'est pas un tag,
 et `Err(CborError::WrongTag { expected, got })` si le tag est different.

```rust,ignore
let inner = cbor_decode_check_tag!(value, TAG_UUID)?;
```

### `cbor_map_get_int!`

Extrait la valeur associee a une cle entiere `$key` (`i64`) dans une map
 `ciborium::Value::Map`.

Retourne `Err(CborError::MissingKey { key })` si la cle est absente.
Le parametre `$pairs` doit etre un `Vec<(ciborium::Value, ciborium::Value)>`.

```rust,ignore
let seconds_val = cbor_map_get_int!(pairs, 1_i64)?;
let us_val      = cbor_map_get_int!(pairs, -6_i64)?;
```

## `shared/src/cbor/error.rs`

### Erreurs de decodage / encodage CBOR

Erreur retournee par les fonctions de decodage et d'encodage CBOR.

#### Variantes de `CborError`

- `WrongTag { expected, got }`:
  - Le tag CBOR recu ne correspond pas au type attendu.
  - `expected` : tag attendu, `got` : tag recu.
- `InvalidValue { context }`:
  - La valeur interne du tag n'a pas le type ciborium attendu.
  - `context` decrit ce qui etait attendu (ex: `"bstr 16 octets pour UUID"`).
- `InvalidSocketAddr`:
  - Erreur lors du parsing d'une `SocketAddr` depuis une chaine.
- `NotTagged`:
  - La valeur n'etait pas un type tague (pas de `ciborium::Value::Tag`).
- `MissingKey { key }`:
  - Cle entiere manquante dans une map CBOR.
- `EncodeError`:
  - Erreur d'encodage ciborium.
- `DecodeError`:
  - Erreur de decodage ciborium.

## `shared/src/cbor/decode.rs`

### Fonctions de decodage des valeurs CBOR tagguees du protocole

Chaque fonction prend une `ciborium::Value` brute (issue de la
 deserialisation du flux binaire) et la convertit en type Rust type.

### Fonctions disponibles

| Fonction               | Tag attendu | Type retourne        |
|------------------------|-------------|----------------------|
| `decode_uuid`          | `37`        | `TaggedUuid`         |
| `decode_socket_addr`   | `260`       | `TaggedSocketAddr`   |
| `decode_timestamp`     | `1001`      | `TaggedTimestamp`    |

### Pipeline de decodage

```text
bytes bruts
   |
   v  ciborium::from_reader / ciborium::de::from_reader
ciborium::Value
   |
   v  decode_uuid / decode_socket_addr / decode_timestamp
TaggedUuid / TaggedSocketAddr / TaggedTimestamp
```

### `decode_uuid`

Decode un `TaggedUuid` depuis une `ciborium::Value`.

#### Format attendu

```text
tag(37, bstr(16 octets UUID big-endian))
```

#### Erreurs

- `CborError::NotTagged` si `value` n'est pas un `Value::Tag`.
- `CborError::WrongTag` si le tag est different de `37`.
- `CborError::InvalidValue` si la valeur interne n'est pas `bstr` de 16 octets.

### `decode_socket_addr`

Decode un `TaggedSocketAddr` depuis une `ciborium::Value`.

#### Format attendu

```text
tag(260, tstr("ip:port"))
```

#### Erreurs

- `CborError::NotTagged` si `value` n'est pas un `Value::Tag`.
- `CborError::WrongTag` si le tag est different de `260`.
- `CborError::InvalidValue` si la valeur interne n'est pas une `tstr`.
- `CborError::InvalidSocketAddr` si la chaine ne parse pas en `SocketAddr`.

### `decode_timestamp`

Decode un `TaggedTimestamp` depuis une `ciborium::Value`.

#### Format attendu

```text
tag(1001, {1: <integer>, -6: <integer>})
```

La map doit contenir exactement les cles entieres `1` (secondes) et `-6`
(microsecondes). Les valeurs sont des entiers CBOR signes.

#### Erreurs

- `CborError::NotTagged` si `value` n'est pas un `Value::Tag`.
- `CborError::WrongTag` si le tag est different de `1001`.
- `CborError::InvalidValue` si la valeur interne n'est pas une map.
- `CborError::MissingKey` si la cle `1` ou `-6` est absente.
- `CborError::InvalidValue` si une valeur n'est pas un entier valide.

### Helpers internes

- `extract_integer`: Extrait un `i64` depuis un `ciborium::Value::Integer`.

### Decodage depuis bytes bruts (framing TCP)

- `from_bytes`: Decode un `ciborium::Value` depuis un slice d'octets bruts CBOR.
  Utilise comme premiere etape avant d'appeler `decode_uuid`,
  `decode_socket_addr` ou `decode_timestamp`.

#### Erreurs

- `CborError::DecodeError` si les octets ne sont pas du CBOR valide.

### Documentation tests

- `make_timestamp_value`: Construit un `tag(1001, {1: sec, -6: us})` pour les tests.

## `shared/src/cbor/encode.rs`

### Fonctions d'encodage des types Rust typees vers `ciborium::Value` tagguee

Symetrique de `crate::cbor::decode`, chaque fonction prend un type Rust
 et retourne un `ciborium::Value` pret a etre serialise sur le fil.

### Fonctions disponibles

| Fonction               | Tag produit | Entree               |
|------------------------|-------------|----------------------|
| `encode_uuid`          | `37`        | `TaggedUuid`         |
| `encode_socket_addr`   | `260`       | `TaggedSocketAddr`   |
| `encode_timestamp`     | `1001`      | `TaggedTimestamp`    |

### Pipeline d'encodage

```text
TaggedUuid / TaggedSocketAddr / TaggedTimestamp
   |
   v  encode_uuid / encode_socket_addr / encode_timestamp
ciborium::Value
   |
   v  ciborium::into_writer
bytes bruts
```

### `encode_uuid`

Encode un `TaggedUuid` en `ciborium::Value`.

#### Format produit

```text
tag(37, bstr(16 octets UUID big-endian))
```

### `encode_socket_addr`

Encode un `TaggedSocketAddr` en `ciborium::Value`.

#### Format produit

```text
tag(260, tstr("ip:port"))
```

### `encode_timestamp`

Encode un `TaggedTimestamp` en `ciborium::Value`.

#### Format produit

```text
tag(1001, {1: <i64>, -6: <i64>})
```

Les cles de la map sont des entiers CBOR signes (`1` et `-6`).

### Encodage vers bytes bruts (framing TCP)

- `to_bytes`: Serialise un `ciborium::Value` vers un vecteur d'octets CBOR.

#### Erreurs

- `CborError::EncodeError` si la serialisation echoue.
