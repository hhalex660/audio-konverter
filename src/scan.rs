// Ordner durchsuchen: findet Audio- und Bilddateien in einem Ordner samt Unterordnern.

use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

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

// Versteckt: Name beginnt mit „.“ (z. B. „._Titel.wav“ vom Mac) oder unter Windows das
// Attribut „versteckt“ (z. B. AlbumArtSmall.jpg vom Media Player, Papierkorb)
fn versteckt(e: &DirEntry) -> bool {
    e.file_name().to_string_lossy().starts_with('.') || windows_versteckt(e)
}

#[cfg(windows)]
fn windows_versteckt(e: &DirEntry) -> bool {
    use std::os::windows::fs::MetadataExt;
    const VERSTECKT: u32 = 0x2; // FILE_ATTRIBUTE_HIDDEN
    e.metadata()
        .is_ok_and(|m| m.file_attributes() & VERSTECKT != 0)
}

#[cfg(not(windows))]
fn windows_versteckt(_: &DirEntry) -> bool {
    false
}

// Ordner und alle Unterordner in einem Durchlauf durchsuchen. Versteckte Dateien und
// Ordner werden übersprungen; der gewählte Ordner selbst zählt immer.
pub fn durchsuche(wurzel: &Path) -> Fund {
    let mut fund = Fund::default();
    let eintraege = WalkDir::new(wurzel)
        .into_iter()
        .filter_entry(|e| e.depth() == 0 || !versteckt(e))
        .filter_map(Result::ok);
    for eintrag in eintraege {
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
    use crate::konverter::tests::ordner;

    #[test]
    fn versteckte_dateien_und_ordner_werden_uebersprungen() {
        let o = ordner(".versteckter-quellordner"); // gewählter Ordner zählt trotzdem
        std::fs::create_dir_all(o.join(".Trashes")).unwrap();
        for name in [
            "a.wav",
            "._a.wav",
            "cover.jpg",
            "._cover.jpg",
            ".Trashes/b.wav",
        ] {
            std::fs::write(o.join(name), b"x").unwrap();
        }
        let fund = durchsuche(&o);
        assert_eq!(fund.audio, vec![o.join("a.wav")]);
        assert_eq!(fund.bilder, vec![o.join("cover.jpg")]);
    }
}
