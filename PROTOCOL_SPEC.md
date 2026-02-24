# Messages relevés via Wireshark (convertis en JSON)

## UDP (Protocole gossip)

Chaque noeud possède une liste de compétences (capabilities) et de recettes (recipes).
Chaque noeud possède une liste des autres noeuds présents sur le réseau avec leurs compétences et recettes.
Chaque noeud garde une trace de sa propre version, et de la dernière version connue des autres noeuds.
Chaque noeud garde une trace de la date du dernier message reçu de la part de chaque pair.

Lorsqu'un noeud reçoit une nouvelle information, il augmente sa version, puis envoie une message de gossip "Announce" aux autres noeuds qui ont une version plus basse (selon son gossip rate).
Chaque message "Announce" est suivi d'un message "Announce" dans le sens inverse avec l'information du noeud destinataire mise à jour.

Chaque noeud échange des pings à intervalle régulier avec chacun de ses pairs.
Si le ping n'atteint pas son destinataire pendant un certain temps (10s par défaut), le pair est oublié.
Le ping contient la version du noeud, ce qui permet de savoir si un voisin a une version plus basse (i.e. il n'a pas reçu un message de gossip pour des raisons d'erreur réseau ou autre)

### "Announce"

```json
{
  "Announce": {
    "node_addr": {
      "value": "127.0.0.1:8001",
      "tag": 260
    },
    "capabilities": [],
    "recipes": [],
    "peers": [
      {
        "value": "127.0.0.1:8000",
        "tag": 260
      },
      {
        "value": "127.0.0.1:8002",
        "tag": 260
      }
    ],
    "version": {
      "counter": 2,
      "generation": 1769778718
    }
  }
}
```

version:

- "counter": Incrémenté à chaque update des informations
- "generation" : Timestamp de la version actuelle

### "Ping"

```json
{
  "Ping": {
    "last_seen": {
      "value": {
        "1": 1769778234,
        "-6": 895985
      },
      "tag": 1001
    },
    "version": {
      "counter": 1,
      "generation": 1769778201
    }
  }
}
```

"last_seen.value":

- "1": timestamp en secondes
- "-6": microsecondes

### "Pong"

```json
{
  "Pong": {
    "last_seen": {
      "value": {
        "1": 1769778235,
        "-6": 398575
      },
      "tag": 1001
    },
    "version": {
      "counter": 1,
      "generation": 1769778108
    }
  }
}
```


## TCP (Protocole de production)

### Framing

Chaque message TCP est précédé de 4 octets en **big-endian** indiquant la taille du payload CBOR qui suit :

```
┌──────────────────────────┬───────────────────────────┐
│  LENGTH (4 bytes, BE u32) │  CBOR PAYLOAD (N bytes)  │
└──────────────────────────┴───────────────────────────┘
```

Une **nouvelle connexion TCP est ouverte pour chaque message** (pas de connexion persistante).

### Types CBOR spéciaux

| Tag CBOR | Type          | Exemple                                  |
|----------|---------------|------------------------------------------|
| `37`     | UUID RFC 4122 | `"299defcb-c217-40e7-9030-af8debf647c6"` |
| `260`    | SocketAddr    | `"127.0.0.1:8001"`                       |

---

## Messages Client → Agent

### `order`

Passe une commande de pizza par nom de recette.

```json
{
  "order": {
    "recipe_name": "Pepperoni"
  }
}
```

### `list_recipes`

Demande la liste de toutes les recettes connues de l'agent, avec leur statut (réalisable ou non).

```json
{
  "list_recipes": {}
}
```

### `get_recipe`

Demande la définition DSL d'une recette spécifique.

```json
{
  "get_recipe": {
    "recipe_name": "Pepperoni"
  }
}
```

---

## Messages Agent → Client

### `order_receipt`

Accusé de réception immédiat après un `order`. Contient l'UUID attribué à la commande.
Le client reste connecté et attend ensuite le message `completed_order` sur la même connexion.

```json
{
  "order_receipt": {
    "order_id": {
      "value": "299defcb-c217-40e7-9030-af8debf647c6",
      "tag": 37
    }
  }
}
```

### `completed_order`

Envoyé sur la même connexion que `order_receipt`, une fois la pizza entièrement produite et livrée.
Le champ `result` est une **string JSON sérialisée** contenant le payload final.

```json
{
  "completed_order": {
    "recipe_name": "Margherita",
    "result": "{\"order_id\":{...},\"order_timestamp\":1771666866903601,\"content\":\"Dough + Base(tomato): ready\\nCheese x2\\n...\",\"updates\":[...]}"
  }
}
```

### `recipe_list_answer`

Réponse à `list_recipes`. Chaque recette indique les actions manquantes (vide si réalisable).

```json
{
  "recipe_list_answer": {
    "recipes": {
      "Pepperoni": {
        "local": {
          "missing_actions": []
        }
      },
      "Funghi": {
        "local": {
          "missing_actions": [
            "AddMushrooms"
          ]
        }
      },
      "Marinara": {
        "local": {
          "missing_actions": [
            "AddGarlic",
            "AddOregano"
          ]
        }
      }
    }
  }
}
```

### `recipe_answer`

Réponse à `get_recipe`. Contient la définition DSL brute de la recette.

```json
{
  "recipe_answer": {
    "recipe": "Pepperoni = MakeDough -> AddBase(base_type=tomato) -> AddCheese(amount=2) -> AddPepperoni(slices=12) -> Bake(duration=6)"
  }
}
```

---

## Messages Agent → Agent

### `process_payload`

