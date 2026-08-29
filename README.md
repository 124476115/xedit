# Code Editor (Rust)

Cross-platform code editor for Markdown, XML, JSON, .env, .properties files.
Compiles to a single binary.

## Features

- **Syntax highlighting** for 5 formats (via syntect)
- **Multi-tab** editing
- **File operations**: New, Open, Save, Save As
- **Auto-detect** file type by extension/content
- **Dark/Light themes**
- **Keyboard shortcuts**: Ctrl+N/O/S/W, Ctrl+Tab
- **Single binary** distribution (~5-10 MB)

## Supported Formats

| Format | Extensions | Icon |
|--------|------------|------|
| Markdown | .md, .markdown, .mdown | 📝 |
| XML | .xml, .xsd, .xsl, .svg | 📄 |
| JSON | .json, .jsonc, .json5 | 📋 |
| Environment | .env, .env.* | 🔐 |
| Properties | .properties, .props, .ini | ⚙️ |

## Building

### Prerequisites

Install Rust: https://rustup.rs/

```bash
# Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Windows (PowerShell)
winget install Rustlang.Rust.GNU
# or download from rustup.rs
```

### Build

```bash
# Linux/macOS
./build.sh

# Windows
build.bat
```

Or directly:
```bash
cargo build --release
```

### Output

Single binary at:
- `target/release/code-editor` (Linux/macOS)
- `target/release/code-editor.exe` (Windows)

## Usage

```bash
code-editor [file1] [file2] ...
```

Drag files onto the binary to open them.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Ctrl+N | New file |
| Ctrl+O | Open file |
| Ctrl+S | Save |
| Ctrl+Shift+S | Save As |
| Ctrl+W | Close tab |
| Ctrl+Tab | Next tab |
| Ctrl+Z | Undo |
| Ctrl+Y | Redo |

## Cross-compilation

```bash
# Windows from Linux
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu

# macOS from Linux (requires osxcross)
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin

# Linux from macOS/Windows
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu
```

## License

MIT