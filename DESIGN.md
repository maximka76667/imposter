# Imposter

A cross-platform simulator that impersonates a fleet of physical PCBs on a local network. The backend connects to it exactly as it would to real hardware — same IPs, same ports, same packet format. One tool, many phantoms.

---

## What it is

Boards bind and listen. The backend initiates the connection. So each simulated board is actually a server in the transport sense — it binds to a specific IP (via network aliases) and waits for the backend to connect over TCP. UDP is used for continuous sensor data streaming outward to the backend address.

Each board has:

- One UDP socket — binds to `board_ip:0` (ephemeral local port), sends packets to `addresses.backend:ports.UDP` from `general_info.json`
- One TCP server — binds to `board_ip:ports.TCP_SERVER` from `general_info.json`

Socket configuration (`sockets.json`) is not used by imposter. All boards use the same ports and backend address from `general_info.json`.

The simulator reads the shared ADJ config format that already exists and is agreed upon between the real boards and the backend. It trusts that config as the source of truth — no domain validation beyond what serde gives for free. Structural errors (missing fields, wrong types) surface as hard faults pointing at the offending file.

Period (tick rate) and other simulation parameters not present in ADJ are defined in a separate `imposter.toml` file that imposter watches live.

---

## Config structure

### ADJ (read-only, shared with backend)

```
adj/
  general_info.json         ← ports, addresses, units, message_ids
  boards.json               ← map of board name → path to board config
  boards/
    {board_name}/
      {board_name}.json               ← board_id, board_ip, references to other files
      {board_name}_measurements.json  ← sensor definitions with types and ranges
      packets.json                    ← data packets (telemetry, type: "data")
      orders.json                     ← command packets (backend → board, type: "order")
```

### imposter.toml (imposter-owned)

```toml
default_period_ms = 100
enable_udp = false         # global kill-switch — overrides all per-board settings
enable_tcp = true
default_enable_udp = true  # per-board default when not explicitly set
default_enable_tcp = true

[boards.VCU]
period_ms = 100
enable_tcp = false         # UDP only

[boards.BCU]
period_ms = 200
enable_udp = false         # TCP only
```

`enable_udp` / `enable_tcp` at the top level are global kill-switches — if set, they override all per-board settings. `default_enable_udp` / `default_enable_tcp` are the fallback for boards that don't specify their own value. A board with both disabled starts but does nothing.

Watched live — changes are applied to running board actors without respawn. Board names are matched dynamically against `boards.json` — unknown names in `imposter.toml` are ignored, boards with no entry fall back to `default_period_ms` / `default_enable_udp` / `default_enable_tcp`.

---

## Config file formats

### general_info.json

```json
{
  "ports": { "TCP_SERVER": 50500, "UDP": 50400, ... },
  "addresses": { "backend": "127.0.0.9" },
  "units": { "string": "string" },
  "message_ids": { "string": "number" }
}
```

Imposter uses `ports.TCP_SERVER`, `ports.UDP`, and `addresses.backend`. The rest is parsed but unused.

### boards.json

Map of board name to path of its main config file:

```json
{
  "VCU": "boards/VCU/VCU.json",
  "BMSL": "boards/BMSL/BMSL.json"
}
```

### {board_name}.json

```json
{
  "board_id": 11,
  "board_ip": "127.0.0.3",
  "measurements": ["VCU_measurements.json"],
  "packets": ["orders.json", "packets.json"]
}
```

`packets` is a flat array of files containing any mix of `"data"` and `"order"` type packets. File paths are relative to the board's directory.

### {board_name}\_measurements.json

Array of measurement definitions. Packet variables reference measurement ids from here.

```json
[
  {
    "id": "brake_pressure",
    "name": "Brake Pressure",
    "type": "float32",
    "podUnits": "bar",
    "displayUnits": "bar",
    "safeRange": [0.0, 100.0],
    "warningRange": [80.0, 95.0]
  },
  {
    "id": "brake_status",
    "name": "Brake Status",
    "type": "enum",
    "enumValues": ["released", "engaged", "error"]
  }
]
```

Types: `uint8`, `uint16`, `uint32`, `uint64`, `int8`, `int16`, `int32`, `int64`, `float32`, `float64`, `enum`

`enum` is a distinct type — it carries `enumValues` and is not a numeric type with annotations. Random walk picks a random index each tick.

### packets.json / orders.json

Both use the same format. Distinguished by the `type` field: `"data"` for telemetry (board → backend), `"order"` for commands (backend → board).

```json
[
  {
    "id": 249,
    "type": "data",
    "name": "Current State",
    "variables": ["general_state", "operational_state"]
  },
  {
    "id": 502,
    "type": "order",
    "name": "Turn on PFM",
    "variables": []
  }
]
```

No period or socket field — period comes from `imposter.toml`, socket endpoints are derived from `general_info.json`.

---

## Config loading graph

```
general_info.json
boards.json → {board_name}.json → measurements files
                               → packet files (mix of data + order entries)
imposter.toml (separate, live-watched)
```

---

## CLI flags

| Flag              | Default                                         | Description                                 |
| ----------------- | ----------------------------------------------- | ------------------------------------------- |
| `--adj <path>`    | `os.userCacheDir/hyperloop-control-station/adj` | Path to ADJ config directory                |
| `--config <path>` | `<adj-parent>/imposter.toml`                    | Path to imposter config file                |
| `--dry-run`       | false                                           | Skip network alias setup (no root required) |

