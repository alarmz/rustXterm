# RustXterm Specification Document

## 1. Project Overview

| Item           | Detail                                                |
|----------------|-------------------------------------------------------|
| Project Name   | RustXterm                                             |
| Language       | Rust (backend) + TypeScript/React (frontend)          |
| GUI Framework  | Tauri v2                                              |
| Platforms      | Windows 10/11, Rocky Linux 8/9                        |
| License        | TBD                                                   |
| Inspired By    | MobaXterm                                             |

## 2. Functional Requirements

### 2.1 Terminal Emulator (P0 - Must Have)

#### FR-T01: Local Terminal
- Spawn local shell process (bash on Linux, PowerShell/cmd on Windows)
- Full VT100/xterm-256color terminal emulation via xterm.js
- Configurable scrollback buffer (default: 10,000 lines, max: unlimited)
- Copy/paste support with configurable keyboard shortcuts
- Search within terminal output (Ctrl+Shift+F)
- Configurable font family, font size, color scheme
- Bell notification (visual/audio/disabled)

#### FR-T02: Multi-Tab Terminal
- Multiple terminal sessions as tabs
- Tab reordering via drag-and-drop
- Tab naming (auto from connection, or user-defined)
- Tab color coding
- Close tab confirmation when session is active
- Keyboard shortcuts: Ctrl+T (new tab), Ctrl+W (close tab), Ctrl+Tab (next tab)

#### FR-T03: Split Pane
- Split terminal horizontally or vertically
- Up to 4 panes per tab (2x2 grid)
- Independent sessions per pane
- Resize panes via drag handle
- Keyboard shortcuts for navigation between panes

#### FR-T04: Multi-Execution
- Select multiple terminal tabs/panes
- Input typed once is sent to all selected terminals simultaneously
- Visual indicator showing which terminals are in multi-exec mode
- Toggle individual terminals in/out of multi-exec group

#### FR-T05: Terminal Macros
- Record terminal input sequence
- Replay macro on current or selected terminals
- Save/load macros with names
- Macro editor for manual editing

---

### 2.2 Session Management (P0 - Must Have)

#### FR-S01: Session Types
Support the following session/connection types:

| Type    | Protocol        | Default Port | Description                    |
|---------|-----------------|--------------|--------------------------------|
| SSH     | SSH2            | 22           | Secure shell with PTY          |
| SFTP    | SFTP over SSH   | 22           | SSH file transfer              |
| FTP     | FTP / FTPS      | 21 / 990     | File transfer protocol         |
| Telnet  | Telnet          | 23           | Unencrypted remote terminal    |
| RDP     | RDP             | 3389         | Windows remote desktop         |
| VNC     | VNC/RFB         | 5900         | Virtual network computing      |
| Serial  | RS-232          | N/A          | Serial port connection         |
| Shell   | Local PTY       | N/A          | Local terminal session         |

#### FR-S02: Session Properties

Common properties for all session types:
```
Session:
  name: string                    # Display name
  group: string                   # Folder/group path (e.g., "Production/Web Servers")
  host: string                    # Hostname or IP
  port: u16                       # Port number
  protocol: ProtocolType          # See FR-S01
  credentials: CredentialRef      # Reference to stored credentials
  auto_connect: bool              # Connect on application start
  startup_commands: Vec<string>   # Commands to run after connection
  environment: Map<string,string> # Environment variables to set
  color_tag: Option<Color>        # Color coding for organization
  notes: string                   # User notes
  created_at: DateTime
  updated_at: DateTime
```

SSH-specific properties:
```
SshSessionConfig:
  auth_method: enum { Password, PublicKey, KeyboardInteractive, Agent }
  private_key_path: Option<PathBuf>
  passphrase: Option<CredentialRef>
  ssh_gateway: Option<SessionRef>       # Jump host
  x11_forwarding: bool
  agent_forwarding: bool
  compression: bool
  keepalive_interval: u32               # Seconds, 0 = disabled
  terminal_type: string                 # Default: "xterm-256color"
  encoding: string                      # Default: "UTF-8"
  port_forwards: Vec<PortForward>       # Pre-configured tunnels
```