Message principal de la chaîne de production. Transmis d'agent en agent à chaque étape.
Contient la recette complète, l'index de l'action courante, l'état actuel de la pizza (`content`), et l'historique des actions effectuées (`updates`).

- `action_index` : index de la prochaine action à exécuter dans `action_sequence` (0-based).
- `delivery_host` : adresse de l'agent qui doit recevoir le `deliver` final (en général l'agent qui a reçu l'`order`).
- `content` : chaîne de caractères décrivant l'état de la pizza, accumulée au fil des étapes.
- `updates` : journal de toutes les actions et transferts effectués depuis le début.

```json
{
  "process_payload": {
    "payload": {
      "order_id": {
        "value": "299defcb-c217-40e7-9030-af8debf647c6",
        "tag": 37
      },
      "order_timestamp": 1771666866903601,
      "delivery_host": {
        "value": "127.0.0.1:8001",
        "tag": 260
      },
      "action_index": 0,
      "action_sequence": [
        { "name": "MakeDough",    "params": {} },
        { "name": "AddBase",      "params": { "base_type": "tomato" } },
        { "name": "AddCheese",    "params": { "amount": "2" } },
        { "name": "AddBasil",     "params": { "leaves": "3" } },
        { "name": "Bake",         "params": { "duration": "5" } },
        { "name": "AddOliveOil",  "params": {} }
      ],
      "content": "",
      "updates": []
    }
  }
}
```

Exemple de `process_payload` à mi-production (`action_index` = 2, `content` et `updates` remplis) :

```json
{
  "process_payload": {
    "payload": {
      "order_id": { "value": "299defcb-c217-40e7-9030-af8debf647c6", "tag": 37 },
      "order_timestamp": 1771666866903601,
      "delivery_host": { "value": "127.0.0.1:8001", "tag": 260 },
      "action_index": 2,
      "action_sequence": [ "..." ],
      "content": "Dough + Base(tomato): ready\n",
      "updates": [
        {
          "Action": {
            "action": { "name": "MakeDough", "params": {} },
            "timestamp": 1771666866903844
          }
        },
        {
          "Forward": {
            "to": { "value": "127.0.0.1:8002", "tag": 260 },
            "timestamp": 1771666866904018
          }
        },
        {
          "Action": {
            "action": { "name": "AddBase", "params": { "base_type": "tomato" } },
            "timestamp": 1771666866904252
          }
        }
      ]
    }
  }
}
```

#### Entrées possibles dans `updates`

| Type      | Champs                               | Description                                      |
|-----------|--------------------------------------|--------------------------------------------------|
| `Action`  | `action: {name, params}`, `timestamp` | Une action exécutée localement                   |
| `Forward` | `to: SocketAddr`, `timestamp`         | Un transfert vers un autre agent                 |
| `Deliver` | `timestamp`                           | Marqueur de livraison finale (présent dans `completed_order`) |

### `deliver`

Envoyé par le dernier agent vers `delivery_host` lorsque toutes les actions de `action_sequence` sont terminées.
Le champ `error` est `null` en cas de succès, ou une string décrivant l'erreur.

```json
{
  "deliver": {
    "payload": {
      "order_id": { "value": "299defcb-c217-40e7-9030-af8debf647c6", "tag": 37 },
      "order_timestamp": 1771666866903601,
      "delivery_host": { "value": "127.0.0.1:8001", "tag": 260 },
      "action_index": 6,
      "action_sequence": [ "..." ],
      "content": "Dough + Base(tomato): ready\nCheese x2\nBasil leaves x3\nBaked(5)\nOlive Oil drizzled\n",
      "updates": [
        { "Action": { "action": { "name": "MakeDough",   "params": {} },                      "timestamp": 1771666866903844 } },
        { "Forward": { "to": { "value": "127.0.0.1:8002", "tag": 260 },                       "timestamp": 1771666866904018 } },
        { "Action": { "action": { "name": "AddBase",     "params": { "base_type": "tomato" } },"timestamp": 1771666866904252 } },
        { "Action": { "action": { "name": "AddCheese",   "params": { "amount": "2" } },        "timestamp": 1771666866904574 } },
        { "Action": { "action": { "name": "AddBasil",    "params": { "leaves": "3" } },        "timestamp": 1771666866904946 } },
        { "Action": { "action": { "name": "Bake",        "params": { "duration": "5" } },      "timestamp": 1771666866905252 } },
        { "Action": { "action": { "name": "AddOliveOil", "params": {} },                       "timestamp": 1771666866905528 } },
        { "Forward": { "to": { "value": "127.0.0.1:8001", "tag": 260 },                       "timestamp": 1771666866905762 } }
      ]
    },
    "error": null
  }
}
```

## Algorithme de routage de l'agent

```
reçoit process_payload(payload):
  action = payload.action_sequence[payload.action_index]

  si je connais action.name:
    payload.content += exécuter(action)
    payload.updates.append(Action { action, timestamp: now() })
    payload.action_index += 1

    si action_index >= len(action_sequence):
      payload.updates.append(Forward { to: delivery_host, timestamp: now() })
      envoyer deliver(payload, error=null) → payload.delivery_host
    sinon:
      envoyer process_payload(payload) → moi-même (nouvelle connexion)

  sinon:
    peer = chercher dans la table gossip un pair qui a action.name
    payload.updates.append(Forward { to: peer, timestamp: now() })
    envoyer process_payload(payload) → peer

reçoit deliver(payload, error):
  si error == null:
    envoyer completed_order(recipe_name, result=serialize(payload)) → client
  sinon:
    // gérer l'erreur
```
