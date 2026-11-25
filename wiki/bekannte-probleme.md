# Bekannte Probleme

| Problem | Lösung |
|---|---|
| `cargo run` ist sehr langsam | `cargo run --release` verwenden |
| Erster Build dauert lange oder LAME-Fehler beim Bauen | C++ Build Tools installieren (bringen den C-Compiler mit), VS Code neu starten |
| Build-Fehler mit `convert.rs` oder `cover.rs` | Beide Dateien gibt es nicht mehr: aus `src/` löschen |
| „file not found for module“ | In `src/` müssen liegen: `main`, `app`, `gui`, `icons`, `texte`, `scan`, `auftraege`, `konverter`, `aiff`, `mp3`, `flac`, `tags`, `bericht` (je `.rs`), dazu `assets/icons/` im Projektordner |
| `cargo` nicht gefunden | Rust über rustup.rs installieren, VS Code neu starten |
| FLAC (oder AIFF/MP3) lässt sich im Windows Media Player nicht öffnen, in VLC schon | Ursache war ein sehr großes Cover (3000 x 3000 Pixel, 4,6 MB). Jetzt wird das Cover auf höchstens 700 x 700 Pixel verkleinert. Ältere Dateien neu umwandeln |
| Protokoll-Knopf ist grau | Erst nach einer Umwandlung aktiv, wieder grau nach Änderung von Ordnern oder Zielformat |
| CSV in Excel: Umlaute falsch | Datei über „Daten → Aus Text/CSV“ öffnen, Trennzeichen „;“ |
| Fehler „Format nicht erkannt“ oder „Codec nicht unterstützt“ | Datei ist kein gültiges FLAC, WAV oder AIFF. Maus über **Fehler** halten |
| Fehler „Zieldatei existiert bereits“ (englisch: „Target file already exists“) | Ziel liegt schon im Ordner, z. B. weil `Track.wav` und `Track.flac` beide `Track.aiff` ergeben. Es wird nie überschrieben |
| Sprache gewechselt, alter Fehlertext bleibt | Fehlertexte entstehen beim Umwandeln und bleiben in der alten Sprache, bis die Datei neu umgewandelt wird |
| Zielformat, Thema oder Fenstergröße werden nicht gemerkt | Sie werden beim normalen Beenden in `einstellungen.txt` (`%APPDATA%\audio-konverter`) gespeichert; nach einem Absturz oder Beenden über den Task-Manager fehlen die letzten Änderungen. Datei löschen setzt alles auf Standard |
| Sprachwahl wird nicht gemerkt | Das Programm schreibt `sprache.txt` nach `%APPDATA%\audio-konverter`; ohne Schreibrecht gilt beim nächsten Start wieder die Systemsprache |
| Ordner hat ein Cover, wird aber als „ohne Cover“ gezeigt | Versteckte Bilder (z. B. `Folder.jpg` vom Windows Media Player) und nicht lesbare Bilder werden ignoriert. Ein sichtbares Bild, z. B. `cover.jpg`, in den Ordner legen |
| Datei mit kurzen Aussetzern, aber Status „fertig“ | Einzelne defekte Blöcke der Quelle werden übersprungen statt abzubrechen; Quelle in einem anderen Programm prüfen |
| Ordner und sein Unterordner beide hinzugefügt: Entfernen des einen nimmt Dateien mit, die auch im anderen liegen | Jede Datei steht nur einmal in der Liste und gehört zum zuerst hinzugefügten Ordner. Ordner erneut hinzufügen |

Bei Fehlern: Maus über das rote **Fehler** halten. Immer mit Kopien testen.
