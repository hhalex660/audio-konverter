// Eigener Konverter (ersetzt ffmpeg):
// Datei lesen (symphonia) -> auf 44,1 kHz umrechnen (rubato) -> auf 16 Bit runden
// -> als AIFF, MP3 oder FLAC schreiben. Läuft blockweise, braucht also auch bei langen
// DJ-Mixen kaum Speicher.

use crate::aiff::AiffSchreiber;
use crate::flac::FlacSchreiber;
use crate::mp3::Mp3Schreiber;
use crate::tags;
use crate::texte::t;
use rubato::{FftFixedIn, Resampler};
use std::fs::{File, OpenOptions};
use std::io::ErrorKind;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use symphonia::core::audio::{AudioBufferRef, SampleBuffer};
use symphonia::core::codecs::{Decoder, DecoderOptions};
use symphonia::core::errors::Error as SymFehler;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::{MetadataOptions, Tag as QuellTag};
use symphonia::core::probe::Hint;

const ZIELRATE: usize = 44_100;
const BLOCK: usize = 4096; // Eingangsblock des Resamplers (Frames)

// Fehlertext bei Abbruch durch den Nutzer
pub const ABGEBROCHEN: &str = "abgebrochen";

// ---------------------------------------------------------------------------
// Zielformat und Sperrregel
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Default)]
pub enum Ziel {
    #[default]
    Aiff,
    Mp3,
    Flac,
}

impl Ziel {
    pub const ALLE: [Ziel; 3] = [Ziel::Aiff, Ziel::Mp3, Ziel::Flac];

    // Beschreibung für die Auswahl (in der gewählten Sprache)
    pub fn name(self) -> &'static str {
        match self {
            Ziel::Aiff => t().ziel_aiff,
            Ziel::Mp3 => t().ziel_mp3,
            Ziel::Flac => t().ziel_flac,
        }
    }
    pub fn kurz(self) -> &'static str {
        match self {
            Ziel::Aiff => "AIFF",
            Ziel::Mp3 => "MP3",
            Ziel::Flac => "FLAC",
        }
    }
    pub fn endung(self) -> &'static str {
        match self {
            Ziel::Aiff => "aiff",
            Ziel::Mp3 => "mp3",
            Ziel::Flac => "flac",
        }
    }
}

// MP3 ist immer gesperrt, AIFF nur beim Ziel AIFF, FLAC nur beim Ziel FLAC
pub fn ist_gesperrt(endung: &str, ziel: Ziel) -> bool {
    endung == "mp3"
        || (ziel == Ziel::Aiff && (endung == "aiff" || endung == "aif"))
        || (ziel == Ziel::Flac && endung == "flac")
}

// ---------------------------------------------------------------------------
// Ausgabe: AIFF-, MP3- oder FLAC-Datei
// ---------------------------------------------------------------------------

enum Senke {
    Aiff(AiffSchreiber),
    Mp3(Mp3Schreiber),
    Flac(Box<FlacSchreiber>),
}

impl Senke {
    fn schreibe(&mut self, pcm: &[i16]) -> Result<(), String> {
        match self {
            Senke::Aiff(a) => a.schreibe(pcm).map_err(|e| e.to_string()),
            Senke::Mp3(m) => m.schreibe(pcm),
            Senke::Flac(f) => f.schreibe(pcm),
        }
    }
    fn fertig(self) -> Result<(), String> {
        match self {
            Senke::Aiff(a) => a.fertig(),
            Senke::Mp3(m) => m.fertig(),
            Senke::Flac(f) => f.fertig(),
        }
    }
}

// Letzte Stufe: Float -> 16 Bit (mit Dither) -> Datei
struct Ausgang {
    senke: Senke,
    dither: bool,     // nur bei AIFF und FLAC und nur wenn wirklich Bits wegfallen
    rng: u32,         // Zufallszahlen für den Dither
    skip: usize,      // Verzögerung des Resamplers am Anfang, wird verworfen
    max: Option<u64>, // am Ende: nicht mehr als die erwartete Länge schreiben
    geschrieben: u64, // bisher geschriebene Frames
}

