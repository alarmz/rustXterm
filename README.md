# RustXterm

**[English](#english) | [中文](#中文)**

---

## English

### Overview

RustXterm is a cross-platform remote computing toolkit written in Rust, inspired by [MobaXterm](https://mobaxterm.mobatek.net/). It integrates a terminal emulator, session manager, file browser, SSH client, and network tools into a single application.

### Supported Platforms

| Platform | Version |
|----------|---------|
| Windows | 10 / 11 (1809+) |
| Rocky Linux | 8.x / 9.x |

### Features

- **Terminal Emulator** - Multi-tab, split-pane terminal with xterm.js rendering
- **SSH / SFTP** - Secure shell with integrated SFTP file browser
- **Session Manager** - Save, organize, and quick-connect to remote servers
- **Multi-Protocol** - SSH, Telnet, RDP, VNC, FTP/FTPS, Serial
- **SSH Tunnels** - Visual port forwarding builder (Local / Remote / Dynamic)
- **File Transfer** - Drag-and-drop upload/download with progress tracking
- **Credential Store** - AES-256-GCM encrypted password management
- **Network Tools** - Port scanner, DNS lookup, ping, traceroute
- **Multi-Execution** - Send commands to multiple servers simultaneously
- **Plugin System** - Extend functionality via dynamic plugins (.so / .dll)

### Tech Stack

| Component | Technology |
|-----------|------------|
| Backend | Rust |
| Frontend | React + TypeScript (Tauri v2) |
| Terminal | xterm.js + portable-pty |
| SSH | russh (pure Rust) |
| Database | SQLite (rusqlite) |
| Encryption | AES-256-GCM + PBKDF2 |

### Project Structure

```
rustxterm/
├── crates/
│   ├── rustxterm-app/            # Tauri application entry point
│   ├── rustxterm-core/           # Core types and traits
│   ├── rustxterm-terminal/       # Terminal emulation & PTY
│   ├── rustxterm-session/        # Session management & persistence
│   ├── rustxterm-ssh/            # SSH/SFTP protocol
│   ├── rustxterm-telnet/         # Telnet protocol
│   ├── rustxterm-rdp/            # RDP protocol
│   ├── rustxterm-vnc/            # VNC protocol
│   ├── rustxterm-ftp/            # FTP/FTPS protocol
│   ├── rustxterm-serial/         # Serial port
│   ├── rustxterm-filemanager/    # File browser
│   ├── rustxterm-credentials/    # Credential management
│   ├── rustxterm-network-tools/  # Network utilities
│   ├── rustxterm-tunnel/         # SSH tunnel management
│   └── rustxterm-plugin/         # Plugin system
├── frontend/                     # React + TypeScript UI
└── docs/                         # Architecture & specification
```

### Building from Source

#### Prerequisites

- [Rust](https://rustup.rs/) (1.75+)
- [Node.js](https://nodejs.org/) (18+)
- [Tauri v2 CLI](https://v2.tauri.app/)

#### Linux Dependencies (Rocky Linux / RHEL)

```bash
sudo dnf install -y gcc webkit2gtk4.1-devel openssl-devel \
  curl wget file libappindicator-gtk3-devel librsvg2-devel \
  pango-devel gtk3-devel
```

#### Build

```bash
# Install Tauri CLI
cargo install tauri-cli

# Build the application
cargo tauri build
```

### Development Roadmap

| Phase | Description | Status |
|-------|-------------|--------|
| Phase 1 | Local terminal + SSH + Session manager | In Progress |
| Phase 2 | SFTP transfer, SSH tunnels, split pane | Planned |
| Phase 3 | Telnet, FTP, RDP, VNC, Serial | Planned |
| Phase 4 | Network tools, plugins, macros | Planned |
| Phase 5 | Polish, packaging, distribution | Planned |

### Documentation

- [Architecture Design](docs/ARCHITECTURE.md)
- [Functional Specification](docs/SPEC.md)

### License

MIT

---

## 中文

### 概述

RustXterm 是一個使用 Rust 編寫的跨平台遠端運算工具包，靈感來自 [MobaXterm](https://mobaxterm.mobatek.net/)。它將終端模擬器、工作階段管理器、檔案瀏覽器、SSH 用戶端和網路工具整合在單一應用程式中。

### 支援平台

| 平台 | 版本 |
|------|------|
| Windows | 10 / 11 (1809+) |
| Rocky Linux | 8.x / 9.x |

### 功能特色

- **終端模擬器** - 多分頁、分割面板終端，使用 xterm.js 渲染
- **SSH / SFTP** - 安全連線，整合 SFTP 檔案瀏覽器
- **工作階段管理** - 儲存、組織並快速連線到遠端伺服器
- **多協定支援** - SSH、Telnet、RDP、VNC、FTP/FTPS、序列埠
- **SSH 通道** - 視覺化通訊埠轉發建構器（本地 / 遠端 / 動態）
- **檔案傳輸** - 拖放上傳/下載，附帶進度追蹤
- **憑證儲存** - AES-256-GCM 加密密碼管理
- **網路工具** - 通訊埠掃描器、DNS 查詢、ping、traceroute
- **多重執行** - 同時向多台伺服器發送命令
- **外掛系統** - 透過動態外掛（.so / .dll）擴充功能

### 技術架構

| 元件 | 技術 |
|------|------|
| 後端 | Rust |
| 前端 | React + TypeScript (Tauri v2) |
| 終端 | xterm.js + portable-pty |
| SSH | russh（純 Rust 實作） |
| 資料庫 | SQLite (rusqlite) |
| 加密 | AES-256-GCM + PBKDF2 |

### 專案結構

```
rustxterm/
├── crates/
│   ├── rustxterm-app/            # Tauri 應用程式進入點
│   ├── rustxterm-core/           # 核心型別與特徵
│   ├── rustxterm-terminal/       # 終端模擬與 PTY
│   ├── rustxterm-session/        # 工作階段管理與持久化
│   ├── rustxterm-ssh/            # SSH/SFTP 協定
│   ├── rustxterm-telnet/         # Telnet 協定
│   ├── rustxterm-rdp/            # RDP 協定
│   ├── rustxterm-vnc/            # VNC 協定
│   ├── rustxterm-ftp/            # FTP/FTPS 協定
│   ├── rustxterm-serial/         # 序列埠
│   ├── rustxterm-filemanager/    # 檔案瀏覽器
│   ├── rustxterm-credentials/    # 憑證管理
│   ├── rustxterm-network-tools/  # 網路工具
│   ├── rustxterm-tunnel/         # SSH 通道管理
│   └── rustxterm-plugin/         # 外掛系統
├── frontend/                     # React + TypeScript 介面
└── docs/                         # 架構與規格文件
```

### 從原始碼建構

#### 先決條件

- [Rust](https://rustup.rs/) (1.75+)
- [Node.js](https://nodejs.org/) (18+)
- [Tauri v2 CLI](https://v2.tauri.app/)

#### Linux 相依套件（Rocky Linux / RHEL）

```bash
sudo dnf install -y gcc webkit2gtk4.1-devel openssl-devel \
  curl wget file libappindicator-gtk3-devel librsvg2-devel \
  pango-devel gtk3-devel
```

#### 建構

```bash
# 安裝 Tauri CLI
cargo install tauri-cli

# 建構應用程式
cargo tauri build
```

### 開發路線圖

| 階段 | 說明 | 狀態 |
|------|------|------|
| 第一階段 | 本地終端 + SSH + 工作階段管理 | 進行中 |
| 第二階段 | SFTP 傳輸、SSH 通道、分割面板 | 規劃中 |
| 第三階段 | Telnet、FTP、RDP、VNC、序列埠 | 規劃中 |
| 第四階段 | 網路工具、外掛、巨集 | 規劃中 |
| 第五階段 | 優化、打包、發布 | 規劃中 |

### 文件

- [架構設計](docs/ARCHITECTURE.md)
- [功能規格書](docs/SPEC.md)

### 授權條款

MIT
