# RustXterm Architecture Document

## 1. Overview

RustXterm is a cross-platform remote computing toolkit written in Rust, inspired by MobaXterm. It provides an integrated terminal emulator, session manager, file browser, and network tools in a single application. Target platforms: **Windows** and **Rocky Linux**.

## 2. High-Level Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                        RustXterm Application                     │
├──────────────┬───────────────────────────────────┬───────────────┤
│   Frontend   │         Core Services             │   Platform    │
│   (Tauri +   │                                   │   Abstraction │
│    WebView)  │  ┌─────────────┐ ┌─────────────┐  │   Layer       │
│              │  │  Session    │ │  Terminal    │  │               │
│  ┌────────┐  │  │  Manager   │ │  Emulator    │  │  ┌─────────┐  │
│  │ UI     │  │  └─────────────┘ └─────────────┘  │  │ Windows │  │
│  │ Shell  │──│──┌─────────────┐ ┌─────────────┐  │  │ API     │  │
│  │ (React │  │  │  Connection │ │  File        │  │  ├─────────┤  │
│  │  /TS)  │  │  │  Protocols  │ │  Manager     │  │  │ Linux   │  │
│  └────────┘  │  └─────────────┘ └─────────────┘  │  │ API     │  │
│              │  ┌─────────────┐ ┌─────────────┐  │  └─────────┘  │
│              │  │  Credential │ │  Network     │  │               │
│              │  │  Store      │ │  Tools       │  │               │
│              │  └─────────────┘ └─────────────┘  │               │
├──────────────┴───────────────────────────────────┴───────────────┤
│                     Plugin System (Dynamic Loading)              │
└──────────────────────────────────────────────────────────────────┘
```

## 3. Technology Stack

| Layer              | Technology                    | Rationale                                      |
|--------------------|-------------------------------|-------------------------------------------------|
| GUI Framework      | **Tauri v2**                  | Cross-platform, small binary, native webview    |
| Frontend UI        | **React + TypeScript**        | Rich UI components, xterm.js terminal rendering |
| Terminal Rendering | **xterm.js**                  | Battle-tested terminal emulator in browser      |
| Terminal Backend   | **portable-pty**              | Cross-platform PTY spawning                     |
| VT Parsing         | **vte** crate                 | Fast VT100/xterm escape sequence parser         |
| SSH                | **russh**                     | Pure Rust async SSH2 client                     |
| SFTP               | **russh-sftp**                | SFTP over russh                                 |
| FTP                | **suppaftp**                  | FTP/FTPS client                                 |
| Serial             | **serialport**                | Cross-platform serial port access               |
| RDP                | **IronRDP**                   | Pure Rust RDP client                            |
| VNC                | Custom / **vnc-rs**           | VNC client protocol                             |
| Telnet             | Custom implementation         | Simple protocol, direct TCP                     |
| Credential Storage | **keyring** + AES-256-GCM     | OS keychain + encrypted file fallback           |
| Database           | **SQLite (rusqlite)**         | Session/bookmark persistence                    |
| Async Runtime      | **Tokio**                     | Industry standard async runtime                 |
| Serialization      | **serde + serde_json/toml**   | Config and data serialization                   |
| Logging            | **tracing**                   | Structured async-aware logging                  |
| Plugin System      | **libloading** + trait objects | Dynamic .so/.dll plugin loading                 |

## 4. Module Architecture

### 4.1 Workspace Structure

```
rustxterm/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── rustxterm-app/            # Tauri application entry point
│   ├── rustxterm-core/           # Core shared types & traits
│   ├── rustxterm-terminal/       # Terminal emulation & PTY
│   ├── rustxterm-session/        # Session management & persistence
│   ├── rustxterm-ssh/            # SSH/SFTP protocol handler
│   ├── rustxterm-telnet/         # Telnet protocol handler
│   ├── rustxterm-rdp/            # RDP protocol handler
│   ├── rustxterm-vnc/            # VNC protocol handler
│   ├── rustxterm-ftp/            # FTP/FTPS protocol handler
│   ├── rustxterm-serial/         # Serial port handler
│   ├── rustxterm-filemanager/    # SFTP/local file browser
│   ├── rustxterm-credentials/    # Credential/password management
│   ├── rustxterm-network-tools/  # Network utilities (port scan, etc.)
│   ├── rustxterm-tunnel/         # SSH tunnel/port forwarding
│   └── rustxterm-plugin/         # Plugin system framework
├── frontend/                     # Tauri frontend (React + TypeScript)
│   ├── src/
│   │   ├── components/
│   │   │   ├── Terminal/         # xterm.js terminal component
│   │   │   ├── SessionManager/   # Session sidebar
│   │   │   ├── FileBrowser/      # SFTP file browser sidebar
│   │   │   ├── TabBar/           # Tab management
│   │   │   ├── SplitPane/        # Split terminal view
│   │   │   └── NetworkTools/     # Network utility UI
│   │   ├── hooks/
│   │   ├── store/                # State management (Zustand)
│   │   └── styles/
│   ├── package.json
│   └── tsconfig.json
├── plugins/                      # Built-in & third-party plugins
├── docs/
└── tests/                        # Integration tests
```

### 4.2 Core Crate (`rustxterm-core`)

Defines shared traits and types used across all crates:

```rust
/// Every connection protocol implements this trait
#[async_trait]
pub trait ConnectionHandler: Send + Sync {
    async fn connect(&mut self, config: &ConnectionConfig) -> Result<()>;
    async fn disconnect(&mut self) -> Result<()>;
    async fn send(&mut self, data: &[u8]) -> Result<()>;
    async fn on_data(&mut self, callback: DataCallback) -> Result<()>;
    fn connection_type(&self) -> ProtocolType;
    fn is_connected(&self) -> bool;
}

