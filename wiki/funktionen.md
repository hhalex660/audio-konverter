# Funktionen

- Ordner per Knopf (mehrere auf einmal), per Drag & Drop oder als Programm-Argument hinzufügen; Unterordner werden durchsucht; versteckte Dateien und Ordner (z. B. `._Titel.wav` vom Mac, Papierkorb, versteckte Media-Player-Bilder) werden übersprungen
- Quellen: FLAC, WAV, AIFF (MP3 wird gelistet, aber nie umgewandelt)
- Ziel per Dropdown: AIFF 16 Bit 44,1 kHz, MP3 320 kbit/s 44,1 kHz oder FLAC 16 Bit 44,1 kHz
- Tabelle: pro Ordner eine Kopfzeile (Pfad, Dateianzahl); darunter Häkchen, Nr, Datei, Format (z. B. FLAC → AIFF), Status mit Icon
- Kopf-Häkchen wählt alle wählbaren Dateien an oder ab
- Flüssig auch bei tausenden Dateien
- Mehrere Dateien gleichzeitig (ein Thread pro CPU-Kern), Abbrechen jederzeit
- Ziel liegt im Ordner der Quelle; Originale und vorhandene Dateien werden nie verändert
- Nach dem Lauf: Zusammenfassung in der Statusleiste, Protokoll als CSV (deutsch: `nr;datei;Ordner/Pfad;von;nach;status`, englisch: `no;file;folder/path;from;to;status`)
- Fenster „Cover“: Ordner ohne Bild und alle gefundenen Bilder; das erste lesbare Bild eines Ordners wird in AIFF, MP3 und FLAC eingebettet (kaputte Bilder werden übersprungen) (größer als 700 x 700 Pixel: auf 700 verkleinert, sonst unverändert)
- Tags (Titel, Künstler, Album, Album-Künstler, Genre, Jahr, Track, Disc, Komponist, BPM, Label) werden übernommen: bei AIFF und MP3 als ID3, bei FLAC als Vorbis-Kommentar
- Sprache Deutsch oder Englisch: Auswahl oben rechts, wird in `sprache.txt` im Einstellungsordner gemerkt (`%APPDATA%\audio-konverter`); beim ersten Start gilt die Systemsprache (`LANG`), sonst Deutsch
- Thema: Auswahl oben rechts (System = wie das Betriebssystem, Hell, Dunkel), Lucide-Icons
- Gemerkt werden Zielformat, Thema, Fenstergröße und der Startordner des Auswahldialogs: `einstellungen.txt` im Einstellungsordner, geschrieben beim Beenden
- Nach dem Lauf: Knopf „Zielordner öffnen“ (öffnet den Ordner der ersten fertigen Datei)
- Sanfte Übergänge: Hover-Farben, Fortschrittsbalken und Auf- und Zuklappen blenden weich

## Ablauf

```mermaid
flowchart LR
    A["Ordner wählen<br/>gui.rs"] --> B["Dateien und Bilder suchen<br/>scan.rs"] --> C["Sperren und Status<br/>app.rs"] --> D["Umwandeln<br/>auftraege.rs"] --> E["parallel pro Datei<br/>konverter.rs"] --> F["Zusammenfassung<br/>app.rs"] --> G["CSV-Protokoll<br/>bericht.rs"]
```

## Sperren

```mermaid
flowchart TD
    A["Datei"] --> B{"MP3?"}
    B -->|ja| X["gesperrt"]
    B -->|nein| C{"AIFF und Ziel AIFF?<br/>FLAC und Ziel FLAC?"}
    C -->|ja| X
    C -->|nein| D{"Zieldatei existiert?"}
    D -->|ja| Y["existiert bereits"]
    D -->|nein| Z["bereit"]
```

## Umwandlung

```mermaid
flowchart LR
    Q["Quelle<br/>FLAC, WAV, AIFF"] --> L["lesen<br/>symphonia"] --> Z{"Ziel"}
    Z --> R["auf 44,1 kHz<br/>rubato"] --> D["auf 16 Bit<br/>Dither nur bei AIFF und FLAC"]
    D --> A["aiff.rs"]
    D --> M["mp3.rs, LAME 320 kbit/s"]
    D --> F["flac.rs"]
    T["tags.rs: Tags + Cover"] --> A
    T --> M
    T --> F
```