Serial-specific properties:
```
SerialSessionConfig:
  device: string             # e.g., COM3 or /dev/ttyUSB0
  baud_rate: u32             # 9600, 115200, etc.
  data_bits: enum { 5, 6, 7, 8 }
  stop_bits: enum { 1, 2 }
  parity: enum { None, Odd, Even }
  flow_control: enum { None, Hardware, Software }
```

RDP-specific properties:
```
RdpSessionConfig:
  width: u32
  height: u32
  color_depth: enum { 15, 16, 24, 32 }
  fullscreen: bool
  drive_redirection: Vec<PathBuf>
  clipboard_sharing: bool
  audio_redirection: bool
```

#### FR-S03: Session Organization
- Tree-based folder hierarchy (unlimited depth)
- Drag-and-drop reordering within sidebar
- Search/filter sessions by name, host, group, or tag
- Favorite sessions (pinned to top)
- Recent sessions list (last 20)

#### FR-S04: Session Import/Export
- Export all sessions to JSON file
- Import sessions from JSON file
- Import from SSH config file (~/.ssh/config)
- Import from PuTTY registry entries (Windows only)
- Merge import (skip duplicates by host+port+user)

#### FR-S05: Quick Connect
- Quick connect dialog: protocol selector + host + port + username
- Remembers last N quick-connect entries
- Option to save quick connect as permanent session

---

### 2.3 SSH Features (P0 - Must Have)

#### FR-SSH01: SSH Connection
- SSH2 protocol support
- Authentication: password, public key, keyboard-interactive, SSH agent
- Host key verification with known_hosts management
- First-connect trust-on-first-use with fingerprint display
- Automatic reconnection on disconnect (configurable)

#### FR-SSH02: SSH Gateway / Jump Host
- Connect through one or more intermediate SSH servers
- Chained gateway support (A → B → C → target)
- Gateway sessions can be shared across multiple target sessions

#### FR-SSH03: SSH Key Management
- Generate SSH key pairs (RSA 2048/4096, Ed25519)
- View public key / fingerprint
- Import existing keys
- SSH agent with key loading
- Pageant support (Windows)

#### FR-SSH04: SSH Tunnels / Port Forwarding
- Local port forwarding (-L): local_port → remote_host:remote_port
- Remote port forwarding (-R): remote_port → local_host:local_port
- Dynamic port forwarding (-D): SOCKS proxy
- Visual tunnel builder with diagram
- Tunnel status monitoring (active connections count)
- Persistent tunnel configurations per session

#### FR-SSH05: X11 Forwarding
- X11 forwarding support in SSH sessions
- On Linux: forward to local X11 display
- On Windows: integrate with external X server or provide guidance
- Configurable DISPLAY variable

---

### 2.4 File Management (P0 - Must Have)

#### FR-F01: SFTP Browser
- Auto-open sidebar when SSH session connects
- Tree view of remote filesystem
- File/directory operations: create, rename, delete, chmod, chown
- File size, permissions, owner, modification date display
- Hidden files toggle
- Bookmark remote directories

#### FR-F02: File Transfer
- Drag-and-drop between local and remote file browsers
- Download/upload with progress bar
- Transfer queue management
- Resume interrupted transfers (when server supports)
- Recursive directory transfer
- Transfer speed display
- Concurrent transfer limit (configurable, default: 3)

#### FR-F03: Remote File Editing
- Double-click to open remote files in integrated text editor
- Auto-upload on save
- Syntax highlighting (common languages)
- Configurable external editor integration

#### FR-F04: Local File Browser
- Left sidebar panel showing local filesystem
- Standard file operations
- Path bar with breadcrumb navigation
- Quick access to home directory, desktop, recent locations

---

### 2.5 Credential Management (P0 - Must Have)

#### FR-C01: Master Password
- Set master password on first launch
- Master password required to unlock credential store
- Option to save master password in OS keychain
- Auto-lock after configurable idle timeout

