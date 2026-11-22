// Alle Texte des Programms in Deutsch und Englisch.
// Die gewählte Sprache gilt für das ganze Programm (auch für Fehlermeldungen aus den
// Hintergrund-Threads) und wird in einer kleinen Datei gemerkt.
// Neuer Text: Feld in `Texte` anlegen, dann in `DE` und `EN` füllen. Fehlt eine Sprache,
// meldet der Compiler es.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum Sprache {
    #[default]
    Deutsch,
    Englisch,
}

impl Sprache {
    pub const ALLE: [Sprache; 2] = [Sprache::Deutsch, Sprache::Englisch];

    // Name in der eigenen Sprache (für die Auswahl)
    pub fn name(self) -> &'static str {
        match self {
            Sprache::Deutsch => "Deutsch",
            Sprache::Englisch => "English",
        }
    }

    // Aus einem Sprachcode wie "de", "de_DE.UTF-8" oder "en-US"
    pub fn aus_code(code: &str) -> Option<Self> {
        let code = code.trim().to_lowercase();
        if code.starts_with("de") {
            Some(Sprache::Deutsch)
        } else if code.starts_with("en") {
            Some(Sprache::Englisch)
        } else {
            None
        }
    }
}

static AKTUELL: AtomicU8 = AtomicU8::new(0);

pub fn sprache() -> Sprache {
    match AKTUELL.load(Ordering::Relaxed) {
        1 => Sprache::Englisch,
        _ => Sprache::Deutsch,
    }
}

pub fn setze(s: Sprache) {
    AKTUELL.store(s as u8, Ordering::Relaxed);
}

// Die Texte der aktuellen Sprache
pub fn t() -> &'static Texte {
    match sprache() {
        Sprache::Deutsch => &DE,
        Sprache::Englisch => &EN,
    }
}

// ---------------------------------------------------------------------------
// Sprache merken: Datei „sprache.txt“ im Einstellungsordner des Nutzers
// ---------------------------------------------------------------------------

fn datei() -> Option<PathBuf> {
    crate::einstellungen::ordner().map(|o| o.join("sprache.txt"))
}

// Beim Start: gemerkte Sprache, sonst Systemsprache (LANG), sonst Deutsch
pub fn laden() {
    let gemerkt = datei()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| Sprache::aus_code(&s));
    let system = ["LC_ALL", "LANGUAGE", "LANG"]
        .iter()
        .filter_map(|v| std::env::var(v).ok())
        .find(|v| !v.is_empty())
        .map(|v| Sprache::aus_code(&v).unwrap_or(Sprache::Englisch));
    setze(gemerkt.or(system).unwrap_or_default());
}

// Sprache wechseln und merken (Fehler beim Speichern sind egal)
pub fn wechseln(s: Sprache) {
    setze(s);
    if let Some(p) = datei() {
        if let Some(ordner) = p.parent() {
            let _ = std::fs::create_dir_all(ordner);
        }
        let _ = std::fs::write(p, if s == Sprache::Deutsch { "de" } else { "en" });
    }
}

// ---------------------------------------------------------------------------
// Die Texte
// ---------------------------------------------------------------------------

