// Zustand und Ablaufsteuerung: Ordner, Dateiliste, Start, Abbruch.

use crate::auftraege::{self, Auftrag, Msg};
use crate::einstellungen::{Einstellungen, Thema};
use crate::konverter::{ist_gesperrt, Ziel};
use crate::scan;
use crate::texte::t;
use eframe::egui;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;

// Eine Zeile der Tabelle: Kopfzeile eines Ordners oder eine Datei (Index in `dateien`)
pub enum Zeile {
    Kopf {
        anzeige: String, // kurzer Pfad ab dem Quellordner
        pfad: PathBuf,   // voller Pfad (Tooltip)
        anzahl: usize,   // Dateien in diesem Ordner
    },
    Datei(usize),
}

// Status einer Datei in der Tabelle
#[derive(Clone, PartialEq)]
pub enum Status {
    Bereit,
    Gesperrt,
    Existiert,
    Wartet,
    Laeuft,
    Fertig,
    Fehler(String),
}

impl Status {
    // Anzeigetext, auch für das Protokoll
    pub fn text(&self) -> &'static str {
        let t = t();
        match self {
            Status::Bereit => t.status_bereit,
            Status::Gesperrt => t.status_gesperrt,
            Status::Existiert => t.status_existiert,
            Status::Wartet => t.status_wartet,
            Status::Laeuft => t.status_laeuft,
            Status::Fertig => t.status_fertig,
            Status::Fehler(_) => t.status_fehler,
        }
    }
}

// Eine gefundene Audiodatei
pub struct Datei {
    pub pfad: PathBuf,          // vollständiger Pfad der Quelldatei
    pub wurzel: PathBuf,        // vom Nutzer gewählter Quellordner
    pub ordner: PathBuf,        // Ordner, in dem die Datei liegt
    pub ordner_anzeige: String, // kurzer Pfad ab dem Quellordner, z. B. „Album\Bonus“
    pub name: String,
    pub endung: String, // kleingeschrieben, z. B. "flac"
    pub gewaehlt: bool, // Häkchen
    pub status: Status,
}

impl Datei {
    // Quellformat für Tabelle und Protokoll, z. B. "FLAC"
    pub fn von(&self) -> String {
        self.endung.to_uppercase()
    }
    // Zielformat ("-" bei MP3-Quellen, die nie umgewandelt werden)
    pub fn nach(&self, ziel: Ziel) -> &'static str {
        if self.endung == "mp3" {
            "-"
        } else {
            ziel.kurz()
        }
    }
    pub fn waehlbar(&self) -> bool {
        self.status == Status::Bereit
    }
}

// Eine gefundene Bilddatei (mögliches Cover). Wird nur gelistet, nie konvertiert.
pub struct Bild {
    pub pfad: PathBuf,
    pub wurzel: PathBuf,
    pub ordner: PathBuf,
    pub ordner_anzeige: String,
    pub name: String,
}

#[derive(Default)]
pub struct App {
    pub ordner: Vec<PathBuf>,               // gewählte Quellordner
    pub dateien: Vec<Datei>,                // alle gefundenen Audiodateien, nach Ordner sortiert
    pub zeilen: Vec<Zeile>, // Tabellenzeilen: je Ordner eine Kopfzeile, dann seine Dateien
    pub bilder: Vec<Bild>,  // gefundene Bilder, nach Ordner sortiert
    pub ohne_cover: Vec<(PathBuf, String)>, // Ordner mit Audio, aber ohne Bild (Pfad, Anzeige)
    pub ziel: Ziel,
    pub laeuft: bool,
    pub gesamt: usize,                   // Aufträge im aktuellen Lauf
    pub erledigt: usize,                 // davon abgeschlossen
    pub fehler: usize,                   // davon mit Fehler
    pub protokoll_bereit: bool,          // Protokoll gibt es erst nach einem Lauf
    pub meldung: Option<String>,         // Hinweis in der Statusleiste
    pub zeige_cover: bool,               // Fenster „Cover“ offen
    pub thema: Thema,                    // Thema: System, hell, dunkel
    pub fenster: Option<(f32, f32)>,     // zuletzt gesehene Fenstergröße (wird gemerkt)
    pub letzter_ordner: Option<PathBuf>, // Startordner des Auswahldialogs
    pub zielordner: Option<PathBuf>,     // Ordner der ersten fertigen Datei des letzten Laufs
    rx: Option<Receiver<Msg>>,
    abbruch: Arc<AtomicBool>,
}

