// Release-Build ohne zusätzliches Konsolenfenster (nicht beim Testlauf, sonst fehlt dessen Ausgabe)
#![cfg_attr(all(not(debug_assertions), not(test)), windows_subsystem = "windows")]

mod aiff; // AIFF-Datei schreiben
mod app; // Zustand und Ablaufsteuerung
mod auftraege; // Umwandlungen parallel im Hintergrund
mod bericht; // CSV-Protokoll
mod einstellungen; // Einstellungen merken
mod flac; // FLAC-Datei schreiben
mod gui; // Oberfläche
mod icons; // Lucide-Icons
mod konverter; // eigener Konverter: lesen, umrechnen, runden
mod mp3; // MP3-Datei schreiben (LAME)
mod scan; // Ordner durchsuchen
mod tags; // Tags und Cover (ID3, Vorbis)
mod texte; // Texte in Deutsch und Englisch

use eframe::egui;
use std::path::PathBuf;

// Fenster-Icon (Titelleiste und Taskleiste); die .exe bekommt ihr Icon über build.rs
fn fenster_icon() -> egui::IconData {
    let bild = image::load_from_memory(include_bytes!("../assets/icon.png"))
        .expect("assets/icon.png ist eingebaut und gültig")
        .into_rgba8();
    let (width, height) = bild.dimensions();
    egui::IconData {
        rgba: bild.into_raw(),
        width,
        height,
    }
}

fn main() -> eframe::Result<()> {
    texte::laden();
    // Ordner aus der Kommandozeile übernehmen (z. B. Verknüpfung oder „Öffnen mit“)
    let ordner: Vec<PathBuf> = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
        .collect();

    let gemerkt = einstellungen::Einstellungen::laden();
    let (breite, hoehe) = gemerkt.fenster.unwrap_or((1000.0, 600.0));

    let optionen = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([breite.clamp(720.0, 8000.0), hoehe.clamp(400.0, 8000.0)])
            .with_min_inner_size([720.0, 400.0])
            .with_drag_and_drop(true)
            .with_icon(fenster_icon()),
        ..Default::default()
    };
    eframe::run_native(
        "Audio-Konverter",
        optionen,
        Box::new(move |cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            gui::stil(&cc.egui_ctx);
            cc.egui_ctx.set_theme(gui::praeferenz(gemerkt.thema));
            let mut app = app::App::neu(gemerkt);
            for o in ordner {
                app.ordner_hinzufuegen(o);
            }
            Ok(Box::new(app))
        }),
    )
}
