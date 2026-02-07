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
