// Oberfläche: Werkzeugleiste oben, Quellordner links, Dateiliste in der Mitte,
// Statusleiste mit Hauptaktion unten, Fenster „Cover“ bei Bedarf.

use crate::app::{ordner_oeffnen, App, Datei, Status, Zeile};
use crate::einstellungen::Thema;
use crate::konverter::Ziel;
use crate::texte::{self, t, Sprache};
use crate::{bericht, icons};
use eframe::egui::{
    self, vec2, Align, Align2, Checkbox, Color32, FontId, ImageSource, Label, Layout, Margin,
    RichText, Sense, Ui, UiBuilder,
};
use std::path::Path;

const ZEILE: f32 = 22.0; // Zeilenhöhe der Tabelle
const ICON: f32 = 14.0; // Icon-Größe in Knöpfen
const GRUEN: Color32 = Color32::from_rgb(60, 170, 100);
const AKZENT: Color32 = Color32::from_rgb(37, 99, 235); // kräftig genug für Weiß in hell und dunkel

// Feste Spaltenbreiten; Datei und Ordner teilen sich den Rest
const B_HAKEN: f32 = 28.0;
const B_NR: f32 = 44.0;
const B_FORMAT: f32 = 104.0;
const B_STATUS: f32 = 140.0;

// ---------------------------------------------------------------------------
// Stil: kompakt, ruhig, ohne Schatten. Gilt für helles und dunkles Systemthema.
// ---------------------------------------------------------------------------

pub fn stil(ctx: &egui::Context) {
    ctx.all_styles_mut(|s| {
        use egui::TextStyle::{Body, Button, Heading, Monospace, Small};
        s.text_styles = [
            (Heading, FontId::proportional(15.0)),
            (Body, FontId::proportional(13.0)),
            (Button, FontId::proportional(13.0)),
            (Small, FontId::proportional(11.0)),
            (Monospace, FontId::monospace(12.0)),
        ]
        .into();
        s.spacing.item_spacing = vec2(6.0, 4.0);
        s.spacing.button_padding = vec2(8.0, 3.0);
        s.spacing.interact_size.y = ZEILE;
        s.animation_time = 0.2; // sanfte Übergänge (Hover, Auf- und Zuklappen)
        s.visuals.window_shadow = egui::Shadow::NONE;
        s.visuals.popup_shadow = egui::Shadow::NONE;
        s.visuals.window_rounding = 6.0.into();
        s.visuals.menu_rounding = 4.0.into();
        let w = &mut s.visuals.widgets;
        for z in [
            &mut w.noninteractive,
            &mut w.inactive,
            &mut w.hovered,
            &mut w.active,
            &mut w.open,
        ] {
            z.rounding = 4.0.into();
        }
    });
}

// Gewähltes Thema für egui: „System“ folgt dem Betriebssystem
pub fn praeferenz(thema: Thema) -> egui::ThemePreference {
    match thema {
        Thema::System => egui::ThemePreference::System,
        Thema::Hell => egui::ThemePreference::Light,
        Thema::Dunkel => egui::ThemePreference::Dark,
    }
}

// ---------------------------------------------------------------------------
// Kleine Bausteine
// ---------------------------------------------------------------------------

// Knopf mit Icon und Text in Textfarbe
fn knopf<'a>(ui: &Ui, icon: ImageSource<'static>, text: &'a str) -> egui::Button<'a> {
    egui::Button::image_and_text(icons::bild(icon, ICON, ui.visuals().text_color()), text)
}

// Hauptaktion: gefüllt in Akzentfarbe
fn haupt_knopf<'a>(icon: ImageSource<'static>, text: &'a str) -> egui::Button<'a> {
    egui::Button::image_and_text(
        icons::bild(icon, ICON, Color32::WHITE),
        RichText::new(text).color(Color32::WHITE),
    )
    .fill(AKZENT)
    .min_size(vec2(112.0, ZEILE + 2.0))
}

// Kleine Überschrift, z. B. „QUELLEN“
fn ueberschrift(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text.to_uppercase()).small().strong().weak());
}

