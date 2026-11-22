// Ordner durchsuchen: findet Audio- und Bilddateien in einem Ordner samt Unterordnern.

use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const AUDIO: [&str; 5] = ["flac", "wav", "aiff", "aif", "mp3"];
const BILD: [&str; 4] = ["jpg", "jpeg", "png", "bmp"];

// Ergebnis einer Suche
#[derive(Default)]
pub struct Fund {
    pub audio: Vec<PathBuf>,
    pub bilder: Vec<PathBuf>,
}

// Dateiendung in Kleinbuchstaben, z. B. "flac"
pub fn endung_von(pfad: &Path) -> String {
    match pfad.extension() {
        Some(e) => e.to_string_lossy().to_lowercase(),
        None => String::new(),
    }
}

// Ordner und alle Unterordner in einem Durchlauf durchsuchen
pub fn durchsuche(wurzel: &Path) -> Fund {
    let mut fund = Fund::default();
    for eintrag in WalkDir::new(wurzel).into_iter().filter_map(Result::ok) {
        if !eintrag.file_type().is_file() {
            continue;
        }
        let pfad = eintrag.into_path();
        let endung = endung_von(&pfad);
        if AUDIO.contains(&endung.as_str()) {
            fund.audio.push(pfad);
        } else if BILD.contains(&endung.as_str()) {
            fund.bilder.push(pfad);
        }
    }
    fund
}
