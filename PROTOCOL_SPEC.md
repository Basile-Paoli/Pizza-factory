# Messages relevés via Wireshark (convertis en JSON)

## UDP

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
