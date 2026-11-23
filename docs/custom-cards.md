# Custom cards

A custom card is a small card on the dashboard that you define in a JSON file. It can show
Omniscience's own readings, or values fetched from a local web service.

Custom cards are not available when the dashboard is embedded in another app.

## Where the files go

In Settings > **Integrations**, choose **Open cards folder**. This opens `cards` inside the
app's settings folder. Each `*.json` file there is one card. Files are re-read every 30 seconds,
so a new or edited file appears without a restart.

## Format

```json
{
  "title": "Build server",
  "rows": [
    { "label": "CPU here", "value": "{core.cpu.percent|0}%" },
    { "label": "Queue", "value": "{data.queue.length}" }
  ],
  "source": { "url": "http://127.0.0.1:3000/stats", "refresh_s": 10 }
}
```

- `title`: the card heading.
- `rows`: the lines of the card. Each has a `label` and a `value`.
- `source`: optional. Where to fetch JSON for `data`. Leave it out for a telemetry-only card.

### Placeholders

Text in braces in a `value` is replaced with a reading.

| Placeholder | Meaning                                                        |
| ----------- | -------------------------------------------------------------- |
| `{path}`    | The value at a dotted path.                                    |
| `{path\|N}` | A number shown with N decimals, for example `{core.ram.percent | 1}`. |

The path starts with one of:

- `core`: Omniscience's main readings. Always available.
- `detail`: extra readings. Available while the dashboard is open.
- `data`: the JSON fetched from `source`.

A value that is missing shows `--`.

### Common `core` paths

| Path                   | Reading                          |
| ---------------------- | -------------------------------- |
| `core.cpu.percent`     | CPU use, %                       |
| `core.cpu_temp_c`      | CPU temperature, °C              |
| `core.ram.used_bytes`  | Memory in use, bytes             |
| `core.ram.total_bytes` | Installed memory, bytes          |
| `core.ram.percent`     | Memory use, %                    |
| `core.igpu.percent`    | Integrated GPU use, %            |
| `core.dgpu.percent`    | Discrete GPU use, %              |
| `core.igpu_temp_c`     | Integrated GPU temperature, °C   |
| `core.dgpu_temp_c`     | Discrete GPU temperature, °C     |
| `core.disk.percent`    | Disk use, %                      |
| `core.net.down_bps`    | Download speed, bytes per second |
| `core.net.up_bps`      | Upload speed, bytes per second   |
| `core.package_watts`   | CPU package power, W             |

The same names appear in the `core` object of `GET /api/status`. See
[integrations](integrations.md#get-apistatus).

## Examples

A card with no source:

```json
{
  "title": "Quick look",
  "rows": [
    { "label": "CPU", "value": "{core.cpu.percent|0}%" },
    { "label": "CPU temp", "value": "{core.cpu_temp_c|0} °C" },
    { "label": "Memory", "value": "{core.ram.percent|0}%" },
    { "label": "Power", "value": "{core.package_watts|1} W" }
  ]
}
```

A card that also reads a local service. If `http://127.0.0.1:3000/stats` returns
`{"queue": {"length": 4}, "status": "ok"}`:

```json
{
  "title": "Build server",
  "rows": [
    { "label": "CPU here", "value": "{core.cpu.percent|0}%" },
    { "label": "Queue", "value": "{data.queue.length}" },
    { "label": "Status", "value": "{data.status}" }
  ],
  "source": { "url": "http://127.0.0.1:3000/stats", "refresh_s": 10 }
}
```

## Limits

| Item         | Limit                                                           |
| ------------ | --------------------------------------------------------------- |
| Rows         | 12                                                              |
| Title        | 60 characters                                                   |
| Label        | 40 characters                                                   |
| Value        | 120 characters                                                  |
| File size    | 64 KB                                                           |
| `source.url` | Plain `http://` to this PC: `127.0.0.1`, `localhost` or `[::1]` |
| Source reply | JSON, up to 256 KB                                              |
| `refresh_s`  | 2 to 3600 seconds (default 10)                                  |

## Troubleshooting

**The card shows an error message.** The file has a mistake. Read the message, fix the file and
wait up to 30 seconds. Check that the JSON is valid, that no limit above is exceeded and that
`source.url` is plain `http://` to this PC.

**A value shows `--`.** The path points to nothing. Check the spelling against the tables above.
Other causes:

- The reading does not exist on this PC, for example `core.dgpu.percent` without a discrete GPU.
- A `detail` path was used while the dashboard is closed.
- A `data` path was used but the source has not answered yet, is not running, or returned JSON
  without that field. Open the source URL in a browser on this PC to see what it returns.
