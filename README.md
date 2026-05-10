# zellij-session-sidebar

A small Zellij plugin that renders a permanent, Strider-style session sidebar.

## Features

- Shows all Zellij sessions in a narrow sidebar
- Pins the current session at the top
- Marks the current session with `•`
- Mouse hover selects a session
- Mouse wheel moves the selection
- Mouse click switches sessions
- Does not keep Zellij alive after all regular terminal panes exit

## Requirements

- Zellij `0.44.1`

## Install from release

Download the prebuilt wasm from the latest GitHub release:

```bash
mkdir -p ~/.config/zellij/plugins
curl -L \
  https://github.com/devlkx/zellij-session-sidebar/releases/latest/download/zellij-session-sidebar.wasm \
  -o ~/.config/zellij/plugins/zellij-session-sidebar.wasm
```

Then add it to your Zellij layout:

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

For example, if you downloaded it with the command above:

```kdl
plugin location="file:~/.config/zellij/plugins/zellij-session-sidebar.wasm"
```

Zellij will ask for plugin permissions the first time it loads the wasm.

## Build from source

Install Rust and the WASI target:

```bash
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1
```

The compiled plugin will be at:

```text
target/wasm32-wasip1/release/zellij-session-sidebar.wasm
```

For a personal config, keep the `plugin location` pointing to the same wasm path across rebuilds. Zellij caches plugin permissions by path, so changing the filename can require granting permissions again.

## Permissions

The plugin requests:

- `ReadApplicationState` — read available sessions
- `ChangeApplicationState` — switch sessions

## License

MIT
