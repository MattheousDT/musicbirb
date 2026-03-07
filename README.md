# MusicBirb

A lightweight, thread-safe music player core written in Rust. It features a Subsonic-compatible API client, an MPV-based playback engine, and a TUI implementation with high-resolution album art support.

## Features

- 🚀 **Core Library:** Decoupled business logic, state management, and playback (designed for cross-platform/mobile use).
- 🎵 **Playback:** Powered by `libmpv` for rock-solid audio support.
- 🖼️ **Album Art:** Terminal-based high-res art rendering (supports Ghostty, Kitty, iTerm2, etc).

## Prerequisites

You need `mpv` / `libmpv` installed on your system. (at the minute)

- **Arch:** `sudo pacman -S mpv`
- **Fedora/RHEL:** `sudo dnf install mpv`
- **Ubuntu/Debian:** `sudo apt install mpv`
- **macOS:** `brew install mpv`
- **Windows:** 🤣

## Quick Start

1. **Configure Environment**
   Create a `.env` file in the root:

   ```sh
   SUBSONIC_URL = "https://music.example.com"
   SUBSONIC_USER = "your_username"
   SUBSONIC_PASS = "your_password"
   ```

2. **Run the UI**
   ```sh
   cargo run
   ```