#### FR-C02: Credential Storage
- Store username/password pairs
- Store SSH private keys (encrypted at rest)
- Associate credentials with sessions
- Credential sharing across multiple sessions
- Named credential entries for reuse

#### FR-C03: Security
- AES-256-GCM encryption for stored credentials
- Key derivation: PBKDF2 with 600,000 iterations (or Argon2id)
- Clear credentials from memory after session disconnect
- No plaintext credential logging
- Configurable clipboard auto-clear timeout (default: 30 seconds)

---

### 2.6 Network Tools (P1 - Should Have)

#### FR-N01: Port Scanner
- TCP port scanning (connect scan)
- Configurable port ranges
- Common port presets (top 100, top 1000)
- Service/banner detection
- Results table with sort/filter/export

#### FR-N02: Network Information
- List local network interfaces
- Display IP addresses, MAC addresses, subnet masks
- Default gateway and DNS servers
- Active connections list (like netstat)

#### FR-N03: DNS Lookup
- Forward lookup (hostname → IP)
- Reverse lookup (IP → hostname)
- Record types: A, AAAA, MX, CNAME, TXT, NS, SOA
- Custom DNS server selection

#### FR-N04: Ping
- ICMP ping with configurable count, interval, packet size
- Round-trip time statistics (min/avg/max/stddev)
- Visual latency graph

#### FR-N05: Traceroute
- ICMP/UDP traceroute
- Hop-by-hop latency display
- Hostname resolution for each hop

#### FR-N06: Wake-on-LAN
- Send WOL magic packet by MAC address
- Saved WOL targets list

---

### 2.7 Appearance & Customization (P1 - Should Have)

#### FR-A01: Themes
- Built-in themes: Dark (default), Light, Solarized, Monokai, Dracula
- Custom theme editor (background, foreground, ANSI 16 colors)
- Per-session theme override
- System theme auto-detection (follow OS dark/light mode)

#### FR-A02: Layout
- Configurable sidebar position (left/right)
- Sidebar auto-hide option
- Configurable toolbar visibility
- Remember window size and position per monitor
- Full-screen mode (F11)

#### FR-A03: Fonts
- Configurable terminal font (monospace only)
- Font size adjustment: Ctrl+Plus / Ctrl+Minus / Ctrl+0 (reset)
- Ligature support toggle
- Bundled fonts: JetBrains Mono, Fira Code, Cascadia Code

---

### 2.8 Multi-Execution & Automation (P1 - Should Have)

#### FR-M01: Multi-Execution Mode
- Toggle multi-exec mode from toolbar or Ctrl+Shift+M
- Checkboxes on each tab to include/exclude from multi-exec
- Select all / deselect all
- Input bar at bottom when multi-exec is active
- Visual border highlight on multi-exec tabs

#### FR-M02: Startup Automation
- Run commands after session connects (per-session config)
- Wait for prompt detection before running commands
- Variable substitution in commands: %HOST%, %USER%, %SESSION_NAME%

---

### 2.9 Plugin System (P2 - Nice to Have)

#### FR-P01: Plugin Architecture
- Plugins as dynamic libraries (.so / .dll)
- Plugin manifest file (plugin.toml) with metadata
- Plugin lifecycle: load, init, enable, disable, unload
- Plugin API version for compatibility checking

#### FR-P02: Plugin Capabilities
- Register new protocol handlers
- Add custom tab types
- Add menu items and toolbar buttons
- Hook into terminal data stream (input/output filters)
- Register custom network tools
- Add sidebar panels

#### FR-P03: Plugin Management
- Built-in plugin manager UI
- Enable/disable plugins without restart
- Plugin settings UI (auto-generated from schema)

---

## 3. Non-Functional Requirements

### 3.1 Performance

| Metric                          | Target                    |
|---------------------------------|---------------------------|
| Cold startup time               | < 1.5 seconds             |
| Session connect time (SSH)      | < 3 seconds (LAN)        |
| Terminal throughput              | > 50 MB/s rendering       |
| File transfer throughput        | > 80% of raw bandwidth   |
| Memory usage (idle, 1 tab)      | < 100 MB                 |
| Memory per additional tab       | < 20 MB                  |
| Maximum concurrent sessions     | 50+                       |
| UI frame rate                   | 60 FPS                    |