// Eine Tabellenzeile über die volle Breite, optional mit Hintergrund
fn zeile(ui: &mut Ui, hintergrund: Option<Color32>, inhalt: impl FnOnce(&mut Ui)) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), ZEILE), Sense::hover());
    if let Some(f) = hintergrund {
        ui.painter().rect_filled(rect, 0.0, f);
    }
    // Hover-Hintergrund blendet weich ein und aus
    let id = ui.id().with(("zeile", rect.min.y as i32));
    let hover = ui.rect_contains_pointer(rect);
    let stufe = ui.ctx().animate_bool_with_time(id, hover, 0.12);
    if stufe > 0.0 {
        let farbe = ui
            .visuals()
            .widgets
            .hovered
            .weak_bg_fill
            .gamma_multiply(0.4 * stufe);
        ui.painter().rect_filled(rect, 0.0, farbe);
    }
    let mut kind = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::left_to_right(Align::Center)),
    );
    kind.spacing_mut().item_spacing.x = 0.0;
    inhalt(&mut kind);
}

// Eine Zelle mit fester Breite und kleinem linken Innenabstand
fn zelle(ui: &mut Ui, breite: f32, inhalt: impl FnOnce(&mut Ui)) {
    ui.allocate_ui_with_layout(
        vec2(breite, ZEILE),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.set_width(breite);
            ui.spacing_mut().item_spacing.x = 4.0;
            ui.add_space(6.0);
            inhalt(ui);
        },
    );
}

// Status als Icon und Text in passender Farbe
fn status_anzeige(ui: &mut Ui, status: &Status) {
    let v = ui.visuals();
    let (icon, farbe) = match status {
        Status::Bereit => (Some(icons::BEREIT), v.weak_text_color()),
        Status::Gesperrt => (Some(icons::GESPERRT), v.weak_text_color()),
        Status::Existiert => (Some(icons::EXISTIERT), v.warn_fg_color),
        Status::Wartet => (Some(icons::WARTET), v.weak_text_color()),
        Status::Laeuft => (None, AKZENT),
        Status::Fertig => (Some(icons::FERTIG), GRUEN),
        Status::Fehler(_) => (Some(icons::FEHLER), v.error_fg_color),
    };
    match icon {
        Some(i) => ui.add(icons::bild(i, 13.0, farbe)),
        None => ui.add(egui::Spinner::new().size(12.0).color(farbe)),
    };
    let antwort = ui.label(RichText::new(status.text()).color(farbe));
    if let Status::Fehler(e) = status {
        antwort.on_hover_text(e);
    }
}

// Kopfzeile eines Ordners: Icon, kurzer Pfad, Anzahl der Dateien
fn kopfzeile(ui: &mut Ui, anzeige: &str, pfad: &Path, anzahl: usize, spalten: &[f32; 5]) {
    let hintergrund = ui
        .visuals()
        .widgets
        .inactive
        .weak_bg_fill
        .gamma_multiply(0.7);
    let text = ui.visuals().text_color();
    zeile(ui, Some(hintergrund), |ui| {
        zelle(ui, spalten[0], |ui| {
            ui.add(icons::bild(icons::ORDNER, ICON, text));
        });
        zelle(ui, spalten[1] + spalten[2], |ui| {
            ui.add(Label::new(RichText::new(anzeige).strong()).truncate())
                .on_hover_text(pfad.display().to_string());
        });
        zelle(ui, spalten[3] + spalten[4], |ui| {
            ui.label(RichText::new(format!("{anzahl} {}", t().dateien)).weak());
        });
    });
}

