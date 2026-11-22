# Audit und Design-Review (v0.2.0)

## Audit

| Befund | Schwere | Stand |
|---|---|---|
| MP3 nutzt nur einen Kern (60 Tracks 6:30 min) | hoch | behoben: parallel, ein Thread pro Kern |
| Bei Fehler wurde die Zieldatei gelöscht, auch wenn sie nicht vom Programm stammte (bei parallelen Aufträgen mit gleichem Ziel real) | hoch | behoben: Datei mit `create_new` anlegen, nur eigene löschen; Test |
| Abbruch erschien als roter Fehler | mittel | behoben: Datei wird wieder „bereit“, Zusammenfassung nennt den Abbruch |
| Ordner zweimal durchsucht, Duplikatprüfung quadratisch, Oberfläche zeichnet alle Zeilen | mittel | behoben: ein Durchlauf, HashSet, nur sichtbare Zeilen |
| Cover-Suche doppelt (eigene Ordnersuche in `cover.rs`) | mittel | behoben: Cover aus der Bilderliste, `cover.rs` entfernt |
| `bericht.rs` hing von `gui.rs` ab, `convert.rs` und `konverter.rs` verwechselbar | niedrig | behoben: `Status::text()`, `convert.rs` → `auftraege.rs`, `Ziel` in `konverter.rs` |
| AIFF über 4 GB wäre still kaputt gegangen | niedrig | behoben: klare Fehlermeldung |
| Keine Tests | mittel | behoben: 11 Tests (`cargo test --release`) |
| `.gitignore` fehlte, `Cargo.lock` nicht im Projekt, veraltete Excalidraw-Diagramme | niedrig | behoben |
| Clippy-Warnungen, uneinheitliches Format | niedrig | behoben (`cargo clippy`, `cargo fmt`) |

## Design-Review (nach Nutzen ÷ Aufwand)

| Tier | Befund | Stand |
|---|---|---|
| S | Hauptaktion sah aus wie jeder andere Knopf und lag zwischen vier gleichen Knöpfen | behoben: blauer Knopf „Umwandeln“ rechts unten, beim Lauf „Abbrechen“ an derselben Stelle |
| S | Pfadspalte wiederholte in jeder Zeile den langen absoluten Pfad | behoben: kurzer Pfad ab Quellordner, voller Pfad als Tooltip |
| S | Zeilenfarben fest blau/grün, im hellen Thema unpassend | behoben: dezente Themenfarbe |
| S | Status nur als farbiges Wort | behoben: Icon + Wort, Spinner bei „läuft“ |
| S | „→“ wurde als Kästchen angezeigt (Schrift ohne Zeichen) | behoben: Pfeil-Icon |
| S | Hauptknopf im hellen Thema blass, weiße Schrift kaum lesbar | behoben: feste Akzentfarbe |
| A | Bilderliste schob die Tabelle nach unten, zweites Fenster nur für fehlende Cover | behoben: ein Fenster „Cover“ mit beiden Listen, Knopf zeigt „3 fehlen“ |
| A | Keine Mehrfachauswahl | behoben: Kopf-Häkchen für alle |
| A | Ordner nur einzeln über Dialog | behoben: mehrere im Dialog, Drag & Drop, Programm-Argumente |
| A | Leere Ansicht ohne Hinweis | behoben: Icon und Hinweis in der Mitte |
| A | Kein Ergebnis nach dem Lauf, „0 ausgewählt“ springt während des Laufs | behoben: Zusammenfassung, ruhige Statuszeile |
| A | Viel Hilfetext in der Seitenleiste, „x“ als Buchstabe | behoben: Tooltips, Icon-Knöpfe |
| B | Spalten sortieren und in der Breite ziehen, Suche/Filter | offen |
| B | Einstellungen merken (Zielformat, Fenstergröße, Thema, Startordner des Dialogs) | behoben: `einstellungen.rs`, beim Beenden gespeichert |
| B | Zielordner nach dem Lauf öffnen | behoben: Knopf „Zielordner öffnen“ |
| B | Programm-Icon für die .exe | behoben: `assets/icon.ico` per `build.rs` und `winres` eingebaut |
| B | Ordner im Hintergrund durchsuchen (sehr große, langsame Laufwerke) | offen |
| C | Themenwahl in der Oberfläche (System, Hell, Dunkel), Animationen | behoben: Auswahl oben rechts, weiche Übergänge |
| C | Ordner in der Liste kaum unterscheidbar (Streifen in Hell unsichtbar) | behoben: eine Kopfzeile pro Ordner mit Dateianzahl, Ordner-Spalte entfällt |