### 3.2 Reliability

- Crash recovery: restore previous session tabs on restart
- Connection auto-reconnect with exponential backoff
- Graceful degradation on plugin failure (isolate plugin crash)
- Data integrity: no credential loss on unexpected shutdown
- Automatic config backup on every modification

### 3.3 Security

- All remote connections encrypted where protocol supports it
- Credential store encrypted at rest (AES-256-GCM)
- No telemetry or data collection
- No automatic update check without user consent
- Strict host key verification by default
- Certificate pinning for FTPS connections
- Plugin sandboxing with restricted filesystem access

### 3.4 Usability

- Keyboard-navigable (all features accessible without mouse)
- Screen reader support (ARIA labels)
- Right-click context menus throughout
- Undo for destructive operations where possible
- Tooltips on all toolbar buttons and icons
- First-run wizard for initial configuration
- Comprehensive keyboard shortcut reference (Ctrl+?)

### 3.5 Compatibility

| Platform       | Minimum Version          | Notes                     |
|----------------|--------------------------|---------------------------|
| Windows        | Windows 10 (1809+)       | ConPTY required           |
| Rocky Linux    | 8.x                     | glibc 2.28+               |
| Display Server | X11, Wayland (XWayland)  | Native Wayland planned    |

### 3.6 Localization

- UI language: English (default)
- UTF-8 terminal encoding support
- CJK character width handling
- RTL text display in terminal
- Locale-aware date/time formatting

---

## 4. Data Storage

### 4.1 Configuration Files

```
Linux:   ~/.config/rustxterm/
Windows: %APPDATA%\rustxterm\

├── config.toml              # Application settings
├── sessions.db              # SQLite: sessions, groups, tunnels
├── credentials.enc          # Encrypted credential store
├── known_hosts              # SSH host key database
├── macros/                  # Saved terminal macros
│   └── *.macro.json
├── themes/                  # Custom themes
│   └── *.theme.json
├── plugins/                 # Installed plugins
│   └── <plugin-name>/
│       ├── plugin.toml
│       └── lib.{so,dll}
├── keys/                    # SSH keys (encrypted)
│   ├── id_rsa.enc
│   └── id_ed25519.enc
└── logs/                    # Application logs
    └── rustxterm.log
```

### 4.2 config.toml Schema

```toml
[general]
language = "en"
theme = "dark"
check_updates = false
auto_save_sessions = true
confirm_on_exit = true
restore_sessions_on_start = true

[terminal]
font_family = "JetBrains Mono"
font_size = 14
scrollback_lines = 10000
cursor_style = "block"         # block, underline, bar
cursor_blink = true
bell = "visual"                # visual, audio, none
copy_on_select = false
paste_on_right_click = true
word_separator = " /\\()\"'-.,:;<>~!@#$%^&*|+=[]{}~?"

[terminal.colors]
preset = "dark"                # or custom
# Custom colors override:
# foreground = "#d4d4d4"
# background = "#1e1e1e"
# cursor = "#ffffff"
# selection = "#264f78"
# black = "#000000"
# ... (full 16-color ANSI palette)

[ssh]
default_terminal_type = "xterm-256color"
keepalive_interval = 60
compression = false
strict_host_key_checking = true
preferred_auth = ["publickey", "keyboard-interactive", "password"]

[file_transfer]
max_concurrent = 3
resume_transfers = true
confirm_overwrite = true
preserve_timestamps = true

[security]
master_password_timeout = 30   # minutes, 0 = never
clipboard_clear_timeout = 30   # seconds, 0 = disabled
use_os_keychain = true

[ui]
sidebar_position = "left"      # left, right
sidebar_width = 280
sidebar_auto_hide = false
show_toolbar = true
tab_position = "top"           # top, bottom
window_width = 1200
window_height = 800
```

### 4.3 SQLite Database Schema