pub enum ProtocolType {
    Ssh, Telnet, Rdp, Vnc, Ftp, Sftp, Serial, Mosh, Rlogin,
}

pub struct ConnectionConfig {
    pub host: String,
    pub port: u16,
    pub protocol: ProtocolType,
    pub credentials: Option<CredentialRef>,
    pub ssh_gateway: Option<Box<ConnectionConfig>>,  // Jump host
    pub extra: HashMap<String, String>,
}
```

### 4.3 Terminal Module (`rustxterm-terminal`)

```
┌─────────────────────────────────┐
│       xterm.js (Frontend)       │  ← Renders terminal output
├─────────────────────────────────┤
│     Tauri IPC Channel           │  ← Bidirectional data stream
├─────────────────────────────────┤
│   Terminal Session Controller   │  ← Routes data to/from backend
├──────────┬──────────────────────┤
│  Local   │   Remote (SSH/etc)   │  ← PTY or protocol connection
│  PTY     │   Connection         │
└──────────┴──────────────────────┘
```

- **Local Shell**: Uses `portable-pty` to spawn local shell (bash/PowerShell/cmd)
- **Remote Shell**: Pipes data through SSH/Telnet connection
- **Features**: Multi-tab, split pane (horizontal/vertical), search, scrollback buffer

### 4.4 Session Manager (`rustxterm-session`)

- Stores sessions as bookmarks in SQLite
- Session types: SSH, Telnet, RDP, VNC, FTP, SFTP, Serial, Local Shell
- Folder-based organization
- Import/Export (JSON format)
- Each session stores: connection params, display settings, startup commands, environment variables

### 4.5 File Manager (`rustxterm-filemanager`)

- Auto-opens SFTP browser sidebar when SSH session connects
- Dual-pane file browser (local ↔ remote)
- Drag-and-drop file transfer
- Remote file editing (opens in integrated or external editor)
- Transfer queue with progress tracking

### 4.6 SSH Tunnel Manager (`rustxterm-tunnel`)

- Local port forwarding (L)
- Remote port forwarding (R)
- Dynamic port forwarding (D / SOCKS proxy)
- Visual tunnel builder UI
- Persistent tunnel configurations

### 4.7 Credential Store (`rustxterm-credentials`)

```
┌──────────────────────┐
│   Master Password    │
│   (PBKDF2 derived)   │
├──────────────────────┤
│   AES-256-GCM        │
│   Encrypted Store    │
├──────────────────────┤
│   OS Keyring         │  ← Stores master key (optional)
│   (keyring crate)    │
└──────────────────────┘
```

- Master password protects all stored credentials
- Falls back to OS keychain integration when available
- SSH key management (generation, import, agent forwarding)
- Credential auto-fill for sessions

### 4.8 Network Tools (`rustxterm-network-tools`)

| Tool             | Description                           |
|------------------|---------------------------------------|
| Port Scanner     | TCP port scanning with service detect |
| Network Info     | Interface listing, IP/MAC display     |
| Packet Capture   | Basic TCP capture (pcap)              |
| Wake-on-LAN      | Send WOL magic packets               |
| Bandwidth Test   | Network speed testing                 |
| DNS Lookup       | DNS query tool                        |
| Ping             | ICMP ping utility                     |
| Traceroute       | Network path tracing                  |

### 4.9 Plugin System (`rustxterm-plugin`)

```rust
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn on_load(&mut self, ctx: &PluginContext) -> Result<()>;
    fn on_unload(&mut self) -> Result<()>;
}