pub struct Texte {
    // Werkzeugleiste
    pub ordner_hinzufuegen: &'static str,
    pub ziel: &'static str,
    pub protokoll: &'static str,
    pub protokoll_tip: &'static str,
    pub protokoll_tip_aus: &'static str,
    pub cover: &'static str,
    pub cover_fehlen: &'static str, // „Cover · 3 fehlen“
    pub cover_tip: &'static str,
    pub sprache_tip: &'static str,
    pub thema: &'static str, // Beschriftung vor der Auswahl
    pub thema_tip: &'static str,
    pub thema_system: &'static str, // „System“ = wie das Betriebssystem
    pub thema_hell: &'static str,
    pub thema_dunkel: &'static str,
    // Zielformate
    pub ziel_aiff: &'static str,
    pub ziel_mp3: &'static str,
    pub ziel_flac: &'static str,
    // Quellen und Statusleiste
    pub quellen: &'static str,
    pub ordner_entfernen: &'static str,
    pub dateien: &'static str, // „12 Dateien“
    pub ausgewaehlt: &'static str,
    pub abbrechen: &'static str,
    pub umwandeln: &'static str,
    pub keine_auswahl: &'static str,
    pub zielordner_oeffnen: &'static str,
    // Tabelle
    pub alle_tip: &'static str,
    pub spalten: [&'static str; 4], // Nr, Datei, Format, Status
    pub status_bereit: &'static str,
    pub status_gesperrt: &'static str,
    pub status_existiert: &'static str,
    pub status_wartet: &'static str,
    pub status_laeuft: &'static str,
    pub status_fertig: &'static str,
    pub status_fehler: &'static str,
    pub leer_titel: &'static str,
    pub leer_info: &'static str,
    // Fenster „Cover“
    pub ordner_ohne_cover: &'static str,
    pub alle_haben_bild: &'static str,
    pub gefundene_bilder: &'static str,
    pub bilder_hinweis: &'static str,
    // Ablegen und Protokoll
    pub nur_ordner: &'static str,
    pub hier_ablegen: &'static str,
    pub protokoll_datei: &'static str,
    pub protokoll_gespeichert: &'static str,
    pub protokoll_nicht_gespeichert: &'static str,
    pub nicht_gewaehlt: &'static str,
    pub fehler_praefix: &'static str,
    pub csv_kopf: &'static str,
    // Meldung nach einem Lauf: „Fertig: 5 umgewandelt, 1 Fehler“
    pub m_fertig: &'static str,
    pub m_abgebrochen: &'static str,
    pub m_umgewandelt: &'static str,
    pub m_fehler: &'static str,
    pub m_nicht_begonnen: &'static str,
    // Fehlertexte des Konverters
    pub zieldatei_existiert: &'static str,
    pub zieldatei_nicht_anlegbar: &'static str,
    pub quelle_nicht_lesbar: &'static str,
    pub format_nicht_erkannt: &'static str,
    pub keine_audiospur: &'static str,
    pub samplerate_unbekannt: &'static str,
    pub kanalzahl_unbekannt: &'static str,
    pub kanaele_nicht_unterstuetzt: &'static str, // „3 Kanäle werden nicht unterstützt …“
    pub codec_nicht_unterstuetzt: &'static str,
    pub lesefehler: &'static str,
    pub dekodierfehler: &'static str,
    pub kanalzahl_aendert_sich: &'static str,
    pub interner_fehler: &'static str,
    pub aiff_zu_gross: &'static str,
    pub mp3_encoder: &'static str,
    pub mp3_encoder_start: &'static str,
    pub flac_encoder: &'static str,
}

pub static DE: Texte = Texte {
    ordner_hinzufuegen: "Ordner hinzufügen",
    ziel: "Ziel",
    protokoll: "Protokoll",
    protokoll_tip: "Protokoll als CSV speichern",
    protokoll_tip_aus: "Verfügbar nach einer Umwandlung",
    cover: "Cover",
    cover_fehlen: "fehlen",
    cover_tip: "Ordner ohne Cover und gefundene Bilder",
    sprache_tip: "Sprache",
    thema: "Thema",
    thema_tip: "Thema: System (wie Windows), Hell oder Dunkel",
    thema_system: "System",
    thema_hell: "Hell",
    thema_dunkel: "Dunkel",
    ziel_aiff: "AIFF · 16 Bit · 44,1 kHz",
    ziel_mp3: "MP3 · 320 kbit/s · 44,1 kHz",
    ziel_flac: "FLAC · 16 Bit · 44,1 kHz",
    quellen: "Quellen",
    ordner_entfernen: "Ordner entfernen",
    dateien: "Dateien",
    ausgewaehlt: "ausgewählt",
    abbrechen: "Abbrechen",
    umwandeln: "Umwandeln",
    keine_auswahl: "Keine Datei ausgewählt",
    zielordner_oeffnen: "Zielordner öffnen",
    alle_tip: "Alle an- oder abwählen",
    spalten: ["Nr", "Datei", "Format", "Status"],
    status_bereit: "bereit",
    status_gesperrt: "gesperrt",
    status_existiert: "existiert bereits",
    status_wartet: "wartet",
    status_laeuft: "läuft",
    status_fertig: "fertig",
    status_fehler: "Fehler",
    leer_titel: "Ordner hinzufügen oder hierher ziehen",
    leer_info: "WAV, AIFF und FLAC werden zu AIFF, MP3 oder FLAC",
    ordner_ohne_cover: "Ordner ohne Cover",
    alle_haben_bild: "Alle Ordner mit Audiodateien haben ein Bild.",
    gefundene_bilder: "Gefundene Bilder",
    bilder_hinweis:
        "Werden nie umgewandelt. Das erste Bild eines Ordners wird als Cover eingebettet.",
    nur_ordner: "Nur Ordner können abgelegt werden",
    hier_ablegen: "Ordner hier ablegen",
    protokoll_datei: "protokoll.csv",
    protokoll_gespeichert: "Protokoll gespeichert",
    protokoll_nicht_gespeichert: "Protokoll nicht gespeichert",
    nicht_gewaehlt: "nicht gewählt",
    fehler_praefix: "Fehler",
    csv_kopf: "nr;datei;Ordner/Pfad;von;nach;status",
    m_fertig: "Fertig",
    m_abgebrochen: "Abgebrochen",
    m_umgewandelt: "umgewandelt",
    m_fehler: "Fehler",
    m_nicht_begonnen: "nicht begonnen",
    zieldatei_existiert: "Zieldatei existiert bereits",
    zieldatei_nicht_anlegbar: "Zieldatei nicht anlegbar",
    quelle_nicht_lesbar: "Quelle nicht lesbar",
    format_nicht_erkannt: "Format nicht erkannt",
    keine_audiospur: "Keine Audiospur gefunden",
    samplerate_unbekannt: "Samplerate unbekannt",
    kanalzahl_unbekannt: "Kanalzahl unbekannt",
    kanaele_nicht_unterstuetzt: "Kanäle werden nicht unterstützt (nur Mono und Stereo)",
    codec_nicht_unterstuetzt: "Codec nicht unterstützt",
    lesefehler: "Lesefehler",
    dekodierfehler: "Dekodierfehler",
    kanalzahl_aendert_sich: "Kanalzahl ändert sich mitten in der Datei",
    interner_fehler: "Interner Fehler im Konverter",
    aiff_zu_gross: "Datei größer als 4 GB (AIFF-Grenze)",
    mp3_encoder: "MP3-Encoder",
    mp3_encoder_start: "MP3-Encoder konnte nicht gestartet werden",
    flac_encoder: "FLAC-Encoder",
};

pub static EN: Texte = Texte {
    ordner_hinzufuegen: "Add folder",
    ziel: "Target",
    protokoll: "Log",
    protokoll_tip: "Save log as CSV",
    protokoll_tip_aus: "Available after a conversion",
    cover: "Cover",
    cover_fehlen: "missing",
    cover_tip: "Folders without cover and found images",
    sprache_tip: "Language",
    thema: "Theme",
    thema_tip: "Theme: System (like Windows), Light or Dark",
    thema_system: "System",
    thema_hell: "Light",
    thema_dunkel: "Dark",
    ziel_aiff: "AIFF · 16-bit · 44.1 kHz",
    ziel_mp3: "MP3 · 320 kbit/s · 44.1 kHz",
    ziel_flac: "FLAC · 16-bit · 44.1 kHz",
    quellen: "Sources",
    ordner_entfernen: "Remove folder",
    dateien: "files",
    ausgewaehlt: "selected",
    abbrechen: "Cancel",
    umwandeln: "Convert",
    keine_auswahl: "No file selected",
    zielordner_oeffnen: "Open target folder",
    alle_tip: "Select or deselect all",
    spalten: ["No", "File", "Format", "Status"],
    status_bereit: "ready",
    status_gesperrt: "locked",
    status_existiert: "already exists",
    status_wartet: "waiting",
    status_laeuft: "running",
    status_fertig: "done",
    status_fehler: "error",
    leer_titel: "Add a folder or drag it here",
    leer_info: "WAV, AIFF and FLAC are converted to AIFF, MP3 or FLAC",
    ordner_ohne_cover: "Folders without cover",
    alle_haben_bild: "All folders with audio files have an image.",
    gefundene_bilder: "Images found",
    bilder_hinweis: "Never converted. The first image of a folder is embedded as cover.",
    nur_ordner: "Only folders can be dropped",
    hier_ablegen: "Drop folder here",
    protokoll_datei: "log.csv",
    protokoll_gespeichert: "Log saved",
    protokoll_nicht_gespeichert: "Log not saved",
    nicht_gewaehlt: "not selected",
    fehler_praefix: "Error",
    csv_kopf: "no;file;folder/path;from;to;status",
    m_fertig: "Done",
    m_abgebrochen: "Cancelled",
    m_umgewandelt: "converted",
    m_fehler: "errors",
    m_nicht_begonnen: "not started",
    zieldatei_existiert: "Target file already exists",
    zieldatei_nicht_anlegbar: "Target file cannot be created",
    quelle_nicht_lesbar: "Source cannot be read",
    format_nicht_erkannt: "Format not recognized",
    keine_audiospur: "No audio track found",
    samplerate_unbekannt: "Sample rate unknown",
    kanalzahl_unbekannt: "Channel count unknown",
    kanaele_nicht_unterstuetzt: "channels are not supported (mono and stereo only)",
    codec_nicht_unterstuetzt: "Codec not supported",
    lesefehler: "Read error",
    dekodierfehler: "Decode error",
    kanalzahl_aendert_sich: "Channel count changes in the middle of the file",
    interner_fehler: "Internal error in the converter",
    aiff_zu_gross: "File larger than 4 GB (AIFF limit)",
    mp3_encoder: "MP3 encoder",
    mp3_encoder_start: "MP3 encoder could not be started",
    flac_encoder: "FLAC encoder",
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sprachcodes() {
        assert_eq!(Sprache::aus_code("de_DE.UTF-8"), Some(Sprache::Deutsch));
        assert_eq!(Sprache::aus_code("en-US\n"), Some(Sprache::Englisch));
        assert_eq!(Sprache::aus_code("fr"), None);
    }

    #[test]
    fn beide_sprachen_gleich_aufgebaut() {
        // Platzhalter-Regel: Kopfzeile des Protokolls hat in beiden Sprachen 6 Spalten
        assert_eq!(DE.csv_kopf.split(';').count(), 6);
        assert_eq!(EN.csv_kopf.split(';').count(), 6);
    }
}
