# beepaper

`beepaper` is a small Rust CLI for wallpaper discovery, selection, and native
Wayland wallpaper application.

This version includes a first native Wayland MVP using layer-shell and
shared-memory buffers, while preserving the original focus on local file
discovery, configuration, selection logic, and persisted state.

## Commands

- `scan`: scan configured directories and persist the discovered image files
- `random`: pick a random wallpaper from the last scan result, avoiding recent repeats
- `random --apply`: select a wallpaper and apply it natively on Wayland
- `history`: print recent selections
- `show-config`: print the resolved configuration after applying defaults and CLI overrides
- `init-config`: create a default TOML config file if one does not already exist
- `apply <path>`: display a static wallpaper image natively on Wayland

## Configuration

The default config file lives under the user config directory:

- Linux example: `~/.config/beepaper/config.toml`

The default state file lives under the user data directory:

- Linux example: `~/.local/share/beepaper/state.toml`

Example config:

```toml
dirs = ["/home/user/Pictures/Wallpapers"]
recursive = true
extensions = ["jpg", "jpeg", "png", "webp"]
history_size = 50
random_no_repeat_window = 5
apply_mode = "fill"
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
- `src/wayland/`: native Wayland apply path
- `src/lib.rs`: library module exports

## Native Wayland MVP

`beepaper apply <path>` creates a native Wayland background surface using:

- `wl_compositor`
- `wl_shm`
- `zwlr_layer_shell_v1`
- one `wl_output` selected as the first advertised output

Current limitations:

- single-output oriented MVP
- static image only
- wlroots/layer-shell compatible compositor required
- no GPU rendering
- no daemon/orchestration layer beyond keeping the client alive while the wallpaper surface exists
- no external wallpaper tools such as `swww`, `swaybg`, or `hyprpaper`

## Development

```bash
cargo test
cargo run -- init-config
cargo run -- scan
cargo run -- random
cargo run -- random --apply
cargo run -- apply /path/to/wallpaper.jpg
```
