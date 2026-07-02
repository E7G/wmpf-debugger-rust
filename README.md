# WMPF Debugger (Rust)

A WeChat Mini Program (WMPF) remote debugger written in pure Rust.

This tool intercepts the communication between WeChatAppEx.exe and the Chrome DevTools Protocol (CDP), allowing you to debug WeChat Mini Programs using standard browser developer tools.

## Architecture

```
┌──────────────────┐     ┌──────────────────┐     ┌──────────────────┐
│   WeChatAppEx    │────▶│  WMPF Debugger   │────▶│  Chrome DevTools │
│   (Mini Program) │     │  (Rust)          │     │  Inspector       │
│                  │◀────│                  │◀────│                  │
└──────────────────┘     └──────────────────┘     └──────────────────┘
      ws://localhost:9421        ↕           ws://localhost:62666
                              Frida
                           (script inject)
```

1. **Frida** injects a hook script into WeChatAppEx.exe to enable the remote debug protocol
2. **Debug Server** (port 9421) receives protobuf-encoded debug messages from the mini program
3. **CDP Proxy** (port 62666) translates between the internal protocol and Chrome DevTools Protocol

## Prerequisites

- Windows with WeChat DevTools installed
- [Frida core devkit](https://github.com/frida/frida/releases) (only if building with `frida-link` feature)

## Build

```bash
# Without Frida (WebSocket servers only)
cargo build --release

# With Frida integration (requires frida-core devkit in frida-devkit/)
cargo build --release --features frida-link
```

## Usage

```bash
# Start the debugger
cargo run --release --features frida-link

# Or with custom ports
cargo run --release --features frida-link -- --debug-port 9421 --cdp-port 62666
```

Then open Chrome DevTools:
```
devtools://devtools/bundled/inspector.html?ws=127.0.0.1:62666
```

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `--debug-port <port>` | Debug server WebSocket port | 9421 |
| `--cdp-port <port>` | CDP proxy WebSocket port | 62666 |
| `--debug-main` | Output main process debug messages | false |
| `--debug-frida` | Output Frida client messages | false |

## How It Works

1. Attaches to WeChatAppEx.exe via Frida
2. Extracts WMPF version from the process path
3. Loads version-specific hook script from `frida/hook.js`
4. Patches the CDP filter to allow remote debugging
5. Modifies the mini program scene to enable debug mode
6. Proxies debug messages between the mini program and Chrome DevTools

## Project Structure

```
src/
├── main.rs           # Entry point, spawns all servers
├── cli.rs            # Command-line argument parsing
├── logger.rs         # Colored logging
├── constants.rs      # Protocol constants and enums
├── proto.rs          # Protobuf generated types (prost)
├── codex.rs          # Protobuf encode/decode + Zlib compression
├── frida_ffi.rs      # Raw FFI bindings to Frida C API (behind frida-link feature)
├── frida_server.rs   # Frida integration (process attach, script inject)
├── debug_server.rs   # WebSocket server for mini program connections
└── proxy_server.rs   # WebSocket server for CDP connections

proto/
└── wa_remote_debug.proto  # WeChat remote debug protocol definition

frida/
├── hook.js           # Frida hook script
└── config/           # Version-specific memory addresses
```

## Acknowledgments

- Original TypeScript implementation: [WMPFDebugger](https://github.com/nicennnnnnnlee/WMPFDebugger) by evi0s
- [Frida](https://frida.re/) - Dynamic instrumentation toolkit

## License

GPL-2.0-only
