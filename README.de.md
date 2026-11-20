# Audio-Konverter

[English](README.md)

Windows-Programm (Rust, egui), das ganze Ordner mit Audiodateien umwandelt, zum Beispiel für den Pioneer XDJ-700 (spielt AIFF und MP3, kein FLAC). Eigener Konverter eingebaut, kein ffmpeg nötig. Oberfläche auf Deutsch und Englisch.

```mermaid
flowchart LR
    A["Ordner hinzufügen"] --> B["Dateien suchen"] --> C["Ziel wählen"] --> D["Parallel umwandeln"] --> E["Zusammenfassung und CSV-Protokoll"]
```

## Funktionen

- Ordner per Knopf (mehrere auf einmal), Drag & Drop oder Kommandozeile; Unterordner werden durchsucht
- Quellen: FLAC, WAV, AIFF; Ziel **AIFF** (16 Bit, 44,1 kHz), **MP3** (320 kbit/s, 44,1 kHz) oder **FLAC** (16 Bit, 44,1 kHz)
- Kompakte Tabelle: Häkchen, Nr, Datei, kurzer Ordnerpfad, Format, Status mit Icon; alle an/aus im Kopf
- Mehrere Dateien gleichzeitig (ein Thread pro CPU-Kern), Abbrechen jederzeit
- Ergebnis neben der Quelle; Originale und vorhandene Dateien bleiben unverändert
- Tags werden übernommen; das erste Bild eines Ordners wird als Cover eingebettet (AIFF, MP3 und FLAC)
- Fenster „Cover“: Ordner ohne Cover und alle gefundenen Bilder
- Sprache Deutsch oder Englisch (Auswahl oben rechts, wird gemerkt), CSV-Protokoll nach dem Lauf, helles und dunkles Thema, Lucide-Icons

## Voraussetzungen

- Windows 11
- [Rust](https://rustup.rs) mit den C++ Build Tools (C-Compiler für den eingebauten MP3-Encoder)

## Starten

```
git clone <repo-url>
cd audio-konverter
cargo run --release
```

Für echte Umwandlungen immer `--release`. Tests: `cargo test --release`. Fertige Datei: `cargo build --release`, Ergebnis `target\release\audio-konverter.exe` (eine Datei, ohne Rust und ohne Installation lauffähig). Ohne Rust auf dem PC: siehe GitHub Actions unten.

Vor dem Commit prüfen: `prek install` (einmal), siehe `wiki/pruefungen.md`.

### Entwicklung mit mise

[mise](https://mise.jdx.dev/getting-started.html) ist für die lokale Entwicklung optional. Es installiert die festgelegte Rust-Version über rustup; C-Compiler und native Bibliotheken der Plattform werden weiterhin benötigt. Im Projektordner:

```sh
mise install
mise run test   # Rust-Tests im Release-Modus
mise run check  # Formatierung und Clippy
mise run build  # optimiertes Programm bauen
mise run run    # Programm starten
```

`mise tasks` zeigt alle Befehle. GitHub Actions verwendet dieselbe festgelegte Rust-Version und dieselben Tasks. Die Tests des Release-Versionsskripts benötigen zusätzlich Python 3.11 oder neuer: `python3 -m unittest discover -s tests`.

## Plattformübergreifende Builds (experimentell)

GitHub Actions → **Build and release** → **Run workflow** baut Windows x64, Linux x64 und ein universelles macOS-Paket. Die Dateien stehen als Artifacts beim Lauf. Ein `v*`-Tag erstellt nach drei erfolgreichen Builds ein GitHub-Release; der Tag muss zur Version in `Cargo.toml` passen.

Die Linux-Pakete zielen auf Ubuntu 24.04 / Linux Mint 22 oder neuer, nicht Mint 21. Das `.deb` installiert Abhängigkeiten; die `.tar.gz`-Datei ist dynamisch gelinkt und benötigt Desktop-Bibliotheken und ein XDG-Dateidialog-Portal. Das macOS-`.dmg` ist nicht notarisiert und kann von Gatekeeper blockiert werden. Die Windows- und macOS-Workflows und Pakete müssen auf den jeweiligen Systemen noch geprüft werden.

## Projektstruktur

```
audio-konverter/
├── Cargo.toml, Cargo.lock
├── assets/icons/     Lucide-Icons (SVG, ISC)
├── src/
│   ├── main.rs       Start
│   ├── app.rs        Zustand, Ablauf
│   ├── gui.rs        Oberfläche
│   ├── icons.rs      Icons
│   ├── scan.rs       Audio- und Bilddateien finden
│   ├── auftraege.rs  Parallele Aufträge im Hintergrund
│   ├── konverter.rs  Eigener Konverter: lesen, umrechnen, 16 Bit
│   ├── aiff.rs       AIFF schreiben
│   ├── mp3.rs        MP3 schreiben (LAME)
│   ├── flac.rs       FLAC schreiben (flacenc)
│   ├── tags.rs       Tags und Cover (ID3, Vorbis)
│   ├── texte.rs      Texte auf Deutsch und Englisch
│   └── bericht.rs    CSV-Protokoll
└── wiki/             Funktionen, Entscheidungen, Review, Diagramme
```

## Lizenz

Copyright © 2026 Alexander Singh. [GPL-3.0-or-later](LICENSE). Freie Software: nutzen, ändern und weitergeben ist erlaubt; geänderte Versionen müssen unter derselben Lizenz bleiben. Enthalten sind der MP3-Encoder LAME (LGPL) und weitere offene Bibliotheken unter ihren eigenen Lizenzen.

Details im [Wiki](wiki/index.md). Immer mit Kopien der Dateien testen.