```sql
CREATE TABLE session_groups (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,
    parent_id   INTEGER REFERENCES session_groups(id),
    sort_order  INTEGER DEFAULT 0,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE sessions (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT NOT NULL,
    group_id        INTEGER REFERENCES session_groups(id),
    protocol        TEXT NOT NULL,    -- ssh, telnet, rdp, vnc, ftp, serial, shell
    host            TEXT,
    port            INTEGER,
    username        TEXT,
    credential_id   INTEGER REFERENCES credentials(id),
    config_json     TEXT,             -- Protocol-specific config as JSON
    color_tag       TEXT,
    notes           TEXT,
    is_favorite     BOOLEAN DEFAULT 0,
    auto_connect    BOOLEAN DEFAULT 0,
    sort_order      INTEGER DEFAULT 0,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_connected  DATETIME
);

CREATE TABLE credentials (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT NOT NULL,
    username        TEXT,
    encrypted_data  BLOB NOT NULL,    -- AES-256-GCM encrypted
    key_type        TEXT,             -- password, private_key
    nonce           BLOB NOT NULL,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE port_forwards (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id      INTEGER REFERENCES sessions(id) ON DELETE CASCADE,
    direction       TEXT NOT NULL,    -- local, remote, dynamic
    local_host      TEXT DEFAULT '127.0.0.1',
    local_port      INTEGER NOT NULL,
    remote_host     TEXT,
    remote_port     INTEGER,
    auto_start      BOOLEAN DEFAULT 1,
    description     TEXT
);

CREATE TABLE known_hosts (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    hostname        TEXT NOT NULL,
    port            INTEGER NOT NULL DEFAULT 22,
    key_type        TEXT NOT NULL,
    public_key      TEXT NOT NULL,
    fingerprint     TEXT NOT NULL,
    first_seen      DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_seen       DATETIME DEFAULT CURRENT_TIMESTAMP,
    trusted         BOOLEAN DEFAULT 1,
    UNIQUE(hostname, port, key_type)
);

CREATE TABLE macros (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT NOT NULL UNIQUE,
    description     TEXT,
    commands_json   TEXT NOT NULL,    -- Array of {input, delay_ms}
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE recent_connections (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    protocol        TEXT NOT NULL,
    host            TEXT NOT NULL,
    port            INTEGER,
    username        TEXT,
    connected_at    DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_sessions_group ON sessions(group_id);
CREATE INDEX idx_sessions_protocol ON sessions(protocol);
CREATE INDEX idx_port_forwards_session ON port_forwards(session_id);
CREATE INDEX idx_recent_connections_time ON recent_connections(connected_at DESC);
```

---

## 5. UI Wireframe Specification

### 5.1 Main Window Layout

```
┌─────────────────────────────────────────────────────────────────────┐
│ Menu Bar: [File] [Edit] [Sessions] [View] [Tools] [Macros] [Help]  │
├─────────────────────────────────────────────────────────────────────┤
│ Toolbar: [+New] [Connect] [Disconnect] [SFTP] [Split] [MultiExec] │
├────────────────┬────────────────────────────────────────────────────┤
│                │ Tab Bar: [Local Shell] [Server1-SSH] [Server2] [+]│
│   Sidebar      ├────────────────────────────────────────────────────┤
│                │                                                    │
│  ┌──────────┐  │                                                    │
│  │ Sessions │  │                                                    │
│  │  ├─ Prod │  │              Terminal Area                         │
│  │  │  ├─A  │  │                                                    │
│  │  │  └─B  │  │         (xterm.js rendering)                       │
│  │  ├─ Dev  │  │                                                    │
│  │  │  └─C  │  │                                                    │
│  │  └─ Test │  │                                                    │
│  └──────────┘  │                                                    │
│                │                                                    │
│  ┌──────────┐  │                                                    │
│  │ SFTP     │  │                                                    │
│  │ Browser  │  │                                                    │
│  │  /home/  │  │                                                    │
│  │  ├─docs/ │  │                                                    │
│  │  ├─.ssh/ │  │                                                    │
│  │  └─app/  │  │                                                    │
│  └──────────┘  │                                                    │
│                ├────────────────────────────────────────────────────┤
│  [Sessions]    │ Status: Connected | user@host | UTF-8 | 80x24     │
│  [SFTP]        │                                                    │
│  [Tools]       │                                                    │
├────────────────┴────────────────────────────────────────────────────┤
│ Status Bar: Ready | 3 sessions active | Transfer: 2.3 MB/s         │
└─────────────────────────────────────────────────────────────────────┘
```

