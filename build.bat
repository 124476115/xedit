@echo off
echo Building Code Editor (Rust)...
cargo build --release
echo.
echo Binary location: target\release\code-editor.exe
echo Copy to desired location or run directly.