# Prüfungen

Automatische Prüfungen des Codes: lokal vor jedem Commit (`prek`) und in GitHub Actions bei Pull Requests und Pushes auf `main`. Der Release-Workflow bleibt separat (siehe [release-plan.md](release-plan.md)).

```mermaid
flowchart LR
    A["git commit"] --> B["rustfmt<br/>Format"] --> C["clippy<br/>-D warnings"] --> D["committed<br/>Commit-Text"] --> E["Commit gespeichert"]
    B & C & D -.->|Fehler| X["Commit abgebrochen"]
```

| Werkzeug | Datei | Prüft | Wann |
|---|---|---|---|
| `rustfmt` | – | einheitliche Formatierung | Hook |
| `clippy` | `[lints]` in `Cargo.toml` | Fehler, unnötiger Code; Gruppen `all` + `pedantic`; Zahlen-Casts erlaubt (Audio-Rechnung) | Hook |
| `committed` | `committed.toml` | Commit-Text im Format `feat: …`, `fix: …` (Voraussetzung für semantic-release) | Hook (commit-msg) |
| `cargo-deny` | `deny.toml` | Lizenzen (müssen zu GPL-3.0-or-later passen), bekannte Sicherheitslücken, fremde Quellen | CI und von Hand |

Die CI prüft Rust-Formatierung, Clippy und die Release-Tests auf Linux, macOS und Windows. Separat prüft sie die Python-Tests für Release-Tags und `cargo-deny`. Bei Fehlschlägen zeigt jeder Schritt seine eigene Ausgabe; es gibt keine Schreibrechte für Pull Requests. Zwei dokumentierte `quick-xml`-Ausnahmen in `deny.toml` bleiben bestehen.

## Einrichten (Windows)

```
winget install --id j178.Prek
cargo install --locked cargo-deny
prek install
prek run --all-files
```

`prek install` einmal pro Klon. Danach laufen die Hooks bei `git commit`; Rust- und Cargo-Manifeständerungen lösen Formatierung und Clippy aus. `prek run --all-files` prüft alles sofort. Hooks überspringen (nur im Notfall): `git commit --no-verify`.

Lizenzen und Lücken prüfen: `cargo deny check`.

## Hinweise

- `deny.toml` ignoriert zwei Meldungen zu `quick-xml` (RUSTSEC-2026-0194/0195): kommt nur unter Linux über `eframe` 0.29 (Bedienungshilfen), das Programm liest kein fremdes XML. Erledigt sich mit einem neueren `eframe`.
- Warnung „LGPL-3.0 veraltet“ kommt von den MP3-Bibliotheken und ist harmlos.
- Nicht übernommen (YAGNI): `typos` (liefert bei deutschen Texten rund 200 Fehlalarme), `rustfmt.toml` (Standard reicht) und `cargo-machete`.
- Hooks, `deny.toml` und `committed` wurden unter Linux geprüft, auf Windows noch nicht.