// Eine Dateizeile
fn dateizeile(ui: &mut Ui, d: &mut Datei, nr: usize, spalten: &[f32; 5], ziel: Ziel, laeuft: bool) {
    zeile(ui, None, |ui| {
        if d.status == Status::Gesperrt {
            ui.disable(); // ganze Zeile ausgegraut
        }
        let waehlbar = d.waehlbar() && !laeuft;
        zelle(ui, spalten[0], |ui| {
            ui.add_enabled(waehlbar, Checkbox::without_text(&mut d.gewaehlt));
        });
        zelle(ui, spalten[1], |ui| {
            ui.label(RichText::new(nr.to_string()).weak());
        });
        zelle(ui, spalten[2], |ui| {
            ui.add(Label::new(d.name.as_str()).truncate())
                .on_hover_text(d.pfad.display().to_string());
        });
        zelle(ui, spalten[3], |ui| {
            let schwach = ui.visuals().weak_text_color();
            ui.label(RichText::new(d.von()).weak());
            if d.nach(ziel) != "-" {
                ui.add(icons::bild(icons::PFEIL, 11.0, schwach));
                ui.label(RichText::new(d.nach(ziel)).weak());
            }
        });
        zelle(ui, spalten[4], |ui| status_anzeige(ui, &d.status));
    });
}

// ---------------------------------------------------------------------------
// Bereiche des Fensters
// ---------------------------------------------------------------------------