// Kurzer Ordnerpfad ab dem Quellordner (inklusive dessen Namen)
fn anzeige(ordner: &Path, wurzel: &Path) -> String {
    wurzel
        .parent()
        .and_then(|p| ordner.strip_prefix(p).ok())
        .unwrap_or(ordner)
        .display()
        .to_string()
}

fn dateiname(p: &Path) -> String {
    match p.file_name() {
        Some(n) => n.to_string_lossy().into_owned(),
        None => String::new(),
    }
}

// Ordner im Datei-Manager öffnen (Windows: Explorer); Fehler sind egal
pub fn ordner_oeffnen(pfad: &Path) {
    let programm = if cfg!(windows) {
        "explorer"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    let _ = std::process::Command::new(programm).arg(pfad).spawn();
}

impl App {
    // Mit den gemerkten Einstellungen starten
    pub fn neu(e: Einstellungen) -> Self {
        Self {
            ziel: e.ziel,
            thema: e.thema,
            fenster: e.fenster,
            letzter_ordner: e.letzter_ordner,
            ..Self::default()
        }
    }

    // Das, was beim Beenden gemerkt wird
    pub fn einstellungen(&self) -> Einstellungen {
        Einstellungen {
            ziel: self.ziel,
            thema: self.thema,
            fenster: self.fenster,
            letzter_ordner: self.letzter_ordner.clone(),
        }
    }

    // Ordner samt Unterordnern durchsuchen und Dateien und Bilder aufnehmen
    pub fn ordner_hinzufuegen(&mut self, wurzel: PathBuf) {
        if self.laeuft || self.ordner.contains(&wurzel) {
            return;
        }
        let fund = scan::durchsuche(&wurzel);
        let bekannt: HashSet<PathBuf> = self.dateien.iter().map(|d| d.pfad.clone()).collect();
        for pfad in fund.audio.into_iter().filter(|p| !bekannt.contains(p)) {
            let ordner = pfad.parent().unwrap_or(&wurzel).to_path_buf();
            self.dateien.push(Datei {
                ordner_anzeige: anzeige(&ordner, &wurzel),
                name: dateiname(&pfad),
                endung: scan::endung_von(&pfad),
                wurzel: wurzel.clone(),
                ordner,
                pfad,
                gewaehlt: false,
                status: Status::Wartet, // wird in aktualisiere_status() bestimmt
            });
        }
        let bekannt: HashSet<PathBuf> = self.bilder.iter().map(|b| b.pfad.clone()).collect();
        for pfad in fund.bilder.into_iter().filter(|p| !bekannt.contains(p)) {
            let ordner = pfad.parent().unwrap_or(&wurzel).to_path_buf();
            self.bilder.push(Bild {
                ordner_anzeige: anzeige(&ordner, &wurzel),
                name: dateiname(&pfad),
                wurzel: wurzel.clone(),
                ordner,
                pfad,
            });
        }
        self.letzter_ordner = wurzel.parent().map(Path::to_path_buf);
        self.ordner.push(wurzel);
        self.neu_ordnen();
    }

    // Quellordner samt seinen Dateien und Bildern entfernen
    pub fn ordner_entfernen(&mut self, index: usize) {
        if self.laeuft {
            return;
        }
        let wurzel = self.ordner.remove(index);
        self.dateien.retain(|d| d.wurzel != wurzel);
        self.bilder.retain(|b| b.wurzel != wurzel);
        self.neu_ordnen();
    }

    pub fn ziel_setzen(&mut self, ziel: Ziel) {
        if self.ziel != ziel && !self.laeuft {
            self.ziel = ziel;
            self.aktualisiere_status();
        }
    }

    // Sortieren, Ordner-Kopfzeilen bilden, Ordner ohne Cover bestimmen
    fn neu_ordnen(&mut self) {
        self.dateien
            .sort_by(|a, b| (&a.ordner, &a.name).cmp(&(&b.ordner, &b.name)));
        self.bilder
            .sort_by(|a, b| (&a.ordner, &a.name).cmp(&(&b.ordner, &b.name)));
        self.zeilen.clear();
        let mut i = 0;
        while i < self.dateien.len() {
            let ordner = &self.dateien[i].ordner;
            let anzahl = self.dateien[i..]
                .iter()
                .take_while(|d| &d.ordner == ordner)
                .count();
            self.zeilen.push(Zeile::Kopf {
                anzeige: self.dateien[i].ordner_anzeige.clone(),
                pfad: ordner.clone(),
                anzahl,
            });
            self.zeilen.extend((i..i + anzahl).map(Zeile::Datei));
            i += anzahl;
        }
        let mit_bild: HashSet<&PathBuf> = self.bilder.iter().map(|b| &b.ordner).collect();
        self.ohne_cover = self
            .dateien
            .iter()
            .filter(|d| !mit_bild.contains(&d.ordner))
            .map(|d| (d.ordner.clone(), d.ordner_anzeige.clone()))
            .collect();
        self.ohne_cover.dedup(); // Dateien sind nach Ordner sortiert
        self.meldung = None;
        self.aktualisiere_status();
    }

    // Status bestimmen: gesperrt / existiert bereits / bereit
    fn aktualisiere_status(&mut self) {
        self.protokoll_bereit = false; // Protokoll passt nicht mehr zur Liste
        let ziel = self.ziel;
        for d in &mut self.dateien {
            let neu = if ist_gesperrt(&d.endung, ziel) {
                Status::Gesperrt
            } else if d.pfad.with_extension(ziel.endung()).exists() {
                Status::Existiert
            } else {
                Status::Bereit
            };
            // Häkchen nur zurücksetzen, wenn sich der Status ändert
            if neu != d.status {
                d.gewaehlt = neu == Status::Bereit;
                d.status = neu;
            }
        }
    }

    pub fn anzahl_gewaehlt(&self) -> usize {
        self.dateien
            .iter()
            .filter(|d| d.gewaehlt && d.waehlbar())
            .count()
    }

    pub fn anzahl_waehlbar(&self) -> usize {
        self.dateien.iter().filter(|d| d.waehlbar()).count()
    }

    // Alle wählbaren Dateien an- oder abwählen
    pub fn alle_waehlen(&mut self, an: bool) {
        for d in self.dateien.iter_mut().filter(|d| d.waehlbar()) {
            d.gewaehlt = an;
        }
    }

    pub fn kann_starten(&self) -> bool {
        !self.laeuft && self.anzahl_gewaehlt() > 0
    }

    // Erstes Bild (alphabetisch) im Ordner, wird als Cover eingebettet
    fn cover_fuer(&self, ordner: &Path) -> Option<PathBuf> {
        self.bilder
            .iter()
            .find(|b| b.ordner == ordner)
            .map(|b| b.pfad.clone())
    }

    // Umwandlung der angehakten Dateien starten
    pub fn starten(&mut self, ctx: &egui::Context) {
        if !self.kann_starten() {
            return;
        }
        let ziel = self.ziel;
        let mut auftraege = Vec::new();
        for (index, d) in self.dateien.iter().enumerate() {
            if d.gewaehlt && d.waehlbar() {
                auftraege.push(Auftrag {
                    index,
                    quelle: d.pfad.clone(),
                    ziel_pfad: d.pfad.with_extension(ziel.endung()),
                    cover: self.cover_fuer(&d.ordner),
                });
            }
        }
        for a in &auftraege {
            self.dateien[a.index].status = Status::Wartet;
        }

        self.gesamt = auftraege.len();
        self.erledigt = 0;
        self.fehler = 0;
        self.laeuft = true;
        self.protokoll_bereit = false;
        self.zielordner = None;
        self.meldung = None;
        self.abbruch = Arc::new(AtomicBool::new(false));

        let (tx, rx) = channel();
        self.rx = Some(rx);
        auftraege::starte(auftraege, ziel, self.abbruch.clone(), tx, ctx.clone());
    }

    pub fn abbrechen(&mut self) {
        self.abbruch.store(true, Ordering::Relaxed);
    }

    // Nachrichten der Hintergrund-Threads abholen und Tabelle aktualisieren
    pub fn nachrichten_verarbeiten(&mut self) {
        let Some(rx) = &self.rx else { return };
        let nachrichten: Vec<Msg> = rx.try_iter().collect();
        for m in nachrichten {
            let (index, status) = match m {
                Msg::Laeuft(i) => (i, Status::Laeuft),
                Msg::Fertig(i) => {
                    self.erledigt += 1;
                    (i, Status::Fertig)
                }
                Msg::Fehler(i, e) => {
                    self.erledigt += 1;
                    self.fehler += 1;
                    (i, Status::Fehler(e))
                }
                // Abgebrochene Datei wurde gelöscht und ist wieder wählbar
                Msg::Abgebrochen(i) => (i, Status::Bereit),
                Msg::Ende => {
                    self.lauf_beenden();
                    continue;
                }
            };
            if let Some(d) = self.dateien.get_mut(index) {
                d.status = status;
            }
        }
    }

    fn lauf_beenden(&mut self) {
        self.laeuft = false;
        self.protokoll_bereit = true;
        self.zielordner = self
            .dateien
            .iter()
            .find(|d| d.status == Status::Fertig)
            .map(|d| d.ordner.clone());
        self.rx = None;
        // Nicht gestartete Dateien (nach Abbruch) wieder freigeben
        for d in &mut self.dateien {
            if d.status == Status::Wartet {
                d.status = Status::Bereit;
            }
        }
        let ok = self.erledigt - self.fehler;
        let offen = self.gesamt - self.erledigt; // nicht begonnen oder abgebrochen
        let t = t();
        self.meldung = Some(match (self.fehler, offen) {
            (0, 0) => format!("{}: {ok} {}", t.m_fertig, t.m_umgewandelt),
            (f, 0) => format!(
                "{}: {ok} {}, {f} {}",
                t.m_fertig, t.m_umgewandelt, t.m_fehler
            ),
            (f, o) => format!(
                "{}: {ok} {}, {f} {}, {o} {}",
                t.m_abgebrochen, t.m_umgewandelt, t.m_fehler, t.m_nicht_begonnen
            ),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::konverter::tests::{ordner, wav};
    use crate::konverter::wandle;

    // Wie die Oberfläche: starten und Nachrichten abholen, bis der Lauf endet
    fn durchlaufen(app: &mut App) {
        app.starten(&egui::Context::default());
        while app.laeuft {
            std::thread::sleep(std::time::Duration::from_millis(10));
            app.nachrichten_verarbeiten();
        }
    }

    #[test]
    fn zwei_quellen_mit_gleichem_ziel() {
        // „a.wav“ und „a.aif“ ergeben beide „a.mp3“: eine klappt, die andere meldet
        // einen Fehler und darf die fertige Datei nicht löschen
        let o = ordner("gleiches-ziel");
        wav(&o.join("a.wav"), 44_100, 16, 0.5);
        let nie = AtomicBool::new(false);
        wandle(&o.join("a.wav"), &o.join("a.aif"), Ziel::Aiff, None, &nie).unwrap();

        let mut app = App::default();
        app.ordner_hinzufuegen(o.to_path_buf());
        app.ziel_setzen(Ziel::Mp3);
        assert_eq!(app.anzahl_gewaehlt(), 2);
        durchlaufen(&mut app);

        let fertig = app
            .dateien
            .iter()
            .filter(|d| d.status == Status::Fertig)
            .count();
        let fehler = app
            .dateien
            .iter()
            .filter(|d| matches!(d.status, Status::Fehler(_)))
            .count();
        assert_eq!((fertig, fehler), (1, 1));
        assert!(std::fs::metadata(o.join("a.mp3")).unwrap().len() > 10_000);
        assert!(app.protokoll_bereit);
        assert_eq!(app.zielordner, Some(o.to_path_buf())); // „Zielordner öffnen“ zeigt hierher
    }

    #[test]
    fn ordner_ohne_cover_und_cover_zuordnung() {
        let o = ordner("cover-zuordnung");
        std::fs::create_dir_all(o.join("mit")).unwrap();
        std::fs::create_dir_all(o.join("ohne")).unwrap();
        wav(&o.join("mit/a.wav"), 44_100, 16, 0.1);
        wav(&o.join("ohne/b.wav"), 44_100, 16, 0.1);
        std::fs::write(o.join("mit/z.png"), b"x").unwrap();
        std::fs::write(o.join("mit/b.jpg"), b"x").unwrap();

        let mut app = App::default();
        app.ordner_hinzufuegen(o.to_path_buf());
        // Je Ordner eine Kopfzeile, danach seine Dateien
        assert_eq!(app.zeilen.len(), 4);
        assert!(matches!(app.zeilen[0], Zeile::Kopf { anzahl: 1, .. }));
        assert!(matches!(app.zeilen[1], Zeile::Datei(0)));
        assert!(matches!(app.zeilen[2], Zeile::Kopf { anzahl: 1, .. }));
        assert!(matches!(app.zeilen[3], Zeile::Datei(1)));
        assert_eq!(app.ohne_cover.len(), 1);
        assert_eq!(app.letzter_ordner, o.parent().map(Path::to_path_buf));
        assert_eq!(app.ohne_cover[0].0, o.join("ohne"));
        assert_eq!(app.cover_fuer(&o.join("mit")), Some(o.join("mit/b.jpg")));
        assert_eq!(
            app.dateien[0].ordner_anzeige,
            PathBuf::from("cover-zuordnung")
                .join("mit")
                .display()
                .to_string()
        );
    }

    // Ganzen Ordner ohne Oberfläche umwandeln, z. B. zum Messen:
    // AK_ORDNER=D:\Musik AK_ZIEL=mp3 (oder flac) cargo test --release -- --ignored --nocapture ordner_umwandeln
    #[test]
    #[ignore = "braucht einen echten Ordner (AK_ORDNER), siehe Kommentar"]
    fn ordner_umwandeln() {
        let mut app = App::default();
        app.ordner_hinzufuegen(PathBuf::from(
            std::env::var("AK_ORDNER").expect("AK_ORDNER setzen"),
        ));
        match std::env::var("AK_ZIEL").as_deref() {
            Ok("mp3") => app.ziel_setzen(Ziel::Mp3),
            Ok("flac") => app.ziel_setzen(Ziel::Flac),
            _ => {}
        }
        let start = std::time::Instant::now();
        durchlaufen(&mut app);
        for d in &app.dateien {
            if let Status::Fehler(e) = &d.status {
                println!("Fehler: {}: {e}", d.pfad.display());
            }
        }
        println!(
            "{} Dateien in {:.1} s · {}",
            app.gesamt,
            start.elapsed().as_secs_f32(),
            app.meldung.unwrap_or_default()
        );
    }
}
