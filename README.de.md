# Audio-Konverter

[English](README.md)

Windows-Programm (Rust, egui), das ganze Ordner mit Audiodateien umwandelt, zum Beispiel für den Pioneer XDJ-700 (spielt AIFF und MP3, kein FLAC). Eigener Konverter eingebaut, kein ffmpeg nötig. Oberfläche auf Deutsch und Englisch.

```mermaid
flowchart LR
    A["Ordner hinzufügen"] --> B["Dateien suchen"] --> C["Ziel wählen"] --> D["Parallel umwandeln"] --> E["Zusammenfassung und CSV-Protokoll"]
```

## Funktionen

- Ordner per Knopf (mehrere auf einmal), Drag & Drop oder Kommandozeile; Unterordner werden durchsucht, versteckte Dateien und Ordner übersprungen
- Quellen: FLAC, WAV, AIFF; Ziel **AIFF** (16 Bit, 44,1 kHz), **MP3** (320 kbit/s, 44,1 kHz) oder **FLAC** (16 Bit, 44,1 kHz)
- Kompakte Tabelle: pro Ordner eine Kopfzeile (Pfad, Dateianzahl), darunter Häkchen, Nr, Datei, Format, Status mit Icon; alle an/aus im Kopf
- Mehrere Dateien gleichzeitig (ein Thread pro CPU-Kern), Abbrechen jederzeit
- Ergebnis neben der Quelle; Originale und vorhandene Dateien bleiben unverändert
- Tags werden übernommen; das erste lesbare Bild eines Ordners wird als Cover eingebettet (AIFF, MP3 und FLAC), größer als 700 x 700 Pixel wird auf 700 verkleinert
- Fenster „Cover“: Ordner ohne Cover und alle gefundenen Bilder
- Sprache Deutsch oder Englisch und Thema (System, Hell, Dunkel) oben rechts; Zielformat, Fenstergröße und Thema werden gemerkt
- Nach dem Lauf: „Zielordner öffnen“ und CSV-Protokoll; sanfte Übergänge, Lucide-Icons

## Voraussetzungen

- Windows 11
- [Rust](https://rustup.rs) mit den C++ Build Tools (C-Compiler für den eingebauten MP3-Encoder)

## Starten

```
git clone <repo-url>
cd audio-konverter
cargo run --release
```

Für echte Umwandlungen immer `--release`. Tests: `cargo test --release`. Fertige Datei: `cargo build --release`, Ergebnis `target\release\audio-konverter.exe` (eine Datei, ohne Rust und ohne Installation lauffähig). Ohne Rust auf dem PC: auf GitHub unter *Actions → Windows-Build → Run workflow*, die `.exe` ist der Download des Laufs; ein Tag wie `v0.4.0` erzeugt zusätzlich ein Release.

Vor dem Commit prüfen: `prek install` (einmal), siehe `wiki/pruefungen.md`.

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
│   ├── einstellungen.rs  Einstellungen merken (Zielformat, Thema, Fenster)
│   └── bericht.rs    CSV-Protokoll
└── wiki/             Funktionen, Entscheidungen, Review, Diagramme
```

## Geplant

- Später macOS und Linux

## Lizenz

Copyright © 2026 Alexander Singh. [GPL-3.0-or-later](LICENSE). Freie Software: nutzen, ändern und weitergeben ist erlaubt; geänderte Versionen müssen unter derselben Lizenz bleiben. Enthalten sind der MP3-Encoder LAME (LGPL) und weitere offene Bibliotheken unter ihren eigenen Lizenzen.

Details im [Wiki](wiki/index.md). Immer mit Kopien der Dateien testen.