impl Ausgang {
    // Zufallszahl in [0, 1) (xorshift, reicht für Dither völlig)
    fn zufall(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        (self.rng >> 8) as f32 / 16_777_216.0
    }

    // Ein Sample auf 16 Bit runden. TPDF-Dither = Summe zweier Zufallszahlen minus 1 LSB.
    fn quantisiere(&mut self, x: f32) -> i16 {
        let mut v = x * 32768.0;
        if self.dither {
            v += self.zufall() - self.zufall();
        }
        v.round().clamp(-32768.0, 32767.0) as i16
    }

    // Einen Block (ein Vec pro Kanal) schreiben
    fn ausgeben(&mut self, kan: &[Vec<f32>]) -> Result<(), String> {
        let n = kan[0].len();
        let start = self.skip.min(n);
        self.skip -= start;
        let mut anzahl = n - start;
        if let Some(max) = self.max {
            anzahl = anzahl.min(max.saturating_sub(self.geschrieben) as usize);
        }
        if anzahl == 0 {
            return Ok(());
        }
        let mut pcm = Vec::with_capacity(anzahl * kan.len());
        for i in start..start + anzahl {
            for k in kan {
                let s = self.quantisiere(k[i]);
                pcm.push(s);
            }
        }
        self.geschrieben += anzahl as u64;
        self.senke.schreibe(&pcm)
    }
}

// ---------------------------------------------------------------------------
// Strom: sammelt Samples, rechnet blockweise auf 44,1 kHz um
// ---------------------------------------------------------------------------

struct Strom {
    resampler: Option<FftFixedIn<f32>>, // None = Quelle hat schon 44,1 kHz
    pending: Vec<Vec<f32>>,             // noch nicht verarbeitete Samples je Kanal
    ausgang: Ausgang,
}

impl Strom {
    fn neu(rate: usize, kanaele: usize, mut ausgang: Ausgang) -> Result<Self, String> {
        let resampler = if rate == ZIELRATE {
            None
        } else {
            let rs = FftFixedIn::<f32>::new(rate, ZIELRATE, BLOCK, 2, kanaele)
                .map_err(|e| e.to_string())?;
            ausgang.skip = rs.output_delay();
            Some(rs)
        };
        Ok(Self {
            resampler,
            pending: vec![Vec::new(); kanaele],
            ausgang,
        })
    }

    // Interleavte Samples entgegennehmen und alles verarbeiten, was vollständig ist
    fn zufuehren(&mut self, interleaved: &[f32]) -> Result<(), String> {
        let k = self.pending.len();
        for frame in interleaved.chunks_exact(k) {
            for (kanal, s) in self.pending.iter_mut().zip(frame) {
                kanal.push(*s);
            }
        }
        match self.resampler.as_mut() {
            // Gleiche Rate: direkt ausgeben
            None => {
                let alle = std::mem::replace(&mut self.pending, vec![Vec::new(); k]);
                self.ausgang.ausgeben(&alle)
            }
            // Andere Rate: in festen Blöcken umrechnen
            Some(rs) => {
                let n = rs.input_frames_next();
                while self.pending[0].len() >= n {
                    let aus = {
                        let teil: Vec<&[f32]> = self.pending.iter().map(|c| &c[..n]).collect();
                        rs.process(&teil, None).map_err(|e| e.to_string())?
                    };
                    for c in &mut self.pending {
                        c.drain(..n);
                    }
                    self.ausgang.ausgeben(&aus)?;
                }
                Ok(())
            }
        }
    }