impl App {
    // Oben: Ordner hinzufügen, Zielformat, Cover, Protokoll
    fn werkzeugleiste(&mut self, ctx: &egui::Context) {
        let t = t();
        let rahmen =
            egui::Frame::side_top_panel(&ctx.style()).inner_margin(Margin::symmetric(10.0, 6.0));
        egui::TopBottomPanel::top("werkzeuge")
            .frame(rahmen)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add_enabled_ui(!self.laeuft, |ui| {
                        if ui
                            .add(knopf(ui, icons::ORDNER_PLUS, t.ordner_hinzufuegen))
                            .clicked()
                        {
                            let mut dialog = rfd::FileDialog::new();
                            if let Some(start) = &self.letzter_ordner {
                                dialog = dialog.set_directory(start); // dort weitermachen
                            }
                            for p in dialog.pick_folders().unwrap_or_default() {
                                self.ordner_hinzufuegen(p);
                            }
                        }
                        ui.separator();
                        ui.label(RichText::new(t.ziel).weak());
                        let mut ziel = self.ziel;
                        egui::ComboBox::from_id_salt("ziel")
                            .selected_text(ziel.name())
                            .width(190.0)
                            .show_ui(ui, |ui| {
                                for z in Ziel::ALLE {
                                    ui.selectable_value(&mut ziel, z, z.name());
                                }
                            });
                        self.ziel_setzen(ziel);
                    });

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        // Thema ganz rechts: wie das Betriebssystem, hell oder dunkel
                        let mut thema = self.thema;
                        egui::ComboBox::from_id_salt("thema")
                            .selected_text(thema.name())
                            .width(76.0)
                            .show_ui(ui, |ui| {
                                for th in Thema::ALLE {
                                    ui.selectable_value(&mut thema, th, th.name());
                                }
                            })
                            .response
                            .on_hover_text(t.thema_tip);
                        if thema != self.thema {
                            self.thema = thema;
                            ctx.set_theme(praeferenz(thema));
                        }
                        ui.label(RichText::new(t.thema).weak());
                        ui.separator();
                        // Sprache (nicht von einem Lauf gesperrt)
                        let mut sprache = texte::sprache();
                        egui::ComboBox::from_id_salt("sprache")
                            .selected_text(sprache.name())
                            .width(76.0)
                            .show_ui(ui, |ui| {
                                for s in Sprache::ALLE {
                                    ui.selectable_value(&mut sprache, s, s.name());
                                }
                            })
                            .response
                            .on_hover_text(t.sprache_tip);
                        if sprache != texte::sprache() {
                            texte::wechseln(sprache);
                        }
                        let schwach = ui.visuals().weak_text_color();
                        ui.add(icons::bild(icons::SPRACHE, ICON, schwach));
                        ui.separator();
                        if ui
                            .add_enabled(
                                self.protokoll_bereit,
                                knopf(ui, icons::PROTOKOLL, t.protokoll),
                            )
                            .on_hover_text(t.protokoll_tip)
                            .on_disabled_hover_text(t.protokoll_tip_aus)
                            .clicked()
                        {
                            self.protokoll_speichern();
                        }
                        let fehlen = self.ohne_cover.len();
                        let text = if fehlen == 0 {
                            t.cover.to_string()
                        } else {
                            format!("{} · {fehlen} {}", t.cover, t.cover_fehlen)
                        };
                        if ui
                            .add_enabled(
                                !self.ordner.is_empty(),
                                knopf(ui, icons::OHNE_COVER, &text),
                            )
                            .on_hover_text(t.cover_tip)
                            .clicked()
                        {
                            self.zeige_cover = !self.zeige_cover;
                        }
                    });
                });
            });
    }

    // Links: gewählte Quellordner
    fn quellen(&mut self, ctx: &egui::Context) {
        let t = t();
        let rahmen = egui::Frame::side_top_panel(&ctx.style()).inner_margin(Margin::same(10.0));
        egui::SidePanel::left("quellen")
            .frame(rahmen)
            .default_width(220.0)
            .width_range(160.0..=420.0)
            .show(ctx, |ui| {
                ueberschrift(ui, t.quellen);
                let schwach = ui.visuals().weak_text_color();
                let mut entfernen = None;
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, o) in self.ordner.iter().enumerate() {
                        let anzahl = self.dateien.iter().filter(|d| &d.wurzel == o).count();
                        let name = o.file_name().map_or_else(
                            || o.display().to_string(),
                            |n| n.to_string_lossy().to_string(),
                        );
                        ui.horizontal(|ui| {
                            ui.add(icons::bild(icons::ORDNER, ICON, schwach));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let x = egui::Button::image(icons::bild(
                                    icons::ENTFERNEN,
                                    12.0,
                                    schwach,
                                ))
                                .frame(false);
                                if ui
                                    .add_enabled(!self.laeuft, x)
                                    .on_hover_text(t.ordner_entfernen)
                                    .clicked()
                                {
                                    entfernen = Some(i);
                                }
                                ui.vertical(|ui| {
                                    ui.spacing_mut().item_spacing.y = 0.0;
                                    ui.add(Label::new(name).truncate())
                                        .on_hover_text(o.display().to_string());
                                    ui.label(
                                        RichText::new(format!("{anzahl} {}", t.dateien))
                                            .small()
                                            .weak(),
                                    );
                                });
                            });
                        });
                        ui.add_space(4.0);
                    }
                });
                if let Some(i) = entfernen {
                    self.ordner_entfernen(i);
                }
            });
    }

    // Unten: Zusammenfassung, Fortschritt, Hauptaktion
    fn statusleiste(&mut self, ctx: &egui::Context) {
        let t = t();
        let rahmen =
            egui::Frame::side_top_panel(&ctx.style()).inner_margin(Margin::symmetric(10.0, 6.0));
        egui::TopBottomPanel::bottom("status")
            .frame(rahmen)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let mut info = format!("{} {}", self.dateien.len(), t.dateien);
                    let gewaehlt = self.anzahl_gewaehlt();
                    if !self.laeuft && gewaehlt > 0 {
                        info = format!("{info} · {gewaehlt} {}", t.ausgewaehlt);
                    }
                    if let Some(m) = &self.meldung {
                        info = format!("{info} · {m}");
                    }
                    ui.label(RichText::new(info).weak());

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if self.laeuft {
                            if ui.add(knopf(ui, icons::STOPP, t.abbrechen)).clicked() {
                                self.abbrechen();
                            }
                            // Balken füllt sich weich statt in Sprüngen
                            let ziel_anteil = self.erledigt as f32 / self.gesamt.max(1) as f32;
                            let anteil = ctx.animate_value_with_time(
                                egui::Id::new("fortschritt"),
                                ziel_anteil,
                                0.25,
                            );
                            ui.label(
                                RichText::new(format!("{} / {}", self.erledigt, self.gesamt))
                                    .weak(),
                            );
                            ui.add(
                                egui::ProgressBar::new(anteil)
                                    .desired_width(160.0)
                                    .desired_height(6.0),
                            );
                        } else {
                            // Balken fürs nächste Mal zurücksetzen, ohne rückwärts zu laufen
                            ctx.animate_value_with_time(egui::Id::new("fortschritt"), 0.0, 0.0);
                            if ui
                                .add_enabled(
                                    self.kann_starten(),
                                    haupt_knopf(icons::START, t.umwandeln),
                                )
                                .on_disabled_hover_text(t.keine_auswahl)
                                .clicked()
                            {
                                self.starten(ctx);
                            }
                            if let Some(ordner) = &self.zielordner {
                                if ui
                                    .add(knopf(ui, icons::ORDNER_OFFEN, t.zielordner_oeffnen))
                                    .on_hover_text(ordner.display().to_string())
                                    .clicked()
                                {
                                    ordner_oeffnen(ordner);
                                }
                            }
                        }
                    });
                });
            });
    }

    // Mitte: Dateiliste oder Hinweis, wenn noch nichts gewählt ist
    fn liste(&mut self, ctx: &egui::Context) {
        let t = t();
        let rahmen =
            egui::Frame::central_panel(&ctx.style()).inner_margin(Margin::symmetric(8.0, 6.0));
        egui::CentralPanel::default().frame(rahmen).show(ctx, |ui| {
            if self.dateien.is_empty() {
                leer(ui);
                return;
            }
            let laeuft = self.laeuft;
            let ziel = self.ziel;
            let rest = (ui.available_width() - B_HAKEN - B_NR - B_FORMAT - B_STATUS).max(200.0);
            let spalten = [B_HAKEN, B_NR, rest, B_FORMAT, B_STATUS];

            // Kopfzeile mit „alle an/aus“
            let waehlbar = self.anzahl_waehlbar();
            let gewaehlt = self.anzahl_gewaehlt();
            zeile(ui, None, |ui| {
                zelle(ui, spalten[0], |ui| {
                    let mut alle = waehlbar > 0 && gewaehlt == waehlbar;
                    let kasten = Checkbox::without_text(&mut alle)
                        .indeterminate(gewaehlt > 0 && gewaehlt < waehlbar);
                    if ui
                        .add_enabled(waehlbar > 0 && !laeuft, kasten)
                        .on_hover_text(t.alle_tip)
                        .changed()
                    {
                        self.alle_waehlen(alle);
                    }
                });
                for (titel, breite) in t.spalten.into_iter().zip(&spalten[1..]) {
                    zelle(ui, *breite, |ui| {
                        ui.label(RichText::new(titel).small().strong().weak());
                    });
                }
            });
            ui.add(egui::Separator::default().spacing(2.0));

            // Nur sichtbare Zeilen zeichnen, auch bei tausenden Dateien flüssig
            ui.spacing_mut().item_spacing.y = 0.0;
            egui::ScrollArea::vertical().auto_shrink(false).show_rows(
                ui,
                ZEILE,
                self.zeilen.len(),
                |ui, bereich| {
                    for z in bereich {
                        match &self.zeilen[z] {
                            Zeile::Kopf {
                                anzeige,
                                pfad,
                                anzahl,
                            } => kopfzeile(ui, anzeige, pfad, *anzahl, &spalten),
                            Zeile::Datei(i) => {
                                dateizeile(
                                    ui,
                                    &mut self.dateien[*i],
                                    i + 1,
                                    &spalten,
                                    ziel,
                                    laeuft,
                                );
                            }
                        }
                    }
                },
            );
        });
    }

    // Fenster „Cover“: Ordner ohne Bild und alle gefundenen Bilder
    fn cover_fenster(&mut self, ctx: &egui::Context) {
        if !self.zeige_cover {
            return;
        }
        let t = t();
        let mut offen = true;
        egui::Window::new(t.cover)
            .id(egui::Id::new("cover"))
            .open(&mut offen)
            .collapsible(false)
            .default_size([480.0, 360.0])
            .show(ctx, |ui| {
                let v = ui.visuals();
                let (warn, schwach) = (v.warn_fg_color, v.weak_text_color());
                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        ueberschrift(
                            ui,
                            &format!("{} · {}", t.ordner_ohne_cover, self.ohne_cover.len()),
                        );
                        if self.ohne_cover.is_empty() {
                            ui.label(RichText::new(t.alle_haben_bild).weak());
                        }
                        for (pfad, anzeige) in &self.ohne_cover {
                            ui.horizontal(|ui| {
                                ui.add(icons::bild(icons::OHNE_COVER, ICON, warn));
                                ui.add(Label::new(anzeige.as_str()).truncate())
                                    .on_hover_text(pfad.display().to_string());
                            });
                        }
                        ui.add_space(10.0);
                        ueberschrift(
                            ui,
                            &format!("{} · {}", t.gefundene_bilder, self.bilder.len()),
                        );
                        ui.label(RichText::new(t.bilder_hinweis).small().weak());
                        for b in &self.bilder {
                            ui.horizontal(|ui| {
                                ui.add(icons::bild(icons::BILD, ICON, schwach));
                                ui.label(b.name.as_str());
                                ui.add(
                                    Label::new(RichText::new(b.ordner_anzeige.as_str()).weak())
                                        .truncate(),
                                )
                                .on_hover_text(b.pfad.display().to_string());
                            });
                        }
                    });
            });
        self.zeige_cover = offen;
    }

    // Ordner per Drag & Drop aufnehmen; beim Ziehen Hinweis über dem Fenster
    fn ablegen(&mut self, ctx: &egui::Context) {
        let (abgelegt, schwebt): (Vec<_>, bool) = ctx.input(|i| {
            (
                i.raw
                    .dropped_files
                    .iter()
                    .filter_map(|f| f.path.clone())
                    .collect(),
                !i.raw.hovered_files.is_empty(),
            )
        });
        for p in abgelegt {
            if p.is_dir() {
                self.ordner_hinzufuegen(p);
            } else {
                self.meldung = Some(t().nur_ordner.into());
            }
        }
        if schwebt && !self.laeuft {
            let maler = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("ablegen"),
            ));
            let rect = ctx.screen_rect();
            maler.rect_filled(rect, 0.0, Color32::from_black_alpha(150));
            maler.text(
                rect.center(),
                Align2::CENTER_CENTER,
                t().hier_ablegen,
                FontId::proportional(18.0),
                Color32::WHITE,
            );
        }
    }

    // Protokoll als CSV speichern (Dialog „Speichern unter“)
    fn protokoll_speichern(&mut self) {
        let Some(pfad) = rfd::FileDialog::new()
            .set_file_name(t().protokoll_datei)
            .add_filter("CSV", &["csv"])
            .save_file()
        else {
            return;
        };
        self.meldung = Some(match bericht::speichern(&self.dateien, self.ziel, &pfad) {
            Ok(()) => format!("{}: {}", t().protokoll_gespeichert, pfad.display()),
            Err(e) => format!("{}: {e}", t().protokoll_nicht_gespeichert),
        });
    }
}

// Hinweis bei leerer Liste
fn leer(ui: &mut Ui) {
    let t = t();
    let schwach = ui.visuals().weak_text_color();
    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() * 0.3);
        ui.add(icons::bild(icons::ORDNER_OFFEN, 40.0, schwach));
        ui.add_space(8.0);
        ui.label(RichText::new(t.leer_titel).weak());
        ui.label(RichText::new(t.leer_info).small().weak());
    });
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Fenstergröße merken (nicht beim Minimieren oder Maximieren)
        let (klein, gross) = ctx.input(|i| {
            let v = i.viewport();
            (v.minimized == Some(true), v.maximized == Some(true))
        });
        if !klein && !gross {
            let groesse = ctx.screen_rect().size();
            self.fenster = Some((groesse.x, groesse.y));
        }
        self.nachrichten_verarbeiten();
        self.ablegen(ctx);
        self.werkzeugleiste(ctx);
        self.statusleiste(ctx);
        self.quellen(ctx);
        self.liste(ctx);
        self.cover_fenster(ctx);
    }

    // Beim Beenden Einstellungen merken
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.einstellungen().speichern();
    }
}