### 5.2 Session Dialog

```
┌─────────────── New Session ────────────────────┐
│                                                 │
│  Session Type:  [SSH ▼]                        │
│                                                 │
│  ── Connection ──────────────────────────────  │
│  Host:     [________________________]          │
│  Port:     [22___]                             │
│  Username: [________________________]          │
│                                                 │
│  ── Authentication ──────────────────────────  │
│  Method:   [Password ▼]                       │
│  Password: [________________________] [👁]    │
│   -or-                                         │
│  Key File: [________________________] [Browse] │
│                                                 │
│  ── Advanced ────────────────────────────────  │
│  Terminal Type: [xterm-256color___]            │
│  Encoding:      [UTF-8___________]            │
│  SSH Gateway:   [None ▼]                      │
│  [✓] X11 Forwarding                           │
│  [ ] Agent Forwarding                          │
│  [ ] Compression                               │
│                                                 │
│  ── Startup Commands ────────────────────────  │
│  [                                    ]        │
│  [                                    ]        │
│                                                 │
│  Session Name: [________________________]      │
│  Group:        [Production ▼]                  │
│                                                 │
│        [Cancel]  [Test Connection]  [OK]       │
└─────────────────────────────────────────────────┘
```

### 5.3 SSH Tunnel Builder

```
┌─────────────── SSH Tunnels ────────────────────┐
│                                                 │
│  Session: Server1-SSH                          │
│                                                 │
│  ┌───┬──────┬───────────┬───────┬────────┬───┐ │
│  │ # │ Type │ Local     │ Remote│ Status │ X │ │
│  ├───┼──────┼───────────┼───────┼────────┼───┤ │
│  │ 1 │ L    │ :8080     │ :80   │ Active │ ✕ │ │
│  │ 2 │ L    │ :3306     │ :3306 │ Active │ ✕ │ │
│  │ 3 │ D    │ :1080     │ SOCKS │ Idle   │ ✕ │ │
│  └───┴──────┴───────────┴───────┴────────┴───┘ │
│                                                 │
│  [+ Add Tunnel]                                │
│                                                 │
│  Direction: [Local (L) ▼]                      │
│  Listen:    [127.0.0.1]:[8080]                 │
│  Forward:   [db.internal]:[3306]               │
│                                                 │
│              [Cancel]  [Apply]                  │
└─────────────────────────────────────────────────┘
```

---

## 6. Keyboard Shortcuts

### 6.1 Global Shortcuts

| Action                  | Shortcut              |
|-------------------------|-----------------------|
| New local terminal      | Ctrl+T                |
| Close tab               | Ctrl+W                |
| Next tab                | Ctrl+Tab              |
| Previous tab            | Ctrl+Shift+Tab        |
| Go to tab N             | Ctrl+1 ... Ctrl+9     |
| New session dialog      | Ctrl+N                |
| Quick connect           | Ctrl+Shift+N          |
| Toggle sidebar          | Ctrl+B                |
| Toggle fullscreen       | F11                   |
| Settings                | Ctrl+,                |
| Keyboard shortcuts help | Ctrl+?                |

### 6.2 Terminal Shortcuts

| Action                  | Shortcut              |
|-------------------------|-----------------------|
| Copy                    | Ctrl+Shift+C          |
| Paste                   | Ctrl+Shift+V          |
| Search                  | Ctrl+Shift+F          |
| Clear terminal          | Ctrl+L (shell built-in)|
| Zoom in                 | Ctrl+Plus             |
| Zoom out                | Ctrl+Minus            |
| Reset zoom              | Ctrl+0                |
| Split horizontal        | Ctrl+Shift+H          |
| Split vertical          | Ctrl+Shift+E          |
| Close pane              | Ctrl+Shift+W          |
| Navigate pane           | Alt+Arrow             |
| Toggle multi-exec       | Ctrl+Shift+M          |