    // Reste verarbeiten, Datei abschließen
    fn abschliessen(self, frames_ein: u64, rate: usize) -> Result<(), String> {
        let Strom {
            mut resampler,
            pending,
            mut ausgang,
        } = self;
        if let Some(rs) = resampler.as_mut() {
            // Erwartete Länge: danach angehängte Auffüll-Stille wird abgeschnitten
            ausgang.max = Some((frames_ein * ZIELRATE as u64).div_ceil(rate as u64));
            if !pending[0].is_empty() {
                let rest: Vec<&[f32]> = pending.iter().map(Vec::as_slice).collect();
                let aus = rs
                    .process_partial(Some(rest.as_slice()), None)
                    .map_err(|e| e.to_string())?;
                ausgang.ausgeben(&aus)?;
            }
            // Ein Block Stille hinterher, damit der Rest des Filters herauskommt
            let aus = rs
                .process_partial(None::<&[Vec<f32>]>, None)
                .map_err(|e| e.to_string())?;
            ausgang.ausgeben(&aus)?;
        }
        ausgang.senke.fertig()
    }
}

// ---------------------------------------------------------------------------
// Einstieg
// ---------------------------------------------------------------------------

// Eine Datei umwandeln. `cover` ist ein Bild, das (wenn vorhanden) in die Metadaten kommt.
pub fn wandle(
    quelle: &Path,
    ziel_pfad: &Path,
    ziel: Ziel,
    cover: Option<&Path>,
    abbruch: &AtomicBool,
) -> Result<(), String> {
    // create_new: eine vorhandene Datei wird nie überschrieben, auch nicht wenn zwei
    // Aufträge gleichzeitig dasselbe Ziel haben (z. B. „Track.wav“ und „Track.aif“)
    let datei = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(ziel_pfad)
        .map_err(|e| match e.kind() {
            ErrorKind::AlreadyExists => t().zieldatei_existiert.to_string(),
            _ => format!("{}: {e}", t().zieldatei_nicht_anlegbar),
        })?;
    let ergebnis = lauf(quelle, datei, ziel, cover, abbruch);
    if ergebnis.is_err() {
        // Nur die eben selbst angelegte, halbfertige Datei löschen
        let _ = std::fs::remove_file(ziel_pfad);
    }
    ergebnis
}

// Geöffnete Quelle: Leser, Decoder und die Eckdaten der Audiospur
struct Quelle {
    leser: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    spur: u32,
    rate: usize,
    kanaele: usize,
    bits: Option<u32>,
    tags: Vec<QuellTag>,
}

// Quelle öffnen, Format erkennen, Tags einsammeln, Decoder bereitstellen
fn oeffne(quelle: &Path) -> Result<Quelle, String> {
    let datei = File::open(quelle).map_err(|e| format!("{}: {e}", t().quelle_nicht_lesbar))?;
    let mss = MediaSourceStream::new(Box::new(datei), MediaSourceStreamOptions::default());
    let mut hint = Hint::new();
    if let Some(e) = quelle.extension().and_then(|e| e.to_str()) {
        hint.with_extension(e);
    }
    let mut probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| format!("{}: {e}", t().format_nicht_erkannt))?;

    // Tags einsammeln (ID3 vor dem Container und Tags im Container)
    let mut tags = Vec::new();
    if let Some(m) = probed.metadata.get() {
        if let Some(r) = m.current() {
            tags.extend(r.tags().iter().cloned());
        }
    }
    let mut leser = probed.format;
    if let Some(r) = leser.metadata().current() {
        tags.extend(r.tags().iter().cloned());
    }

    let (spur, params) = {
        let s = leser.default_track().ok_or(t().keine_audiospur)?;
        (s.id, s.codec_params.clone())
    };
    let rate = params.sample_rate.ok_or(t().samplerate_unbekannt)? as usize;
    let kanaele = params.channels.ok_or(t().kanalzahl_unbekannt)?.count();
    if kanaele == 0 || kanaele > 2 {
        return Err(format!("{kanaele} {}", t().kanaele_nicht_unterstuetzt));
    }
    let decoder = symphonia::default::get_codecs()
        .make(&params, &DecoderOptions::default())
        .map_err(|e| format!("{}: {e}", t().codec_nicht_unterstuetzt))?;
    Ok(Quelle {
        leser,
        decoder,
        spur,
        rate,
        kanaele,
        bits: params.bits_per_sample,
        tags,
    })
}