/// Plugins are .so (Linux) / .dll (Windows) loaded at runtime
/// Plugin API provides:
///   - Terminal output hooks
///   - Custom tab types
///   - Menu extensions
///   - Custom protocol handlers
```

## 5. Data Flow

### 5.1 SSH Session Lifecycle

```
User clicks "Connect" in UI
  → Tauri command: connect_session(session_id)
    → SessionManager loads config from SQLite
      → If SSH gateway defined, establish jump connection first
      → CredentialStore decrypts credentials
      → SshHandler::connect() establishes SSH channel
        → PTY channel opened on remote host
        → SFTP subsystem opened in parallel
      → Terminal data stream bound to xterm.js via IPC
      → SFTP browser sidebar populated with remote filesystem
User types in terminal
  → xterm.js captures keystrokes
    → IPC sends raw bytes to Rust backend
      → SshHandler::send() writes to SSH channel
        → Remote PTY receives input
        → Remote PTY produces output
      → SshHandler::on_data() callback fires
    → IPC sends output bytes to frontend
  → xterm.js renders terminal output
```

### 5.2 Multi-Execution Flow

```
User selects multiple terminal tabs
  → Multi-exec mode activated
    → Keystrokes captured once
      → Broadcast to all selected terminal sessions
      → Each session processes independently
      → Individual outputs rendered in respective tabs
```

## 6. Platform Abstraction

```rust
// Platform-specific implementations behind a unified trait
pub trait PlatformService {
    fn default_shell(&self) -> PathBuf;
    fn config_dir(&self) -> PathBuf;
    fn open_file_manager(&self, path: &Path) -> Result<()>;
    fn native_notifications(&self) -> Box<dyn NotificationService>;
    fn system_proxy(&self) -> Option<ProxyConfig>;
}

// Compile-time platform selection
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "linux")]
mod linux;
```

| Feature            | Windows                       | Linux (Rocky)               |
|--------------------|-------------------------------|-----------------------------|
| Default Shell      | PowerShell / cmd.exe          | bash / zsh                  |
| PTY                | ConPTY (Win10+)               | /dev/ptmx                   |
| Config Storage     | %APPDATA%\rustxterm           | ~/.config/rustxterm          |
| Credential Store   | Windows Credential Manager    | Secret Service / file-based |
| Notifications      | Windows Toast                 | D-Bus notifications          |
| File Associations  | Registry                      | XDG MIME                     |

## 7. Security Architecture

1. **Credential Encryption**: AES-256-GCM with PBKDF2-derived keys
2. **SSH Key Handling**: Keys stored in-memory only during session; never logged
3. **Master Password**: Required on first launch; cached in OS keychain optionally
4. **Transport Security**: All remote connections use TLS/SSH where applicable
5. **Plugin Sandboxing**: Plugins run with restricted filesystem access
6. **No Plaintext Secrets**: Passwords/keys never written to disk unencrypted
7. **Host Key Verification**: Strict host key checking with known_hosts management

## 8. Performance Considerations

1. **Async I/O**: All network operations use Tokio async runtime
2. **Terminal Rendering**: xterm.js with WebGL renderer for GPU-accelerated output
3. **Lazy Loading**: Protocol crates loaded on-demand, not at startup
4. **Connection Pooling**: Reuse SSH connections for multiple channels
5. **Streaming Transfers**: File transfers use streaming with backpressure
6. **Memory**: Terminal scrollback configurable; default 10,000 lines
7. **Startup**: Target < 1 second cold start

## 9. Build & Distribution

| Target              | Format                         |
|---------------------|--------------------------------|
| Windows             | MSI installer, portable .exe   |
| Rocky Linux (RHEL)  | RPM package, AppImage          |

- CI/CD: GitHub Actions with cross-compilation
- Tauri handles platform-specific bundling
- Frontend assets embedded in binary (single-file distribution)
