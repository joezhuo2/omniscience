# Integrations

Integrations let other programs and devices read Omniscience's numbers. All of them are off by
default. Turn them on in Settings (gear icon) > **Integrations**.

| Integration  | What it is for                                                        |
| ------------ | --------------------------------------------------------------------- |
| Local API    | Scripts, Prometheus, a phone view, the command line, Stream Deck.     |
| MQTT         | Sending readings to Home Assistant or any other MQTT broker.          |
| Custom cards | Your own cards on the dashboard. See [custom cards](custom-cards.md). |

## Local API

Turn on **Local API** and choose a port (default 9124). While it is on, this PC serves the
endpoints below on `127.0.0.1` only. A token is created when the API is turned on.

### Access rules

- A program on this PC (for example `curl` or a script) needs no token.
- A request that comes from a web page or an embedded browser has an `Origin` header. It must pass
  the token as `?token=<token>` or as `Authorization: Bearer <token>`.
- Devices elsewhere on the network always need the token (see
  [Phone and tablet view](#phone-and-tablet-view)).

### Endpoints

| Endpoint          | Returns                                           |
| ----------------- | ------------------------------------------------- |
| `GET /api/status` | The current readings as JSON.                     |
| `GET /metrics`    | The same readings as Prometheus text.             |
| `GET /`           | A small page that shows the readings, for phones. |

Process names, IP addresses and the Wi-Fi name are never included.

### `GET /api/status`

```json
{
  "app": "omniscience",
  "version": "0.11.0",
  "timestamp_ms": 1767225600000,
  "core": {
    "cpu": { "percent": 12.5 },
    "cpu_temp_c": 54.0,
    "ram": { "used_bytes": 8589934592, "total_bytes": 17179869184, "percent": 50.0 },
    "igpu": { "percent": 3.0 },
    "dgpu": { "percent": null },
    "igpu_temp_c": null,
    "dgpu_temp_c": null,
    "disk": { "percent": 41.0 },
    "net": { "down_bps": 125000, "up_bps": 8000 },
    "package_watts": 18.2
  },
  "detail": null
}
```

A reading this machine does not have is `null`. `detail` is only filled in while the dashboard is
open; otherwise it is `null`. The values above are only an example.

From a terminal on this PC:

```bash
curl http://127.0.0.1:9124/api/status
```

From a web page, pass the token:

```bash
curl "http://127.0.0.1:9124/api/status?token=YOUR_TOKEN"
curl -H "Authorization: Bearer YOUR_TOKEN" http://127.0.0.1:9124/api/status
```

### `GET /metrics`

Prometheus text format. Every gauge is named `omniscience_...`. Example scrape config:

```yaml
scrape_configs:
  - job_name: omniscience
    scrape_interval: 15s
    static_configs:
      - targets: ['127.0.0.1:9124']
```

Prometheus does not send an `Origin` header, so a Prometheus on the same PC needs no token. For
Prometheus on another machine, turn on **Allow other devices on this network** and add the token
to the scrape config, for example:

```yaml
authorization:
  type: Bearer
  credentials: YOUR_TOKEN
```

## Phone and tablet view

Turn on **Allow other devices on this network**. The API then also listens on the network, and
the settings show an address like `http://192.168.1.20:9124/?token=...`. Open it on a phone or
tablet on the same network.

Safety notes:

- Anyone with that address can read the values.
- **New token** invalidates every old address. Use it if an address was shared by mistake.
- The connection is plain HTTP. It is meant for a home network, not for the internet or a shared
  network you do not trust.
- Windows may ask whether to allow Omniscience through the firewall the first time.

## Command line

```bash
omniscience status
omniscience status --json
```

`status` prints the current readings in the terminal. `--json` prints the same JSON as
`/api/status`. The command asks the running app through the local API when that is on. Otherwise
it takes a reading itself.

## MQTT

Turn on **MQTT** to publish readings to a broker such as Mosquitto.

| Setting                  | Default       | Notes                                                |
| ------------------------ | ------------- | ---------------------------------------------------- |
| Host                     |               | The broker's address.                                |
| Port                     | 1883          |                                                      |
| Username / password      |               | Optional.                                            |
| Topic                    | `omniscience` | Prefix for all topics.                               |
| Publish interval         | 10 s          |                                                      |
| Home Assistant discovery | On            | Creates the sensors in Home Assistant automatically. |

The username and password are stored in the settings file on this PC, unencrypted. The connection
is plain TCP with no TLS, so use it on a trusted network.

### Topics

- `<topic>/state`: one retained JSON object with the latest readings.
- `<topic>/availability`: `online` while Omniscience is running, `offline` when it is not.

Fields in `<topic>/state`: `cpu_percent`, `cpu_temp_c`, `memory_percent`, `memory_used_bytes`,
`memory_total_bytes`, `igpu_percent`, `dgpu_percent`, `igpu_temp_c`, `dgpu_temp_c`,
`disk_percent`, `net_down_bps`, `net_up_bps`, `package_watts`.

### Home Assistant

With **Home Assistant discovery** on, Home Assistant shows a device called "Omniscience" with one
sensor per field. Nothing else is needed, as long as Home Assistant uses the same broker.

If you turned discovery off, add sensors by hand. For example, in `configuration.yaml`:

```yaml
mqtt:
  sensor:
    - name: 'PC CPU'
      state_topic: 'omniscience/state'
      availability_topic: 'omniscience/availability'
      value_template: '{{ value_json.cpu_percent }}'
      unit_of_measurement: '%'
    - name: 'PC CPU temperature'
      state_topic: 'omniscience/state'
      availability_topic: 'omniscience/availability'
      value_template: '{{ value_json.cpu_temp_c }}'
      unit_of_measurement: '°C'
```

Change `omniscience` if you chose another topic. Add one entry per field you want.

## Custom cards

Your own cards on the dashboard, filled from the readings or from a local JSON source. See
[custom cards](custom-cards.md).

## Stream Deck

A plugin in `integrations/streamdeck/` shows one reading on a Stream Deck key. It needs the Local
API and its token. Install steps are in its
[README](../integrations/streamdeck/README.md).

**The plugin has not been tested on a real Stream Deck yet.**