// Pakete lesen und dekodieren, jeden dekodierten Puffer an `verarbeite` geben
fn dekodiere(
    q: &mut Quelle,
    abbruch: &AtomicBool,
    mut verarbeite: impl FnMut(AudioBufferRef) -> Result<(), String>,
) -> Result<(), String> {
    loop {
        if abbruch.load(Ordering::Relaxed) {
            return Err(ABGEBROCHEN.into());
        }
        let paket = match q.leser.next_packet() {
            Ok(p) => p,
            Err(SymFehler::IoError(e)) if e.kind() == ErrorKind::UnexpectedEof => break,
            Err(SymFehler::ResetRequired) => break,
            Err(e) => return Err(format!("{}: {e}", t().lesefehler)),
        };
        if paket.track_id() != q.spur {
            continue;
        }
        match q.decoder.decode(&paket) {
            Ok(puffer) => {
                if puffer.spec().channels.count() != q.kanaele {
                    return Err(t().kanalzahl_aendert_sich.into());
                }
                verarbeite(puffer)?;
            }
            Err(SymFehler::DecodeError(_)) => {} // defektes Paket überspringen
            Err(e) => return Err(format!("{}: {e}", t().dekodierfehler)),
        }
    }
    Ok(())
}

fn lauf(
    quelle: &Path,
    ziel_datei: File,
    ziel: Ziel,
    cover: Option<&Path>,
    abbruch: &AtomicBool,
) -> Result<(), String> {
    let q = oeffne(quelle)?;
    nach_pcm(q, ziel_datei, ziel, cover, abbruch)
}

// Alle Ziele: auf 44,1 kHz und 16 Bit bringen und schreiben
fn nach_pcm(
    mut q: Quelle,
    ziel_datei: File,
    ziel: Ziel,
    cover: Option<&Path>,
    abbruch: &AtomicBool,
) -> Result<(), String> {
    let (rate, kanaele) = (q.rate, q.kanaele);

    // Dither nur bei AIFF und FLAC und nur, wenn Bits wegfallen oder umgerechnet wird.
    // 16-Bit-Quellen mit 44,1 kHz werden dadurch bitgenau übernommen.
    let dither = ziel != Ziel::Mp3 && !(rate == ZIELRATE && q.bits == Some(16));

    let senke = match ziel {
        Ziel::Mp3 => Senke::Mp3(Mp3Schreiber::neu(
            ziel_datei,
            kanaele as u8,
            &tags::erzeuge(&q.tags, cover),
        )?),
        Ziel::Aiff => Senke::Aiff(
            AiffSchreiber::neu(ziel_datei, kanaele as u16, tags::erzeuge(&q.tags, cover))
                .map_err(|e| e.to_string())?,
        ),
        Ziel::Flac => Senke::Flac(Box::new(FlacSchreiber::neu(
            ziel_datei,
            kanaele,
            tags::flac_bloecke(&q.tags, cover),
        )?)),
    };
    let ausgang = Ausgang {
        senke,
        dither,
        rng: 0x1234_5678,
        skip: 0,
        max: None,
        geschrieben: 0,
    };
    let mut strom = Strom::neu(rate, kanaele, ausgang)?;

    let mut frames_ein: u64 = 0;
    dekodiere(&mut q, abbruch, |puffer| {
        let mut sb = SampleBuffer::<f32>::new(puffer.capacity() as u64, *puffer.spec());
        sb.copy_interleaved_ref(puffer);
        frames_ein += (sb.samples().len() / kanaele) as u64;
        strom.zufuehren(sb.samples())
    })?;
    strom.abschliessen(frames_ein, rate)
}

