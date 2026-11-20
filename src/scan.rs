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
    pfad.extension()
        .map_or_default(|e| e.to_string_lossy().to_lowercase())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endung_ist_kleingeschrieben_oder_leer() {
        assert_eq!(endung_von(Path::new("titel.FLAC")), "flac");
        assert_eq!(endung_von(Path::new("ohne-endung")), "");
    }

    #[test]
    fn findet_audio_und_bilder_rekursiv_aber_ignoriert_andere_dateien() {
        let temp = tempfile::tempdir().unwrap();
        let unterordner = temp.path().join("unterordner");
        std::fs::create_dir(&unterordner).unwrap();
        let audio = unterordner.join("titel.WAV");
        let bild = temp.path().join("cover.JPG");
        std::fs::write(&audio, b"audio").unwrap();
        std::fs::write(&bild, b"bild").unwrap();
        std::fs::write(temp.path().join("notiz.txt"), b"text").unwrap();

        let fund = durchsuche(temp.path());
        assert_eq!(fund.audio, vec![audio]);
        assert_eq!(fund.bilder, vec![bild]);
    }
}
