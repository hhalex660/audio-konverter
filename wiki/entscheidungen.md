# Entscheidungen

| Entscheidung | Grund |
|---|---|
| Rust + egui | kleine, schnelle Einzeldatei |
| ffmpeg statt Audacity | Audacity nur per mod-script-pipe, instabil bei vielen Dateien |
| Eigener Konverter statt ffmpeg | keine zweite Programmdatei nötig, Standalone ist einfacher |
| Zuerst nur Windows 11 | macOS/Linux später per GitHub Actions |
| Ein Zielformat-Dropdown für alle | einfacher als je Datei |
| MP3-Quellen immer gesperrt | MP3 nicht erneut umwandeln |
| Zieldateien nie überschreiben | Datenverlust vermeiden |
| MP3 mit ID3v2.3 | beste Player-Kompatibilität |
| Dither nur bei AIFF und FLAC und nur wenn Bits wegfallen | 16-Bit-Quellen mit 44,1 kHz bleiben bitgenau |
| LAME eingebaut (mp3lame-encoder) | MP3 ohne ffmpeg |
| symphonia zum Lesen, rubato zum Umrechnen | reine Rust-Bibliotheken, kein externes Programm |
| Code in Module aufgeteilt | eine `main.rs` wäre mit den offenen Punkten zu groß geworden |
| Nr zählt alle Zeilen | gesperrte Dateien bleiben sichtbar, Nr passt zur Gesamtzahl |
| CSV mit Semikolon, ohne Anführungszeichen | deutsches Excel öffnet das direkt in Spalten |
| Semikolon im Text wird zu Komma | sonst verrutschen die Spalten |
| BOM am Dateianfang | Umlaute in Excel korrekt |
| Protokoll-Knopf erst nach einem Lauf | ein Protokoll ohne Lauf wäre leer |
| Mehrere Dateien gleichzeitig, ein Thread pro Kern | LAME nutzt nur einen Kern; 60 Tracks nach MP3 dauerten 6:30 min |
| Release mit `lto`, `codegen-units = 1`, `strip`; C-Laufzeit fest eingebaut; Build per GitHub Actions | eine `.exe` ohne Rust und ohne Visual-C++-Runtime; der Build braucht keinen Rust-PC. Kein `panic = "abort"`, weil die Umwandlung Abstürze abfängt (`catch_unwind`) |
| Standalone-Datei wie `cargo run --release` | Messung: 60 Tracks (WAV und FLAC) nach AIFF in 19 s im Release-Build. Im Debug-Build ist der eigene Konverter deutlich langsamer. Die Standalone-Datei muss also optimiert gebaut werden. |
| Ziel mit `create_new` anlegen, nur selbst angelegte Dateien löschen | nie überschreiben; bei parallelen Aufträgen mit gleichem Ziel (`a.wav` und `a.flac`) wird nie eine fremde Datei gelöscht |
| Abgebrochene Datei wird wieder „bereit“ | sie wurde gelöscht, ist also kein Fehler |
| Cover kommt aus der Bilderliste der App | eine Quelle der Wahrheit für Fenster „Cover“ und Einbetten |
| Lucide-Icons (Iconify), als SVG über `egui_extras` | schlicht, einheitlich, ISC-Lizenz; weiß gespeichert und eingefärbt |
| Feste Akzentfarbe für die Hauptaktion | Weiß auf Blau ist in hellem und dunklem Thema lesbar |
| Format als eine Spalte („FLAC → AIFF“), Ordner kurz ab Quellordner | kompakte Tabelle, voller Pfad als Tooltip und im Protokoll |
| Nur sichtbare Zeilen zeichnen | flüssig auch bei tausenden Dateien |
| Tests im Projekt (`cargo test --release`) | Bitgenauigkeit, Länge, MP3, Cover, Abbruch, vorhandene Dateien |
| Zielformate AIFF (16 Bit, 44,1 kHz), MP3 (320 kbit/s) und FLAC, keine Playerauswahl | alle geprüften Pioneer- und Denon-Player spielen AIFF und MP3 (Liste in `dj-player.md`); eine Auswahl würde am Ergebnis nichts ändern (KISS, YAGNI). FLAC ist optional, weil nicht jeder Player es spielt |
| FLAC wie AIFF fest 16 Bit, 44,1 kHz | gleiche Umrechnung für alle Ziele (KISS); FLAC ist dann verlustfrei gegenüber dem 16-Bit-Ergebnis, 16-Bit-Quellen mit 44,1 kHz bleiben sample-genau |
| FLAC mit `flacenc` (reines Rust), Kopf selbst geschrieben | kein C-Compiler nötig; blockweises Schreiben spart Speicher, der Kopf wird am Ende mit Länge und MD5 neu geschrieben. Blockgröße min = max, damit Leser den Strom nicht für variabel halten |
| Tags und Cover in FLAC als Vorbis-Kommentar und PICTURE-Block | Standard für FLAC; ID3 in FLAC wird von vielen Playern ignoriert |
| Lizenz GPL-3.0-or-later | Copyleft für ein fertiges Programm; kompatibel mit LAME (LGPL) und den übrigen Bibliotheken (MIT, Apache-2.0, MPL-2.0). LGPL wäre für Bibliotheken, AGPL für Netzdienste |
| `unsafe` nur in `mp3.rs` (`set_len`) | `mp3lame-encoder` bietet keine sichere Variante; Stellen mit `SAFETY`-Kommentar |
| Alle Texte in `texte.rs`, Sprache global (`AtomicU8`) | Fehlertexte entstehen in Hintergrund-Threads und brauchen die Sprache ohne Weiterreichen |
| Sprache in `sprache.txt` im Einstellungsordner gemerkt, sonst Systemsprache | ohne Zusatzbibliothek; ohne Schreibrecht gilt die Systemsprache |
| Das Protokoll (CSV) folgt der Sprache, auch der Kopf | eine Sprache pro Programmlauf, einfacher zu erklären |
| Prüfungen: nur rustfmt, clippy (pedantic), committed, cargo-deny | `typos` meldet bei deutschen Texten rund 200 Fehlalarme; `rustfmt.toml` und `cargo-machete` bringen jetzt nichts (YAGNI) |
| Cover höchstens 700 x 700 Pixel (Bibliothek `image`), kleinere Bilder bleiben unverändert | 3000-Pixel-Cover (4,6 MB) ließ sich im Windows Media Player nicht öffnen; 700 Pixel ist die Größe, die Bandcamp selbst einbettet |

