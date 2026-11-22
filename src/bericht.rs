// Protokoll der Dateien als CSV: nr;datei;Ordner/Pfad;von;nach;status

use crate::app::{Datei, Status};
use crate::konverter::Ziel;
use crate::texte::t;
use std::path::Path;

const TRENNER: &str = ";";

// Ein Feld aufbereiten: Zeilenumbrüche und Semikolons entschärfen, ohne Anführungszeichen
fn feld(text: &str) -> String {
    text.replace(['\r', '\n'], " ").replace(';', ",")
}

// Statuswert für das Protokoll
fn status_fuer_protokoll(d: &Datei) -> String {
    match &d.status {
        Status::Bereit if !d.gewaehlt => t().nicht_gewaehlt.to_string(),
        Status::Fehler(e) => format!("{}: {e}", t().fehler_praefix),
        s => s.text().to_string(),
    }
}

// CSV-Text erzeugen, eine Zeile pro Datei in der Reihenfolge der Tabelle
pub fn erstelle(dateien: &[Datei], ziel: Ziel) -> String {
    // BOM am Anfang, damit Excel Umlaute richtig anzeigt
    let mut csv = String::from("\u{FEFF}");
    csv.push_str(t().csv_kopf);
    csv.push_str("\r\n");
    for (i, d) in dateien.iter().enumerate() {
        let felder = [
            (i + 1).to_string(),
            d.name.clone(),
            d.ordner.display().to_string(),
            d.von(),
            d.nach(ziel).to_string(),
            status_fuer_protokoll(d),
        ];
        let felder: Vec<String> = felder.iter().map(|f| feld(f)).collect();
        csv.push_str(&felder.join(TRENNER));
        csv.push_str("\r\n");
    }
    csv
}

pub fn speichern(dateien: &[Datei], ziel: Ziel, pfad: &Path) -> Result<(), String> {
    std::fs::write(pfad, erstelle(dateien, ziel)).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn csv_aufbau() {
        let d = Datei {
            pfad: PathBuf::from("/m/a;b.flac"),
            wurzel: PathBuf::from("/m"),
            ordner: PathBuf::from("/m"),
            ordner_anzeige: "m".into(),
            name: "a;b.flac".into(),
            endung: "flac".into(),
            gewaehlt: true,
            status: Status::Fehler("kaputt\nZeile 2".into()),
        };
        let csv = erstelle(&[d], Ziel::Aiff);
        let zeilen: Vec<&str> = csv.trim_start_matches('\u{FEFF}').split("\r\n").collect();
        assert_eq!(zeilen[0], "nr;datei;Ordner/Pfad;von;nach;status");
        assert_eq!(zeilen[1], "1;a,b.flac;/m;FLAC;AIFF;Fehler: kaputt Zeile 2");
    }
}
