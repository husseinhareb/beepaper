# wallselect

`wallselect` is a small Rust CLI for wallpaper discovery and selection.

This initial version intentionally focuses on local file discovery, configuration,
selection logic, and persisted state. It does not set wallpapers or integrate
with Wayland compositors yet.

## Commands

- `scan`: scan configured directories and persist the discovered image files
- `random`: pick a random wallpaper from the last scan result, avoiding recent repeats
- `history`: print recent selections
- `show-config`: print the resolved configuration after applying defaults and CLI overrides
- `init-config`: create a default TOML config file if one does not already exist

## Configuration

The default config file lives under the user config directory:

- Linux example: `~/.config/wallselect/config.toml`

The default state file lives under the user data directory:

- Linux example: `~/.local/share/wallselect/state.toml`

Example config:

```toml
dirs = ["/home/user/Pictures/Wallpapers"]
recursive = true
extensions = ["jpg", "jpeg", "png", "webp"]
history_size = 50
random_no_repeat_window = 5
```

## Project Structure

- `src/main.rs`: command dispatch and user-facing flow
- `src/cli.rs`: clap-derived CLI definitions and config override parsing
- `src/config.rs`: config defaults, path resolution, loading, and initialization
- `src/scanner.rs`: directory walking and image extension filtering
- `src/selector.rs`: random selection with no-repeat handling
- `src/history.rs`: history push, trim, and recent-view helpers
- `src/state.rs`: persisted scan results and selection history
- `src/error.rs`: domain-specific error types
- `src/lib.rs`: library module exports

## Development

```bash
cargo test
cargo run -- init-config
cargo run -- scan
cargo run -- random
```
