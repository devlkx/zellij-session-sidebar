# zellij-session-sidebar

A small Zellij plugin that renders a permanent, Strider-style session sidebar.

## Features

- Shows all Zellij sessions in a narrow sidebar
- Pins the current session at the top
- Marks the current session with `•`
- Mouse hover selects a session
- Mouse wheel moves the selection
- Mouse click switches sessions
- `Enter` switches to the selected session

## Requirements

- Zellij `0.44.1`
- Rust toolchain with the `wasm32-wasip1` target

## Build

```bash
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1
```

The compiled plugin will be at:

```text
target/wasm32-wasip1/release/zellij-session-sidebar.wasm
```

## Zellij layout example

```kdl
layout {
    pane split_direction="vertical" {
        pane size=28 borderless=true {
            plugin location="file:/absolute/path/to/zellij-session-sidebar.wasm"
        }

        pane
    }

    pane size=1 borderless=true {
        plugin location="compact-bar"
    }
}
```

For a personal config, you may copy the built wasm to a stable filename and keep your layout pointing to that path. This avoids re-granting Zellij plugin permissions after every rebuild.

Example:

```bash
cp target/wasm32-wasip1/release/zellij-session-sidebar.wasm \
   target/wasm32-wasip1/release/zellij_session_sidebar_v8.wasm
```

## Permissions

The plugin requests:

- `ReadApplicationState` — read available sessions
- `ChangeApplicationState` — switch sessions
