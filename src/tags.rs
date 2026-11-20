// Tags der Quelldatei (Titel, Künstler ...) und Cover für die Zielformate:
// - AIFF: ID3v2.3 als "ID3 "-Chunk, MP3: ID3v2.3 am Dateianfang
// - FLAC: Vorbis-Kommentar und PICTURE-Block (Metadatenblöcke)

use id3::frame::{Content, Picture, PictureType};
use id3::{Frame, Tag, TagLike, Version};
use std::path::Path;
use symphonia::core::meta::{StandardTagKey as K, Tag as QuellTag};

// Übernommene Felder: Schlüssel der Quelle -> (ID3-Frame, Vorbis-Feld)
fn feld(k: K) -> Option<(&'static str, &'static str)> {
    Some(match k {
        K::TrackTitle => ("TIT2", "TITLE"),
        K::Artist => ("TPE1", "ARTIST"),
        K::AlbumArtist => ("TPE2", "ALBUMARTIST"),
        K::Album => ("TALB", "ALBUM"),
        K::Genre => ("TCON", "GENRE"),
        K::Date => ("TYER", "DATE"),
        K::TrackNumber => ("TRCK", "TRACKNUMBER"),
        K::DiscNumber => ("TPOS", "DISCNUMBER"),
        K::Composer => ("TCOM", "COMPOSER"),
        K::Bpm => ("TBPM", "BPM"),
        K::Label => ("TPUB", "ORGANIZATION"),
        _ => return None,
    })
}

// Bild laden: (MIME-Typ, Bytes der Bilddatei unverändert)
fn bild_laden(pfad: &Path) -> Option<(&'static str, Vec<u8>)> {
    let daten = std::fs::read(pfad).ok()?;
    let mime = match pfad
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase)
        .as_deref()
    {
        Some("png") => "image/png",
        Some("bmp") => "image/bmp",
        _ => "image/jpeg",
    };
    Some((mime, daten))
}

// Fertigen ID3-Tag als Bytes erzeugen. Leer, wenn es nichts zu schreiben gibt.
pub fn erzeuge(quelle: &[QuellTag], bild: Option<&Path>) -> Vec<u8> {
    let mut tag = Tag::new();

    for t in quelle {
        let Some((id, _)) = t.std_key.and_then(feld) else {
            continue;
        };
        let mut text = t.value.to_string();
        if id == "TYER" {
            text = text.chars().take(4).collect(); // ID3v2.3 kennt nur das Jahr
        }
        if !text.is_empty() {
            tag.set_text(id, text);
        }
    }

    if let Some((mime, daten)) = bild.and_then(bild_laden) {
        tag.add_frame(Frame::with_content(
            "APIC",
            Content::Picture(Picture {
                mime_type: mime.to_string(),
                picture_type: PictureType::CoverFront,
                description: String::new(),
                data: daten,
            }),
        ));
    }

    let mut aus = Vec::new();
    if tag.frames().next().is_some() {
        let _ = tag.write_to(&mut aus, Version::Id3v23);
    }
    aus
}

// FLAC-Metadatenblöcke (Typ, Inhalt): Vorbis-Kommentar (4) und Bild (6)
pub fn flac_bloecke(quelle: &[QuellTag], bild: Option<&Path>) -> Vec<(u8, Vec<u8>)> {
    let mut bloecke = Vec::new();

    let felder: Vec<String> = quelle
        .iter()
        .filter_map(|t| {
            let (_, name) = t.std_key.and_then(feld)?;
            let wert = t.value.to_string();
            (!wert.is_empty()).then(|| format!("{name}={wert}"))
        })
        .collect();
    if !felder.is_empty() {
        let hersteller = b"audio-konverter";
        let mut d = Vec::new();
        d.extend_from_slice(&(hersteller.len() as u32).to_le_bytes());
        d.extend_from_slice(hersteller);
        d.extend_from_slice(&(felder.len() as u32).to_le_bytes());
        for f in &felder {
            d.extend_from_slice(&(f.len() as u32).to_le_bytes());
            d.extend_from_slice(f.as_bytes());
        }
        bloecke.push((4, d));
    }

    // Ein Metadatenblock darf höchstens 16 MB groß sein
    if let Some((mime, daten)) = bild
        .and_then(bild_laden)
        .filter(|(_, d)| d.len() < 16_000_000)
    {
        let mut d = Vec::new();
        d.extend_from_slice(&3u32.to_be_bytes()); // Typ 3 = Cover (vorne)
        d.extend_from_slice(&(mime.len() as u32).to_be_bytes());
        d.extend_from_slice(mime.as_bytes());
        d.extend_from_slice(&0u32.to_be_bytes()); // keine Beschreibung
        d.extend_from_slice(&[0; 16]); // Breite, Höhe, Farbtiefe, Farbanzahl: unbekannt
        d.extend_from_slice(&(daten.len() as u32).to_be_bytes());
        d.extend_from_slice(&daten);
        bloecke.push((6, d));
    }
    bloecke
}

#[cfg(test)]
mod tests {
    use super::*;
    use symphonia::core::meta::Value;

    #[test]
    fn vorbis_kommentar_und_id3_aus_denselben_tags() {
        let tags = [
            QuellTag::new(Some(K::TrackTitle), "TITLE", Value::from("Intro")),
            QuellTag::new(Some(K::Artist), "ARTIST", Value::from("DJ Test")),
            QuellTag::new(None, "EGAL", Value::from("wird ignoriert")),
        ];
        let bloecke = flac_bloecke(&tags, None);
        assert_eq!(bloecke.len(), 1);
        let (typ, d) = &bloecke[0];
        assert_eq!(*typ, 4);
        let text = String::from_utf8_lossy(d);
        assert!(text.contains("TITLE=Intro") && text.contains("ARTIST=DJ Test"));
        assert!(!text.contains("EGAL"));
        let id3 = erzeuge(&tags, None);
        assert_eq!(&id3[0..3], b"ID3");
        assert_eq!(flac_bloecke(&[], None), []);
    }
}
