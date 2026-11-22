# Noch zu integrieren

```mermaid
flowchart TD
    E["1 Design B und C (später)"]
    F["2 PR #1 aufteilen und prüfen"]
```

## Offen

| Nr. | Punkt | Hinweis |
|---|---|---|
| 1 | Design-Punkte B und C aus `review.md` | Auf später verschoben (Sortierung, Suche, Einstellungen merken, Programm-Icon, Hintergrund-Scan, Design-Umschalter, Animationen) |
| 2 | Pull Request #1 prüfen: https://github.com/hhalex660/audio-konverter/pull/1 | Erst nach Release 0.4.4, nicht blind mergen. Punkte: (a) Kompilierfehler: `map_or_default` in `src/app.rs:112` und `src/scan.rs:19` ist auf stabilem Rust nicht verfügbar, stabil ersetzen durch `map(…).unwrap_or_default()`; (b) zu viel auf einmal: mise, cargo-packager, Universal-macOS und Release-Kette gehören zu Schritt 5 des Plans, nicht zu 0.4.4; (c) Abhängigkeiten `rfd` 0.15 → 0.18, `mp3lame-encoder` 0.1 → 0.3, `id3`, `walkdir` einzeln prüfen und MP3-Klang anhören; (d) Linux-Basis: PR setzt Ubuntu 24.04 (Mint 22) voraus, Plan sieht Mint 21 vor. Entscheiden; (e) Tests: Testordner werden nach dem Lauf gelöscht (vorher blieben sie zur Kontrolle). Übernehmen, wenn gewünscht; (f) MP3-Test zählt Nulldurchgänge in festem Bereich (345 bis 359), kann bei Encoder-Updates brechen |

## Erledigt

| Nr. | Punkt | Modul |
|---|---|---|
| 1 | Protokoll als CSV | `bericht.rs` |
| 2 | Cover in AIFF-Metadaten | `app.rs`, `tags.rs` |
| 3 | Bildersuche | `scan.rs` |
| 4 | Fenster „Cover“ (vorher „Ordner ohne Bild“) | `gui.rs` |
| 5 | Eigener Konverter | `konverter.rs` |
| 6 | Standalone-Datei (`cargo build --release`, Release mit LTO, C-Laufzeit eingebaut, Build über GitHub Actions) | `Cargo.toml`, `.cargo/`, `.github/workflows/` |
| 7 | Aufteilung in Module | `main.rs` u. a. |
| 8 | DJ-Player-Auswahl: nicht gebaut, siehe `entscheidungen.md` | – |
| 9 | Cover auch in MP3-Metadaten | `app.rs` |
| 10 | MP3-Umwandlung beschleunigen (60 Tracks WAV/FLAC nach MP3: 34 s) | `auftraege.rs` |
| 11 | Umwandlung nach FLAC (16 Bit, 44,1 kHz) | `konverter.rs`, `flac.rs`, `tags.rs` |
| 12 | Sprachauswahl Deutsch / Englisch | `texte.rs`, `gui.rs` |
| 13 | Release 0.4.4 mit Windows-`.exe` (Lints, Pre-Commit-Hooks, `deny.toml`, Doku) | `Cargo.toml`, `.github/`, `wiki/pruefungen.md` |
| 14 | Cover auf höchstens 700 × 700 Pixel verkleinern (AIFF, MP3, FLAC); im Windows Media Player geprüft | `tags.rs`, `Cargo.toml` |
| 15 | FLAC im Windows Media Player: Fehler war das 3000-Pixel-Cover. Abgespielt und Cover angezeigt (Test mit umbenannter Datei) | – |
| 16 | `cargo machete` ausgeführt: keine ungenutzten Abhängigkeiten (Version 0.9.1); läuft nur von Hand, nicht in der CI | `wiki/pruefungen.md` |
| 17 | Git-Hooks (prek) auf Windows: `rustfmt` bricht schlechten Commit ab | `.pre-commit-config.yaml`, `wiki/pruefungen.md` |

Zu Punkt 6: `Cargo.toml` (Release mit `lto`, `codegen-units = 1`, `strip`), `.cargo/config.toml` (C-Laufzeit fest eingebaut, keine Visual-C++-Runtime nötig), `.github/workflows/windows.yml` (baut und testet auf GitHub, `.exe` als Download, bei Tag `v*` als Release). Im Sandbox-Test (Linux, 12 Dateien à 30 s, 96 kHz/24 Bit): AIFF 0,3 s, FLAC 0,8 s, MP3 2,3 s; alle 18 Tests grün. Die Windows-`.exe` wurde auf deinem PC gebaut und von dir als erledigt bestätigt.

Später: Builds für macOS und Linux Mint über GitHub Actions.