### 6.3 File Browser Shortcuts

| Action                  | Shortcut              |
|-------------------------|-----------------------|
| Upload file             | Ctrl+U                |
| Download file           | Ctrl+D                |
| Refresh                 | F5                    |
| Delete                  | Delete                |
| Rename                  | F2                    |
| New folder              | Ctrl+Shift+D          |
| Toggle hidden files     | Ctrl+H                |

---

## 7. Development Phases

### Phase 1: Foundation (MVP)
**Goal**: Basic terminal emulator with local shell and SSH

- [ ] Project setup: Tauri + React + Cargo workspace
- [ ] Local terminal with xterm.js + PTY backend
- [ ] Multi-tab support
- [ ] SSH connection (password + key auth)
- [ ] Basic session management (create, save, connect)
- [ ] SFTP file browser sidebar (read-only browsing)
- [ ] Application settings (config.toml)
- [ ] Credential storage with master password
- [ ] Basic theming (dark/light)

### Phase 2: Core Features
**Goal**: Full-featured SSH workflow

- [ ] SFTP file transfer (upload/download with progress)
- [ ] Remote file editing
- [ ] SSH key management (generate, import)
- [ ] SSH tunnels / port forwarding
- [ ] SSH gateway / jump host
- [ ] Split pane terminals
- [ ] Session import/export
- [ ] Session folders and organization
- [ ] Quick connect dialog
- [ ] Terminal search

### Phase 3: Protocol Expansion
**Goal**: Support all major protocols

- [ ] Telnet client
- [ ] FTP/FTPS client
- [ ] Serial port connection
- [ ] RDP client (via IronRDP)
- [ ] VNC client
- [ ] Multi-execution mode
- [ ] Terminal macros

### Phase 4: Advanced Features
**Goal**: Power user features and polish

- [ ] Network tools (port scanner, ping, traceroute, DNS)
- [ ] Custom themes editor
- [ ] Plugin system framework
- [ ] Keyboard shortcut customization
- [ ] SSH config import
- [ ] PuTTY session import (Windows)
- [ ] X11 forwarding support
- [ ] First-run wizard

### Phase 5: Polish & Distribution
**Goal**: Production-ready release

- [ ] Performance optimization
- [ ] Accessibility audit
- [ ] Windows MSI installer
- [ ] RPM package for Rocky Linux
- [ ] AppImage for generic Linux
- [ ] Documentation (user guide)
- [ ] Crash recovery / session restore
- [ ] Auto-update mechanism (optional)

---

## 8. Testing Strategy

| Test Type        | Tool/Framework         | Scope                              |
|------------------|------------------------|------------------------------------|
| Unit Tests       | Rust built-in + mockall| Core crates, protocol handlers     |
| Integration Tests| Rust integration tests | SSH connection, SFTP operations    |
| Frontend Tests   | Vitest + Testing Lib   | React components, state management |
| E2E Tests        | Tauri + Playwright     | Full application workflows         |
| Performance      | criterion              | Terminal throughput, transfer speed |
| Security         | cargo-audit            | Dependency vulnerability scanning  |

---

## 9. Risk Analysis

| Risk                              | Impact | Probability | Mitigation                           |
|-----------------------------------|--------|-------------|--------------------------------------|
| RDP pure-Rust immaturity          | Medium | High        | IronRDP actively developed; fallback to system rdesktop |
| Terminal emulation edge cases     | Medium | Medium      | Use xterm.js (proven); extensive vttest validation |
| Cross-platform PTY differences    | Medium | Medium      | portable-pty crate handles abstraction; test on both OSes |
| Plugin system security            | High   | Low         | Strict API surface; filesystem sandboxing; review process |
| Performance with many sessions    | Medium | Low         | Lazy loading; async I/O; connection pooling |
| WebView rendering inconsistency  | Low    | Medium      | Tauri v2 uses system webview; test on target platforms |
