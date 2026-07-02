# WMPF 调试器 (Rust)

一个用纯 Rust 编写的微信小程序 (WMPF) 远程调试工具。

该工具拦截 WeChatAppEx.exe 与 Chrome DevTools Protocol (CDP) 之间的通信，允许你使用标准浏览器开发者工具调试微信小程序。

## 架构

```
┌──────────────────┐     ┌──────────────────┐     ┌──────────────────┐
│   WeChatAppEx    │────▶│  WMPF 调试器     │────▶│  Chrome DevTools │
│   (小程序进程)    │     │  (Rust)          │     │  Inspector       │
│                  │◀────│                  │◀────│                  │
└──────────────────┘     └──────────────────┘     └──────────────────┘
      ws://localhost:9421        ↕           ws://localhost:62666
                              Frida
                           (脚本注入)
```

1. **Frida** 向 WeChatAppEx.exe 注入钩子脚本，启用远程调试协议
2. **调试服务器** (端口 9421) 接收小程序发送的 protobuf 编码调试消息
3. **CDP 代理** (端口 62666) 在内部协议与 Chrome DevTools Protocol 之间转换

## 安装

### 下载预编译二进制文件（推荐）

从 [GitHub Releases](https://github.com/E7G/wmpf-debugger-rust/releases) 下载最新版本：

| 平台 | 文件 |
|------|------|
| Windows x64 | `wmpf-debugger-x86_64-pc-windows-msvc.exe` |
| Linux x64 | `wmpf-debugger-x86_64-unknown-linux-gnu` |
| macOS x64 | `wmpf-debugger-x86_64-apple-darwin` |

```bash
# Windows (PowerShell)
Invoke-WebRequest -Uri "https://github.com/E7G/wmpf-debugger-rust/releases/latest/download/wmpf-debugger-x86_64-pc-windows-msvc.exe" -OutFile "wmpf-debugger.exe"

# Linux
curl -LO https://github.com/E7G/wmpf-debugger-rust/releases/latest/download/wmpf-debugger-x86_64-unknown-linux-gnu
chmod +x wmpf-debugger-x86_64-unknown-linux-gnu

# macOS
curl -LO https://github.com/E7G/wmpf-debugger-rust/releases/latest/download/wmpf-debugger-x86_64-apple-darwin
chmod +x wmpf-debugger-x86_64-apple-darwin
```

### 从源码编译

**前置要求：**
- [Rust](https://rustup.rs/) (1.70+)
- [protobuf 编译器](https://github.com/protocolbuffers/protobuf/releases) (`protoc`)

```bash
# 克隆仓库
git clone https://github.com/E7G/wmpf-debugger-rust.git
cd wmpf-debugger-rust

# 不带 Frida 编译（仅 WebSocket 服务器）
cargo build --release

# 带 Frida 集成编译（需要 frida-core devkit）
# 1. 从 https://github.com/frida/frida/releases 下载 frida-core-devkit
# 2. 解压到项目根目录的 frida-devkit/ 文件夹
# 3. 使用 feature flag 编译
cargo build --release --features frida-link
```

编译后的二进制文件位于 `target/release/wmpf-debugger.exe` (Windows) 或 `target/release/wmpf-debugger` (Linux/macOS)。

## 使用方法

### 1. 启动调试器

```bash
# Windows
wmpf-debugger.exe

# Linux/macOS
./wmpf-debugger-x86_64-unknown-linux-gnu
```

确保微信开发者工具正在运行（WeChatAppEx.exe 进程必须处于活动状态）。

### 2. 打开 Chrome DevTools

在浏览器中访问：
```
devtools://devtools/bundled/inspector.html?ws=127.0.0.1:62666
```

### 3. 调试小程序

在微信开发者工具中打开任意小程序 - 调试器会自动连接，你可以使用 Chrome DevTools 进行调试。

### 命令行参数

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `--debug-port <port>` | 调试服务器 WebSocket 端口 | 9421 |
| `--cdp-port <port>` | CDP 代理 WebSocket 端口 | 62666 |
| `--debug-main` | 输出主进程调试消息 | false |
| `--debug-frida` | 输出 Frida 客户端消息 | false |
| `-h, --help` | 显示帮助信息 | |

```bash
# 示例：自定义端口并开启调试输出
wmpf-debugger.exe --debug-port 9421 --cdp-port 62666 --debug-main --debug-frida
```

## 工作原理

1. 通过 Frida 附加到 WeChatAppEx.exe 进程
2. 从进程路径中提取 WMPF 版本号（如 `...\14185\...`）
3. 加载版本对应的钩子脚本 `frida/hook.js`
4. 修补 CDP 过滤器以允许远程调试
5. 修改小程序场景以启用调试模式
6. 在小程序与 Chrome DevTools 之间代理调试消息

## 项目结构

```
src/
├── main.rs           # 入口，启动所有服务器
├── cli.rs            # 命令行参数解析
├── logger.rs         # 彩色日志
├── constants.rs      # 协议常量和枚举
├── proto.rs          # Protobuf 生成类型 (prost)
├── codex.rs          # Protobuf 编解码 + Zlib 压缩
├── frida_ffi.rs      # Frida C API 原始 FFI 绑定 (frida-link feature)
├── frida_server.rs   # Frida 集成（进程附加、脚本注入）
├── debug_server.rs   # 小程序连接的 WebSocket 服务器
└── proxy_server.rs   # CDP 连接的 WebSocket 服务器

proto/
└── wa_remote_debug.proto  # 微信远程调试协议定义

frida/
├── hook.js           # Frida 钩子脚本
└── config/           # 版本特定的内存地址 (45 个版本)
```

## 故障排除

| 问题 | 解决方案 |
|------|----------|
| "WeChatAppEx.exe process not found" | 确保微信开发者工具正在运行 |
| "error finding WMPF version" | 你的 WMPF 版本可能不受支持（检查 `frida/config/`） |
| "hook script not found" | 确保 `frida/hook.js` 在工作目录中 |
| 端口已被占用 | 使用 `--debug-port` 和 `--cdp-port` 更改端口 |
| Frida 不可用 | 使用 `--features frida-link` 编译并提供 frida-core devkit |

## 致谢

- 原始 TypeScript 实现：[WMPFDebugger](https://github.com/nicennnnnnnlee/WMPFDebugger) by evi0s
- [Frida](https://frida.re/) - 动态插桩工具包

## 许可证

GPL-2.0-only