In dev: `imposter --adj ./adj --config ./imposter.toml`. In production: no flags needed.

---

## Boot sequence

```
1. Parse CLI flags
2. Read adj git branch → log it (requires no git installation, uses libgit2)
3. Load + parse general_info.json         → hard fault if malformed
4. Load + parse boards.json               → hard fault if malformed
5. For each board, load all referenced files → hard fault if any malformed
6. Load imposter.toml                     → hard fault if malformed
7. Setup network aliases (netsetup/)      → hard fault if privileges missing
8. Spawn board actors (one per board)
9. Start filesystem watcher (adj/ + imposter.toml)
10. Enter supervisor loop — wait for ctrl_c
```

---

## Per-board actor

Three concurrent things running independently inside each board's task:

**Tick loop** — ticks at the period configured in `imposter.toml` (or default). Each tick: nudges all measurement values with a random walk within their configured ranges, serializes `"data"` packets and fires them over UDP to `addresses.backend:ports.UDP`. If a measurement crossed a safe/warning range boundary this tick, builds an alert and tries to send it down the TCP writer channel — if no client is connected the alert is silently dropped.

**TCP listener** — always running on `board_ip:ports.TCP_SERVER`. When the backend connects, splits the stream and spawns two sub-tasks:

- Reader — parses incoming frames, matches against the board's `"order"` packets, sends a generic ack back through the writer channel
- Writer — owns the write half, drains the mpsc channel receiving acks and alerts

When the client disconnects, reader and writer die, alert sender becomes None, listener goes back to waiting. UDP is completely unaffected by TCP client state.

**Control receiver** — select!s alongside the tick loop and TCP listener. Receives commands from the watcher — period updates, mode/step changes, enable/disable UDP/TCP.

---

## Random value generation

Measurements have types and optional safeRange/warningRange. The random walk strategy:

- If safeRange is defined, generate initial value within it and walk within it
- If only warningRange, use that as bounds
- If no range, generate sensible defaults per type (0.0–1.0 for floats, 0 for ints)
- For `enum`, pick randomly from `enumValues` by index each tick
- Each tick: last value + small random delta, clamped to range

Pure random noise per tick is avoided — values drift naturally like real sensors.

---

## Network aliases

Managed automatically by `netsetup/` at startup and torn down at clean shutdown. Each board binds its UDP socket and TCP server to its configured `board_ip`.

Three platform implementations behind a common trait:

- Linux — `ip addr`
- macOS — `ifconfig`
- Windows — `netsh`

The `--dry-run` flag skips alias management entirely for development without root.

---

## Filesystem watcher

Watches the adj config directory and `imposter.toml` with debounce (~300ms). On modification:

- Any file under `adj/` changed → shut down entire fleet, tear down all aliases, reload everything, respawn. ADJ is not edited manually — a change means a coordinated config update, so a full restart is always correct.
- `imposter.toml` changed → reload periods, push new values to running board actors via control channel (no respawn)

---

## Wire protocol

Newline-delimited JSON over TCP and UDP. Packets are serialized from the board's `"data"` packet definitions, with measurement values generated by random walk. Orders arrive over TCP and are matched against the board's `"order"` packet catalog.

---

## Module map

```
src/
  main.rs          ← entry, supervisor loop, signal handling
  config.rs        ← load general_info, boards, and all referenced board files
  imposter_cfg.rs  ← load + watch imposter.toml, push period updates
  board.rs         ← BoardHandle, spawn(), CancellationToken
  state.rs         ← MeasurementState, RangeStatus, connected flag
  simulator.rs     ← tick loop, random walk, range evaluation, alert dispatch
  udp.rs           ← serialize packets, bind to board_ip:0, send to backend
  tcp.rs           ← listener, split stream, reader + writer tasks
  protocol.rs      ← OutboundMsg, InboundCmd, wire framing (newline JSON)
  alert.rs         ← RangeViolation builder
  watcher.rs       ← notify watcher, debounce, reload signals
  netsetup/
    mod.rs         ← AliasManager trait, factory, privilege check
    linux.rs
    macos.rs
    windows.rs
```

---

## What to ignore

- `sockets.json` files in board directories — not parsed, not used
- Domain validation (logical range errors, duplicate ids) — responsibility of external tooling, not imposter
- `podUnits` / `displayUnits` in measurements — parsed but not used at runtime
- `units` and `message_ids` in `general_info.json` — parsed but not used at runtime

---

## Where to start

Start with `config.rs`. Parse `general_info.json` and one board's full file tree into typed structs with serde, print them, done. No network, no async, no actors. Just "does my data model match reality."

Build order:

```
1. config.rs          — parse everything into structs
2. imposter_cfg.rs    — parse imposter.toml, define period defaults
3. Single board UDP   — no TCP, no actors, fire packets at backend
4. Add TCP listener   — handle one client connect/disconnect
5. Wrap in actor      — async, tokio tasks, tick loop + TCP + control receiver
6. Multi-board        — spawn N actors, prove isolation works
7. netsetup           — network aliases, platform implementations
8. watcher            — filesystem watcher, reload logic (adj + imposter.toml)
```
