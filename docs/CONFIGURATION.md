# Configuration reference

## Location

Default config: `~/.config/bandhop/config.toml`. Respects absolute `XDG_CONFIG_HOME`. The state file is `~/.local/state/bandhop/state` by default and may be relocated using `XDG_STATE_HOME`.

```toml
[playback]
volume = 60

[notifications]
enabled = true
on_play = true
on_stop = true
on_change = true

[[stations]]
id = "npr"
name = "NPR Example"
url = "https://DIRECT-STREAM-URL"

[[stations]]
id = "bbc"
name = "BBC Example"
url = "https://DIRECT-STREAM-URL"
```

**Note:** Example URLs are placeholders, not verified radio streams. Paste a working HTTP(S) media stream URL.

## Supported fields

| Section | Field | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `[playback]` | `volume` | integer 0–100 | 60 | mpv start volume |
| `[notifications]` | `enabled` | boolean | true | Master notification toggle |
| `[notifications]` | `on_play` | boolean | true | Notify when starting playback |
| `[notifications]` | `on_stop` | boolean | true | Notify when stopping or stream ends |
| `[notifications]` | `on_change` | boolean | true | Notify on station selection |
| `[[stations]]` | `id` | lowercase slug | required | Unique identifier `[a-z0-9-]+` |
| `[[stations]]` | `name` | quoted string | required | Display name |
| `[[stations]]` | `url` | quoted HTTP(S) URL | required | Direct stream URL |

Parser scope: only these sections/fields, standard double-quoted strings with `\\`, `\"`, `\n`, `\r` and `\t` escapes, and whole-line `#` comments. **It is not a complete TOML parser.** Unknown sections and fields are errors. Use `bandhop config validate` after hand-editing and `bandhop config reload` to apply to a running instance. `bandhop stations add/remove/move` saves and attempts a reload automatically.

No station is included by default because stream URLs may change and must be verified by the broadcaster.
