# Audio Converter

[Deutsch](README.de.md)

Windows desktop app (Rust, egui) that converts whole folders of audio files, for example for the Pioneer XDJ-700 (plays AIFF and MP3, no FLAC). Own converter built in, no ffmpeg needed. User interface in German and English.

```mermaid
flowchart LR
    A["Add folders"] --> B["Find files"] --> C["Choose target"] --> D["Convert in parallel"] --> E["Summary and CSV log"]
```

## Features

- Add folders via button (several at once), drag and drop or command line; subfolders are scanned
- Sources: FLAC, WAV, AIFF; target **AIFF** (16 bit, 44.1 kHz), **MP3** (320 kbit/s, 44.1 kHz) or **FLAC** (16 bit, 44.1 kHz)
- Compact table: checkbox, number, file, short folder path, format, status with icon; select all in the header
- Several files at once (one thread per CPU core), cancel at any time
- Output next to the source; originals and existing files are never touched
- Tags are copied; the first image of a folder is embedded as cover (AIFF, MP3 and FLAC)
- "Cover" window: folders without cover and all found images
- Language: German or English (menu top right, remembered), CSV log after a run, light and dark theme, Lucide icons

## Requirements

- Windows 11
- [Rust](https://rustup.rs) with the C++ Build Tools (C compiler for the built-in MP3 encoder)

## Run

```
git clone <repo-url>
cd audio-konverter
cargo run --release
```

Always use `--release` for real conversions. Tests: `cargo test --release`. Standalone build: `cargo build --release`, result `target\release\audio-konverter.exe` (single file, no Rust, no installation needed). Without Rust on your PC: use the GitHub Actions build described below.

Checks before committing: `prek install` (once), see `wiki/pruefungen.md`.

### Development with mise

[mise](https://mise.jdx.dev/getting-started.html) is optional for local development. It installs the project's pinned Rust toolchain using rustup; the platform's C compiler and native libraries are still required. From the project directory:

```sh
mise install
mise run test   # Rust release tests
mise run check  # formatting and Clippy
mise run build  # optimized binary
mise run run    # launch the app
```

`mise tasks` lists the available commands. GitHub Actions uses the same pinned Rust version and tasks. The release-version script tests additionally require Python 3.11 or newer: `python3 -m unittest discover -s tests`.

## Cross-platform builds (experimental)

GitHub Actions → **Build and release** → **Run workflow** builds Windows x64, Linux x64 and a universal macOS package. Download the artifacts from the run. Pushing a `v*` tag creates one GitHub release after all three builds succeed; the tag must match the version in `Cargo.toml`.

Linux packages target Ubuntu 24.04 / Linux Mint 22 or newer, not Mint 21. The `.deb` installs dependencies; the `.tar.gz` is dynamically linked and needs desktop libraries and an XDG file-dialog portal. The macOS `.dmg` is not notarized and may be blocked by Gatekeeper. The macOS and Windows build workflows and packaged apps still need to be run and checked on those systems.

## Project structure

```
audio-konverter/
├── Cargo.toml, Cargo.lock
├── assets/icons/     Lucide icons (SVG, ISC)
├── src/
│   ├── main.rs       Start
│   ├── app.rs        State, flow
│   ├── gui.rs        User interface
│   ├── icons.rs      Icons
│   ├── scan.rs       Find audio and image files
│   ├── auftraege.rs  Parallel jobs in the background
│   ├── konverter.rs  Own converter: read, resample, 16 bit
│   ├── aiff.rs       AIFF writer
│   ├── mp3.rs        MP3 writer (LAME)
│   ├── flac.rs       FLAC writer (flacenc)
│   ├── tags.rs       Tags and cover (ID3, Vorbis)
│   ├── texte.rs      Texts in German and English
│   └── bericht.rs    CSV log
└── wiki/             features, decisions, review, diagrams (German)
```

## License

Copyright © 2026 Alexander Singh. [GPL-3.0-or-later](LICENSE). Free software: you may use, change and share it; changed versions must stay under the same license. It contains the MP3 encoder LAME (LGPL) and other open libraries under their own licenses.

Details in the [wiki](wiki/index.md). Always test with copies of your files.
