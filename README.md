# Audio Converter

[Deutsch](README.de.md)

Windows desktop app (Rust, egui) that converts whole folders of audio files, for example for the Pioneer XDJ-700 (plays AIFF and MP3, no FLAC). Own converter built in, no ffmpeg needed. User interface in German and English.

```mermaid
flowchart LR
    A["Add folders"] --> B["Find files"] --> C["Choose target"] --> D["Convert in parallel"] --> E["Summary and CSV log"]
```

## Features

- Add folders via button (several at once), drag and drop or command line; subfolders are scanned, hidden files and folders are skipped
- Sources: FLAC, WAV, AIFF; target **AIFF** (16 bit, 44.1 kHz), **MP3** (320 kbit/s, 44.1 kHz) or **FLAC** (16 bit, 44.1 kHz)
- Compact table: one header row per folder (path, file count), then checkbox, number, file, format, status with icon; select all in the header
- Several files at once (one thread per CPU core), cancel at any time
- Output next to the source; originals and existing files are never touched
- Tags are copied; the first readable image of a folder is embedded as cover (AIFF, MP3 and FLAC), scaled down to 700 x 700 pixels if larger
- "Cover" window: folders without cover and all found images
- Language (German or English) and theme (System, Light, Dark) top right; target format, theme and window size are remembered
- After a run: "Open target folder" and CSV log; smooth transitions, Lucide icons

## Requirements

- Windows 11
- [Rust](https://rustup.rs) with the C++ Build Tools (C compiler for the built-in MP3 encoder)

## Run

```
git clone <repo-url>
cd audio-konverter
cargo run --release
```

Always use `--release` for real conversions. Tests: `cargo test --release`. Standalone build: `cargo build --release`, result `target\release\audio-konverter.exe` (single file, no Rust, no installation needed). Without Rust on your PC: on GitHub under *Actions → Windows-Build → Run workflow*, the `.exe` is the download of the run; a tag like `v0.4.0` also creates a release.

Checks before committing: `prek install` (once), see `wiki/pruefungen.md`.

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
│   ├── einstellungen.rs  Remembers settings (target format, theme, window)
│   └── bericht.rs    CSV log
└── wiki/             features, decisions, review, diagrams (German)
```

## Planned

- Later macOS and Linux

## License

Copyright © 2026 Alexander Singh. [GPL-3.0-or-later](LICENSE). Free software: you may use, change and share it; changed versions must stay under the same license. It contains the MP3 encoder LAME (LGPL) and other open libraries under their own licenses.

Details in the [wiki](wiki/index.md). Always test with copies of your files.
