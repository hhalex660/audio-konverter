# Noch zu integrieren

```mermaid
flowchart TD
    E["1 Design B Rest (später)"]
    F["2 PR #1 aufteilen und prüfen"]
```

## Offen

| Nr. | Punkt | Hinweis |
|---|---|---|
| 1 | Design-Punkte B aus `review.md`, die noch fehlen | Spalten sortieren und in der Breite ziehen, Suche/Filter, Ordner im Hintergrund durchsuchen |
| 2 | Linux-Build für Linux Mint: zuerst Entscheidung Mint 21 oder 22, dann `.deb` mit `cargo-packager` über GitHub Actions. Im PR #1 gibt es dafür einen Vorschlag (`mise.toml`, Workflow), den wir nicht 1:1 übernehmen | `windows.yml` (neuer Workflow), `Cargo.toml` |
| 3 | macOS-Build: `.dmg` mit `cargo-packager` (Universal-Build), über GitHub Actions. Erst nach Linux-Build | `windows.yml` (neuer Workflow), `Cargo.toml` |
| 4 | `unsafe` in `mp3.rs` entfernen | `mp3lame-encoder` 0.3 hat `encode_to_vec` und `flush_to_vec`; MP3-Ausgabe vorher und nachher vergleichen (Dekodierung war bei der Aktualisierung 0,3 identisch) |
| 5 | (optional) MP3-Pegel vor dem Encoder um 1 dB senken, gegen Übersteuerung bei sehr lauten Stücken. Nur bei hörbarer Verzerrung nötig (Messung: Ausschnitt „Lizard Brain“ 669 Samples auf Vollpegel, Vermont/Fragfest 0). Nachteil: MP3 etwas leiser als AIFF/FLAC | `mp3.rs`, `konverter.rs` |
| 6 | (optional) LAME-Gapless-Header einbauen: MP3 beginnt und endet exakt wie die Quelle (aktuell ca. 25 ms Versatz am Anfang, 17 ms länger). Nur für millimetergenaue Cue-Punkte relevant. Mittlerer Aufwand, mit mehreren Playern testen | `mp3.rs` |

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
| 18 | Einstellungen merken, Zielordner öffnen, Themenwahl (System, Hell, Dunkel), sanfte Übergänge | `einstellungen.rs`, `app.rs`, `gui.rs`, `texte.rs` |
| 19 | Dateiliste mit Ordner-Kopfzeilen statt Streifen und Ordner-Spalte (gut sichtbar in Hell und Dunkel) | `app.rs`, `gui.rs`, `texte.rs` |
| 20 | Programm-Icon (Schallplatte) in der Windows-`.exe` und im Fenster (Titelleiste, Taskleiste) | `build.rs`, `assets/icon.ico`, `assets/icon.png`, `main.rs`, `Cargo.toml` |
| 21 | Release-Tag wird gegen die Version in `Cargo.toml` geprüft | `scripts/check-release-version.py`, `tests/test_release_version.py`, `windows.yml` |
| 22 | Abhängigkeiten aktualisiert: `rfd` 0.18, `mp3lame-encoder` 0.3; Mindestversionen `walkdir` 2.5, `id3` 1.17 (MP3 vor und nach dem Update: gleiche Größe, 320 kbit/s, Dekodierung identisch) | `Cargo.toml`, `Cargo.lock` |
| 23 | Test-Ordner mit `tempfile`: eigener Ordner pro Test, wird gelöscht | `konverter.rs`, `app.rs`, `Cargo.toml` |
| 24 | Prüfung: Cover-Tests räumen ihre Testbilder auf | `tags.rs` |
| 25 | Audit-Fixes: Absturz im Konverter löscht die halbfertige Zieldatei; nur lesbare Bilder werden Cover (MIME-Typ aus dem Inhalt); versteckte Dateien und Ordner werden übersprungen; ungültige Fenstergröße (`NaN`) wird ignoriert | `konverter.rs`, `tags.rs`, `scan.rs`, `einstellungen.rs` |
| 26 | PR #1 (Senci) geschlossen, nicht gemergt. Übernommen: Versionsprüfung (Skript und Test unverändert), Abhängigkeiten, Test-Ordner mit `tempfile`. Senci im Kommentar am PR genannt, Gutschrift in den Release-Notizen. Nicht übernommen: MP3-Nulldurchgangstest, `mise` (vorerst), Zeitlimit im Test | `scripts/`, `tests/`, `Cargo.toml`, `windows.yml` |

Zu Punkt 6: `Cargo.toml` (Release mit `lto`, `codegen-units = 1`, `strip`), `.cargo/config.toml` (C-Laufzeit fest eingebaut, keine Visual-C++-Runtime nötig), `.github/workflows/windows.yml` (baut und testet auf GitHub, `.exe` als Download, bei Tag `v*` als Release). Im Sandbox-Test (Linux, 12 Dateien à 30 s, 96 kHz/24 Bit): AIFF 0,3 s, FLAC 0,8 s, MP3 2,3 s; alle 22 Tests grün. Die Windows-`.exe` wurde auf deinem PC gebaut und von dir als erledigt bestätigt.

Später: Builds für macOS und Linux Mint über GitHub Actions.