// ---------------------------------------------------------------------------
// Tests: `cargo test --release`
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::path::PathBuf;

    pub struct Testordner {
        _temp: tempfile::TempDir,
        pfad: PathBuf,
    }

    impl std::ops::Deref for Testordner {
        type Target = PathBuf;

        fn deref(&self) -> &Self::Target {
            &self.pfad
        }
    }

    // Jeder Test bekommt einen eigenen Ordner, der beim Verlassen gelöscht wird.
    pub fn ordner(name: &str) -> Testordner {
        let temp = tempfile::tempdir().unwrap();
        let o = temp.path().join(name);
        std::fs::create_dir_all(&o).unwrap();
        Testordner {
            _temp: temp,
            pfad: o,
        }
    }

    // Einfache PCM-WAV-Datei schreiben: Sinus 440 Hz, Stereo
    pub fn wav(pfad: &Path, rate: u32, bits: u16, sekunden: f32) -> Vec<i32> {
        let frames = (rate as f32 * sekunden) as u32;
        let bytes = u32::from(bits / 8);
        let mut samples = Vec::new();
        let mut daten = Vec::new();
        let max = (1i64 << (bits - 1)) as f32 * 0.5;
        for i in 0..frames {
            let v = ((i as f32 * 440.0 * std::f32::consts::TAU / rate as f32).sin() * max) as i32;
            for _ in 0..2 {
                samples.push(v);
                daten.extend_from_slice(&v.to_le_bytes()[..bytes as usize]);
            }
        }
        let mut f = Vec::new();
        f.extend_from_slice(b"RIFF");
        f.extend_from_slice(&(36 + daten.len() as u32).to_le_bytes());
        f.extend_from_slice(b"WAVEfmt ");
        f.extend_from_slice(&16u32.to_le_bytes());
        f.extend_from_slice(&1u16.to_le_bytes()); // PCM
        f.extend_from_slice(&2u16.to_le_bytes()); // Stereo
        f.extend_from_slice(&rate.to_le_bytes());
        f.extend_from_slice(&(rate * 2 * bytes).to_le_bytes());
        f.extend_from_slice(&(2 * bits / 8).to_le_bytes());
        f.extend_from_slice(&bits.to_le_bytes());
        f.extend_from_slice(b"data");
        f.extend_from_slice(&(daten.len() as u32).to_le_bytes());
        f.extend_from_slice(&daten);
        std::fs::write(pfad, f).unwrap();
        samples
    }

    // AIFF lesen: (Frames laut COMM, Audio-Samples aus SSND, Inhalt des ID3-Chunks)
    fn aiff_lesen(pfad: &Path) -> (u32, Vec<i16>, Vec<u8>) {
        let d = std::fs::read(pfad).unwrap();
        assert_eq!(&d[0..4], b"FORM");
        assert_eq!(&d[8..12], b"AIFF");
        assert_eq!(
            u32::from_be_bytes(d[4..8].try_into().unwrap()) as usize,
            d.len() - 8
        );
        let (mut frames, mut pcm, mut id3) = (0, Vec::new(), Vec::new());
        let mut p = 12;
        while p + 8 <= d.len() {
            let id = &d[p..p + 4];
            let groesse = u32::from_be_bytes(d[p + 4..p + 8].try_into().unwrap()) as usize;
            let inhalt = &d[p + 8..p + 8 + groesse];
            match id {
                b"COMM" => frames = u32::from_be_bytes(inhalt[2..6].try_into().unwrap()),
                b"SSND" => {
                    pcm = inhalt[8..]
                        .as_chunks::<2>()
                        .0
                        .iter()
                        .map(|b| i16::from_be_bytes([b[0], b[1]]))
                        .collect();
                }
                b"ID3 " => id3 = inhalt.to_vec(),
                _ => {}
            }
            p += 8 + groesse + groesse % 2;
        }
        (frames, pcm, id3)
    }

    fn nie_abbrechen() -> AtomicBool {
        AtomicBool::new(false)
    }

    #[test]
    fn testordner_sind_auch_bei_gleichem_namen_getrennt() {
        let erster = ordner("gleichzeitig");
        std::fs::write(erster.join("original"), b"unveraendert").unwrap();
        let zweiter = ordner("gleichzeitig");
        assert_ne!(erster.join("original"), zweiter.join("original"));
        assert_eq!(
            std::fs::read(erster.join("original")).unwrap(),
            b"unveraendert"
        );
        let pfad = erster.join("original");
        drop(erster);
        assert!(!pfad.exists());
        assert!(zweiter.exists());
    }

    #[test]
    fn wav_16bit_44khz_wird_bitgenau_aiff() {
        let o = ordner("bitgenau");
        let quelle = o.join("a.wav");
        let original = wav(&quelle, 44_100, 16, 0.5);
        wandle(
            &quelle,
            &o.join("a.aiff"),
            Ziel::Aiff,
            None,
            &nie_abbrechen(),
        )
        .unwrap();
        let (frames, pcm, _) = aiff_lesen(&o.join("a.aiff"));
        assert_eq!(frames as usize * 2, original.len());
        assert_eq!(pcm.len(), original.len());
        assert!(pcm.iter().zip(&original).all(|(a, b)| i32::from(*a) == *b));
    }

    #[test]
    fn wav_24bit_96khz_wird_44khz_aiff_mit_richtiger_laenge() {
        let o = ordner("umrechnen");
        let quelle = o.join("a.wav");
        wav(&quelle, 96_000, 24, 1.0);
        wandle(
            &quelle,
            &o.join("a.aiff"),
            Ziel::Aiff,
            None,
            &nie_abbrechen(),
        )
        .unwrap();
        let (frames, pcm, _) = aiff_lesen(&o.join("a.aiff"));
        assert_eq!(frames, 44_100);
        assert_eq!(pcm.len(), 44_100 * 2);
        // Sinus mit halber Aussteuerung: Spitzen bei ca. 16384
        let spitze = pcm.iter().map(|s| i32::from(*s).abs()).max().unwrap();
        assert!((15_500..17_500).contains(&spitze), "Spitze {spitze}");
    }

    #[test]
    fn wav_wird_mp3() {
        let o = ordner("mp3");
        let quelle = o.join("a.wav");
        wav(&quelle, 48_000, 16, 1.0);
        wandle(&quelle, &o.join("a.mp3"), Ziel::Mp3, None, &nie_abbrechen()).unwrap();
        let d = std::fs::read(o.join("a.mp3")).unwrap();
        // 1 s bei 320 kbit/s = ca. 40 kB, erster Frame beginnt mit dem Sync-Wort
        assert!(d.len() > 35_000, "nur {} Bytes", d.len());
        assert_eq!(d[0], 0xFF);
        assert_eq!(d[1] & 0xE0, 0xE0);

        // Die Datei muss auch dekodierbares Audio mit der erwarteten Tonhöhe enthalten.
        let mut mp3 = oeffne(&o.join("a.mp3")).unwrap();
        assert_eq!((mp3.rate, mp3.kanaele), (44_100, 2));
        let mut links = Vec::new();
        dekodiere(&mut mp3, &nie_abbrechen(), |puffer| {
            let mut samples = SampleBuffer::<f32>::new(puffer.capacity() as u64, *puffer.spec());
            samples.copy_interleaved_ref(puffer);
            links.extend(samples.samples().iter().step_by(2).copied());
            Ok(())
        })
        .unwrap();
        assert!(
            (44_000..=47_000).contains(&links.len()),
            "{} Frames",
            links.len()
        );
        let max = links.iter().copied().fold(0.0_f32, |a, b| a.max(b.abs()));
        assert!(max > 0.1, "MP3 ist stumm: {max}");
        // Encoder-Padding am Anfang und Ende nicht als Ton interpretieren.
        let mitte = &links[4_410..39_690];
        let nullstellen = mitte
            .windows(2)
            .filter(|w| w[0] <= 0.0 && w[1] > 0.0)
            .count();
        assert!(
            (345..=359).contains(&nullstellen),
            "{nullstellen} Nullstellen"
        );
    }

    #[test]
    fn cover_landet_im_id3_chunk() {
        let o = ordner("cover");
        let quelle = o.join("a.wav");
        wav(&quelle, 44_100, 16, 0.1);
        let bild = o.join("cover.jpg");
        std::fs::write(&bild, b"\xFF\xD8\xFF\xE0 kein echtes Bild").unwrap();
        wandle(
            &quelle,
            &o.join("a.aiff"),
            Ziel::Aiff,
            Some(&bild),
            &nie_abbrechen(),
        )
        .unwrap();
        let (_, _, id3) = aiff_lesen(&o.join("a.aiff"));
        assert_eq!(&id3[0..3], b"ID3");
        assert!(id3.windows(4).any(|w| w == b"APIC"));
    }

    #[test]
    fn cover_landet_im_mp3() {
        let o = ordner("cover-mp3");
        let quelle = o.join("a.wav");
        wav(&quelle, 44_100, 16, 0.1);
        let bild = o.join("cover.jpg");
        std::fs::write(&bild, b"\xFF\xD8\xFF\xE0 kein echtes Bild").unwrap();
        wandle(
            &quelle,
            &o.join("a.mp3"),
            Ziel::Mp3,
            Some(&bild),
            &nie_abbrechen(),
        )
        .unwrap();
        let d = std::fs::read(o.join("a.mp3")).unwrap();
        assert_eq!(&d[0..3], b"ID3");
        assert!(d.windows(4).any(|w| w == b"APIC"));
    }

    #[test]
    fn vorhandene_zieldatei_bleibt_unangetastet() {
        let o = ordner("vorhanden");
        let quelle = o.join("a.wav");
        wav(&quelle, 44_100, 16, 0.1);
        let ziel = o.join("a.aiff");
        std::fs::write(&ziel, b"nicht anfassen").unwrap();
        let fehler = wandle(&quelle, &ziel, Ziel::Aiff, None, &nie_abbrechen()).unwrap_err();
        assert_eq!(fehler, t().zieldatei_existiert);
        assert_eq!(std::fs::read(&ziel).unwrap(), b"nicht anfassen");
    }

    #[test]
    fn abbruch_loescht_halbfertige_datei() {
        let o = ordner("abbruch");
        let quelle = o.join("a.wav");
        wav(&quelle, 44_100, 16, 0.1);
        let ziel = o.join("a.aiff");
        let fehler = wandle(&quelle, &ziel, Ziel::Aiff, None, &AtomicBool::new(true)).unwrap_err();
        assert_eq!(fehler, ABGEBROCHEN);
        assert!(!ziel.exists());
    }

    #[test]
    fn kaputte_quelle_hinterlaesst_keine_datei() {
        let o = ordner("kaputt");
        let quelle = o.join("a.flac");
        std::fs::write(&quelle, b"kein Audio").unwrap();
        let ziel = o.join("a.aiff");
        assert!(wandle(&quelle, &ziel, Ziel::Aiff, None, &nie_abbrechen()).is_err());
        assert!(!ziel.exists());
    }

    #[test]
    fn sperrregel() {
        assert!(ist_gesperrt("mp3", Ziel::Aiff));
        assert!(ist_gesperrt("mp3", Ziel::Mp3));
        assert!(ist_gesperrt("aif", Ziel::Aiff));
        assert!(!ist_gesperrt("aiff", Ziel::Mp3));
        assert!(!ist_gesperrt("flac", Ziel::Aiff));
        assert!(ist_gesperrt("flac", Ziel::Flac));
        assert!(ist_gesperrt("mp3", Ziel::Flac));
        assert!(!ist_gesperrt("wav", Ziel::Flac));
        assert!(!ist_gesperrt("aiff", Ziel::Flac));
        assert!(!ist_gesperrt("flac", Ziel::Mp3));
    }

    // Eine Datei mit dem eigenen Dekodierpfad lesen: (Samplerate, Bits, Samples in Originalbreite)
    fn dekodiert(pfad: &Path) -> (usize, u32, Vec<i32>) {
        let mut q = oeffne(pfad).unwrap();
        let (rate, bits) = (q.rate, q.bits.unwrap());
        let mut samples = Vec::new();
        dekodiere(&mut q, &nie_abbrechen(), |puffer| {
            let mut sb = SampleBuffer::<i32>::new(puffer.capacity() as u64, *puffer.spec());
            sb.copy_interleaved_ref(puffer);
            samples.extend(sb.samples().iter().map(|s| s >> (32 - bits)));
            Ok(())
        })
        .unwrap();
        (rate, bits, samples)
    }

    // 16 Bit / 44,1 kHz bleibt sample-genau; alles andere wird 44,1 kHz / 16 Bit
    #[test]
    fn wav_wird_flac_mit_16_bit_44khz() {
        let o = ordner("flac-16-44");
        let quelle = o.join("a.wav");
        let original = wav(&quelle, 44_100, 16, 0.5);
        wandle(
            &quelle,
            &o.join("a.flac"),
            Ziel::Flac,
            None,
            &nie_abbrechen(),
        )
        .unwrap();
        let (rate, bits, samples) = dekodiert(&o.join("a.flac"));
        assert_eq!((rate, bits), (44_100, 16));
        assert_eq!(samples, original);

        for (rate, bits) in [(96_000, 24), (48_000, 24)] {
            let quelle = o.join(format!("b{rate}.wav"));
            wav(&quelle, rate, bits, 1.0);
            let ziel = o.join(format!("b{rate}.flac"));
            wandle(&quelle, &ziel, Ziel::Flac, None, &nie_abbrechen()).unwrap();
            let (rate_aus, bits_aus, samples) = dekodiert(&ziel);
            assert_eq!((rate_aus, bits_aus), (44_100, 16), "{rate} Hz, {bits} Bit");
            assert_eq!(samples.len() / 2, 44_100);
        }
    }

    // Kopf: STREAMINFO, danach Bild; der letzte Block trägt die Last-Markierung
    #[test]
    fn flac_mit_cover_hat_bildblock() {
        let o = ordner("flac-cover");
        let quelle = o.join("a.wav");
        let original = wav(&quelle, 44_100, 16, 0.2);
        let bild = o.join("cover.png");
        std::fs::write(&bild, b"\x89PNG kein echtes Bild").unwrap();
        wandle(
            &quelle,
            &o.join("a.flac"),
            Ziel::Flac,
            Some(&bild),
            &nie_abbrechen(),
        )
        .unwrap();
        let d = std::fs::read(o.join("a.flac")).unwrap();
        assert_eq!(&d[0..4], b"fLaC");
        let mut typen = Vec::new();
        let mut p = 4;
        loop {
            let (letzter, typ) = (d[p] & 0x80 != 0, d[p] & 0x7F);
            let laenge = u32::from_be_bytes([0, d[p + 1], d[p + 2], d[p + 3]]) as usize;
            typen.push(typ);
            p += 4 + laenge;
            if letzter {
                break;
            }
        }
        assert_eq!(typen, vec![0, 6]);
        assert!(d.windows(9).any(|w| w == b"image/png"));
        // Und die Datei bleibt abspielbar
        assert_eq!(dekodiert(&o.join("a.flac")).2, original);
    }

    // Eine leere Quelle ergibt eine gültige Datei ohne Frames (Kopf mit 0 Samples)
    #[test]
    fn leere_quelle_ergibt_gueltiges_flac() {
        let o = ordner("flac-leer");
        wav(&o.join("a.wav"), 44_100, 16, 0.0);
        wandle(
            &o.join("a.wav"),
            &o.join("a.flac"),
            Ziel::Flac,
            None,
            &nie_abbrechen(),
        )
        .unwrap();
        let d = std::fs::read(o.join("a.flac")).unwrap();
        assert_eq!(&d[0..4], b"fLaC");
        let gesamt = u64::from_be_bytes(d[18..26].try_into().unwrap()) & ((1 << 36) - 1);
        assert_eq!(gesamt, 0);
        assert_eq!(d.len(), 4 + 4 + 34); // nur Kopf
    }
}