## Module

```mermaid
flowchart LR
    main["main.rs"] --> app["app.rs"]
    main --> gui["gui.rs"]
    gui --> app
    gui --> icons["icons.rs"]
    gui --> bericht["bericht.rs"]
    gui --> texte["texte.rs"]
    app --> scan["scan.rs"]
    app --> auftraege["auftraege.rs"]
    auftraege --> konverter["konverter.rs"]
    konverter --> aiff["aiff.rs"]
    konverter --> mp3["mp3.rs"]
    konverter --> flac["flac.rs"]
    konverter --> tags["tags.rs"]
```

| Modul | Inhalt |
|---|---|
| `main.rs` | Start, Ordner aus Programm-Argumenten |
| `app.rs` | `App`, `Datei`, `Bild`, `Status`, Ordner, Auswahl, Start, Abbruch, Zusammenfassung |
| `gui.rs` | Stil, Werkzeugleiste, Quellen, Tabelle, Statusleiste, Fenster „Cover“, Drag & Drop |
| `icons.rs` | Lucide-Icons |
| `scan.rs` | Audio- und Bilddateien in einem Durchlauf finden |
| `auftraege.rs` | Aufträge parallel im Hintergrund, Nachrichten an die Oberfläche |
| `konverter.rs` | `Ziel`, Sperrregel, lesen, auf 44,1 kHz umrechnen, auf 16 Bit runden, als AIFF, MP3 oder FLAC schreiben |
| `aiff.rs` | AIFF schreiben |
| `mp3.rs` | MP3 schreiben (LAME, 320 kbit/s) |
| `flac.rs` | FLAC schreiben (flacenc), Kopf mit Länge und MD5 |
| `texte.rs` | alle Texte in Deutsch und Englisch, Sprachwahl merken |
| `tags.rs` | Tags und Cover als ID3 (AIFF, MP3) und als Vorbis-Kommentar mit Bildblock (FLAC) |
| `bericht.rs` | CSV-Protokoll |
