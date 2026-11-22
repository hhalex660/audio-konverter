// Einstellungen merken: kleine Textdatei „einstellungen.txt“ (Zeilen „schluessel=wert“)
// im Einstellungsordner des Nutzers. Kann sie nicht gelesen oder geschrieben werden,
// gelten die Standardwerte; das Programm läuft trotzdem. Die Sprache hat eine eigene
// Datei (siehe texte.rs).

use crate::konverter::Ziel;
use crate::texte::t;
use std::path::PathBuf;

// Thema: wie das Betriebssystem (Standard), hell oder dunkel
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum Thema {
    #[default]
    System,
    Hell,
    Dunkel,
}

impl Thema {
    pub const ALLE: [Thema; 3] = [Thema::System, Thema::Hell, Thema::Dunkel];

    // Name für die Auswahl (in der gewählten Sprache)
    pub fn name(self) -> &'static str {
        match self {
            Thema::System => t().thema_system,
            Thema::Hell => t().thema_hell,
            Thema::Dunkel => t().thema_dunkel,
        }
    }

    // Kurzer Code für die Datei
    fn code(self) -> &'static str {
        match self {
            Thema::System => "system",
            Thema::Hell => "hell",
            Thema::Dunkel => "dunkel",
        }
    }

    fn aus_code(code: &str) -> Option<Self> {
        Self::ALLE.into_iter().find(|t| t.code() == code)
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Einstellungen {
    pub ziel: Ziel,
    pub thema: Thema,
    pub fenster: Option<(f32, f32)>,     // Fenstergröße beim Beenden
    pub letzter_ordner: Option<PathBuf>, // Startordner des Auswahldialogs
}

impl Einstellungen {
    fn aus_text(text: &str) -> Self {
        let mut e = Self::default();
        for zeile in text.lines() {
            let Some((schluessel, wert)) = zeile.split_once('=') else {
                continue;
            };
            let wert = wert.trim();
            match schluessel.trim() {
                "ziel" => e.ziel = Ziel::aus_code(wert).unwrap_or_default(),
                "thema" => e.thema = Thema::aus_code(wert).unwrap_or_default(),
                "fenster" => {
                    e.fenster = wert
                        .split_once('x')
                        .and_then(|(b, h)| Some((b.parse::<f32>().ok()?, h.parse::<f32>().ok()?)));
                }
                "ordner" if !wert.is_empty() => e.letzter_ordner = Some(PathBuf::from(wert)),
                _ => {}
            }
        }
        e
    }

    fn als_text(&self) -> String {
        let mut zeilen = vec![
            format!("ziel={}", self.ziel.endung()),
            format!("thema={}", self.thema.code()),
        ];
        if let Some((b, h)) = self.fenster {
            zeilen.push(format!("fenster={}x{}", b.round(), h.round()));
        }
        if let Some(o) = &self.letzter_ordner {
            zeilen.push(format!("ordner={}", o.display()));
        }
        zeilen.join("\n") + "\n"
    }

    pub fn laden() -> Self {
        match datei().and_then(|p| std::fs::read_to_string(p).ok()) {
            Some(text) => Self::aus_text(&text),
            None => Self::default(),
        }
    }

    // Fehler beim Speichern sind egal
    pub fn speichern(&self) {
        if let Some(p) = datei() {
            if let Some(ordner) = p.parent() {
                let _ = std::fs::create_dir_all(ordner);
            }
            let _ = std::fs::write(p, self.als_text());
        }
    }
}

// Einstellungsordner des Nutzers: %APPDATA%\audio-konverter (Linux/macOS: ~/.config/...)
pub fn ordner() -> Option<PathBuf> {
    let basis = std::env::var_os("APPDATA")
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME"))
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(basis.join("audio-konverter"))
}

fn datei() -> Option<PathBuf> {
    ordner().map(|o| o.join("einstellungen.txt"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schreiben_und_lesen_ergibt_dasselbe() {
        let e = Einstellungen {
            ziel: Ziel::Flac,
            thema: Thema::Dunkel,
            fenster: Some((1000.0, 600.0)),
            letzter_ordner: Some(PathBuf::from("/Musik/Mixe")),
        };
        assert_eq!(Einstellungen::aus_text(&e.als_text()), e);
    }

    #[test]
    fn kaputte_oder_fehlende_werte_ergeben_standard() {
        let e = Einstellungen::aus_text("ziel=ogg\nthema=bunt\nfenster=abc\nquatsch\n\n");
        assert_eq!(e, Einstellungen::default());
        assert_eq!(e.thema, Thema::System); // Standard: wie das Betriebssystem
    }
}
