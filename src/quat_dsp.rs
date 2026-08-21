use eframe::egui;
use rodio::{OutputStream, Sink, Source};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use crate::kernel::{QuatByte, QuatKernel};

const SAMPLE_RATE: u32 = 44100;

// ============================================================================
// 1. TALASNI OBLICI I ADSR ENVELOPE ENGINE
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Waveform {
    Square,   // Kvat 0: Pulsni talas sa PWM-om (Kanal 1 - Melodija)
    Triangle, // Kvat 1: Trouglasti talas (Kanal 2 - Bas)
    Sawtooth, // Kvat 2: Testerasti talas (Kanal 3 - Pratnja / Lead)
    Noise,    // Kvat 3: Chiptune LFSR Buka (Kanal 4 - Doboš / Činjele)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvelopeStage {
    Off,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Debug, Clone)]
pub struct ADSR {
    pub attack: f32,  // Trajanje u sekundama
    pub decay: f32,   // Trajanje u sekundama
    pub sustain: f32, // Nivo (0.0 - 1.0)
    pub release: f32, // Trajanje u sekundama
}

impl Default for ADSR {
    fn default() -> Self {
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.7,
            release: 0.2,
        }
    }
}

// ============================================================================
// 2. KANAL SINTETIZATORA (1 OD 4 KVAT-KANALA)
// ============================================================================

#[derive(Debug, Clone)]
pub struct DspChannel {
    pub enabled: bool,
    pub waveform: Waveform,
    pub frequency: f32,
    pub volume: f32,      // 0.0 - 1.0
    pub pwm_duty: f32,    // 0.1 - 0.9 za Square wave
    pub adsr: ADSR,
    
    // Unutrašnje stanje generisanja zvuka
    phase: f32,
    env_stage: EnvelopeStage,
    env_level: f32,
    env_time: f32,
    lfsr_state: u16, // Random generator za Noise
}

impl DspChannel {
    pub fn new(waveform: Waveform) -> Self {
        Self {
            enabled: true,
            waveform,
            frequency: 440.0,
            volume: 0.5,
            pwm_duty: 0.5,
            adsr: ADSR::default(),
            phase: 0.0,
            env_stage: EnvelopeStage::Off,
            env_level: 0.0,
            env_time: 0.0,
            lfsr_state: 0xACE1,
        }
    }

    pub fn trigger_note(&mut self, freq: f32) {
        self.frequency = freq;
        self.env_stage = EnvelopeStage::Attack;
        self.env_time = 0.0;
        self.env_level = 0.0;
    }

    pub fn release_note(&mut self) {
        if self.env_stage != EnvelopeStage::Off {
            self.env_stage = EnvelopeStage::Release;
            self.env_time = 0.0;
        }
    }

    /// Izračunava sledeći audio uzorak (sample) za dati kanal
    pub fn next_sample(&mut self, dt: f32) -> f32 {
        if !self.enabled || self.env_stage == EnvelopeStage::Off || self.frequency <= 0.0 {
            return 0.0;
        }

        // --- ADSR Izračunavanje ---
        self.env_time += dt;
        match self.env_stage {
            EnvelopeStage::Attack => {
                if self.adsr.attack > 0.0 {
                    self.env_level = (self.env_time / self.adsr.attack).min(1.0);
                } else {
                    self.env_level = 1.0;
                }
                if self.env_time >= self.adsr.attack {
                    self.env_stage = EnvelopeStage::Decay;
                    self.env_time = 0.0;
                }
            }
            EnvelopeStage::Decay => {
                if self.adsr.decay > 0.0 {
                    let progress = self.env_time / self.adsr.decay;
                    self.env_level = 1.0 - progress * (1.0 - self.adsr.sustain);
                } else {
                    self.env_level = self.adsr.sustain;
                }
                if self.env_time >= self.adsr.decay {
                    self.env_stage = EnvelopeStage::Sustain;
                    self.env_level = self.adsr.sustain;
                }
            }
            EnvelopeStage::Sustain => {
                self.env_level = self.adsr.sustain;
            }
            EnvelopeStage::Release => {
                if self.adsr.release > 0.0 {
                    let start_level = self.env_level;
                    self.env_level = start_level * (1.0 - (self.env_time / self.adsr.release)).max(0.0);
                } else {
                    self.env_level = 0.0;
                }
                if self.env_time >= self.adsr.release || self.env_level <= 0.001 {
                    self.env_stage = EnvelopeStage::Off;
                    self.env_level = 0.0;
                }
            }
            EnvelopeStage::Off => self.env_level = 0.0,
        }

        // --- Generator Talasa ---
        self.phase += self.frequency * dt;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        let raw_sample = match self.waveform {
            Waveform::Square => {
                if self.phase < self.pwm_duty { 1.0 } else { -1.0 }
            }
            Waveform::Triangle => {
                if self.phase < 0.5 {
                    4.0 * self.phase - 1.0
                } else {
                    3.0 - 4.0 * self.phase
                }
            }
            Waveform::Sawtooth => 2.0 * self.phase - 1.0,
            Waveform::Noise => {
                // Retro 16-bit LFSR Generator Buke
                if self.phase < dt * self.frequency {
                    let bit = ((self.lfsr_state >> 0) ^ (self.lfsr_state >> 2) ^ (self.lfsr_state >> 3) ^ (self.lfsr_state >> 5)) & 1;
                    self.lfsr_state = (self.lfsr_state >> 1) | (bit << 15);
                }
                if (self.lfsr_state & 1) == 1 { 1.0 } else { -1.0 }
            }
        };

        raw_sample * self.volume * self.env_level
    }
}

// ============================================================================
// 3. TRACKER SEKVENCIJER & SINTETIZATOR CORE (ZA DELJENJE MEĐU THREAD-OVIMA)
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub struct TrackerNote {
    pub note_index: u8, // 0 = Prazno, 1..=48 (C-3 do B-6)
    pub volume: u8,     // 0..=3 (Preslikano u Kvat)
}

pub struct DspCore {
    pub channels: [DspChannel; 4],
    pub master_volume: f32,
    pub is_playing: bool,
    pub bpm: u16,
    pub current_step: usize,
    pub pattern: Vec<[TrackerNote; 4]>, // 32 Koraka x 4 Kvatanalna Kanala
    
    step_timer: f32,
    pub oscilloscope_buffer: Vec<f32>,
}

impl DspCore {
    pub fn new() -> Self {
        let mut pattern = vec![[TrackerNote { note_index: 0, volume: 3 }; 4]; 32];
        
        // Unosimo demo melodiju u Tracker
        let demo_notes_ch0 = [13, 0, 13, 0, 16, 0, 13, 0, 18, 0, 13, 0, 16, 0, 11, 0];
        for (i, &note) in demo_notes_ch0.iter().enumerate() {
            pattern[i * 2][0] = TrackerNote { note_index: note, volume: 3 };
        }
        
        // Bas linija na Kanalu 1
        for i in (0..32).step_by(4) {
            pattern[i][1] = TrackerNote { note_index: 1, volume: 2 };
        }

        Self {
            channels: [
                DspChannel::new(Waveform::Square),
                DspChannel::new(Waveform::Triangle),
                DspChannel::new(Waveform::Sawtooth),
                DspChannel::new(Waveform::Noise),
            ],
            master_volume: 0.3,
            is_playing: false,
            bpm: 125,
            current_step: 0,
            pattern,
            step_timer: 0.0,
            oscilloscope_buffer: vec![0.0; 128],
        }
    }

    /// Konvertuje indeks note u frekvenciju (Hz)
    pub fn note_to_freq(note: u8) -> f32 {
        if note == 0 { return 0.0; }
        // C-3 počinje od ~130.81 Hz
        130.81 * (2.0f32).powf((note as f32 - 1.0) / 12.0)
    }

    /// Izvršava jedan korak sekvencijera
    pub fn advance_step(&mut self) {
        let step_notes = self.pattern[self.current_step];
        for (ch_idx, note_info) in step_notes.iter().enumerate() {
            if note_info.note_index > 0 {
                let freq = Self::note_to_freq(note_info.note_index);
                self.channels[ch_idx].volume = (note_info.volume as f32) / 3.0;
                self.channels[ch_idx].trigger_note(freq);
            }
        }
    }

    /// Generiše jedan sledeći zbirni audio sample
    pub fn mix_next_sample(&mut self, dt: f32) -> f32 {
        if self.is_playing {
            let seconds_per_step = 60.0 / (self.bpm as f32 * 4.0);
            self.step_timer += dt;
            if self.step_timer >= seconds_per_step {
                self.step_timer -= seconds_per_step;
                self.current_step = (self.current_step + 1) % 32;
                self.advance_step();
            }
        }

        let mut mix = 0.0;
        for ch in self.channels.iter_mut() {
            mix += ch.next_sample(dt);
        }
        
        let final_sample = mix * self.master_volume;

        // Punjenje bafera za egui Osciloskop
        if self.oscilloscope_buffer.len() >= 128 {
            self.oscilloscope_buffer.remove(0);
        }
        self.oscilloscope_buffer.push(final_sample);

        final_sample
    }
}

// ============================================================================
// 4. RODIO CUSTOM AUDIO SOURCE (RODIO 0.19 COMPATIBLE)
// ============================================================================

pub struct ChiptuneSource {
    core: Arc<Mutex<DspCore>>,
}

impl Iterator for ChiptuneSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let dt = 1.0 / SAMPLE_RATE as f32;
        if let Ok(mut core) = self.core.lock() {
            Some(core.mix_next_sample(dt))
        } else {
            Some(0.0)
        }
    }
}

impl Source for ChiptuneSource {
    fn current_frame_len(&self) -> Option<usize> { None }
    fn channels(&self) -> u16 { 1 }
    fn sample_rate(&self) -> u32 { SAMPLE_RATE }
    fn total_duration(&self) -> Option<Duration> { None }
}

// ============================================================================
// 5. GLAVNA STRUCTURA ZA QuatOS UI & SISTEM
// ============================================================================

pub struct QuatDSP {
    pub core: Arc<Mutex<DspCore>>,
    _stream: Option<OutputStream>,
    _sink: Option<Sink>,
    selected_channel: usize,
    status_msg: String,
}

impl QuatDSP {
    pub fn new() -> Self {
        let core = Arc::new(Mutex::new(DspCore::new()));

        // Inicijalizacija Rodio Audio Zvučnog Izlaza
        let (stream, sink) = match OutputStream::try_default() {
            Ok((st, handle)) => {
                let sink = Sink::try_new(&handle).ok();
                if let Some(ref s) = sink {
                    let source = ChiptuneSource { core: Arc::clone(&core) };
                    s.append(source);
                    s.play();
                }
                (Some(st), sink)
            }
            Err(_) => (None, None),
        };

        Self {
            core,
            _stream: stream,
            _sink: sink,
            selected_channel: 0,
            status_msg: "QuatDSP 4-Kanalni Zvučni Čip Spreman.".to_string(),
        }
    }

    /// Okida direktan zvučni signal na određenoj frekvenciji (Za DOS BEEP komandu)
    pub fn beep(&mut self, freq: f32, duration_ms: u64) {
        if let Ok(mut core) = self.core.lock() {
            core.channels[0].trigger_note(freq);
            self.status_msg = format!("BEEP: {} Hz ({} ms)", freq, duration_ms);
        }
    }

    /// Prikaz UI okruženja unutar egui
    pub fn ui(&mut self, ui: &mut egui::Ui, _kernel: &mut QuatKernel) {
        ui.heading("🔊 QuatDSP 4-Kanalni Zvučni Sintetizator & Tracker");
        ui.separator();

        let mut core = self.core.lock().unwrap();

        // Top Control Panel
        ui.horizontal(|ui| {
            if ui.button(if core.is_playing { "⏸ Pauza" } else { "▶ Pusti Tracker" }).clicked() {
                core.is_playing = !core.is_playing;
            }
            if ui.button("⏹ Stop").clicked() {
                core.is_playing = false;
                core.current_step = 0;
            }

            ui.add_space(20.0);
            ui.label("BPM:");
            ui.add(egui::DragValue::new(&mut core.bpm).clamp_range(40..=240));

            ui.add_space(20.0);
            ui.label("Master Vol:");
            ui.add(egui::Slider::new(&mut core.master_volume, 0.0..=1.0));
        });

        ui.separator();

        // OSILOSKOP VIZUELIZACIJA TALASA
        ui.label("📊 Osciloskop Zvučnog Izlaza (Real-time):");
        let (response, painter) = ui.allocate_painter(egui::vec2(ui.available_width(), 60.0), egui::Sense::hover());
        let rect = response.rect;
        painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(15, 20, 15));

        let points: Vec<egui::Pos2> = core.oscilloscope_buffer.iter().enumerate().map(|(i, &val)| {
            let x = rect.left() + (i as f32 / 128.0) * rect.width();
            let y = rect.center().y - val * (rect.height() * 0.4);
            egui::pos2(x, y)
        }).collect();

        for window in points.windows(2) {
            painter.line_segment([window[0], window[1]], egui::Stroke::new(1.5, egui::Color32::GREEN));
        }

        ui.add_space(10.0);

        // KANALI & ADSR EDITOVANJE
        ui.columns(2, |cols| {
            cols[0].vertical(|ui| {
                ui.label("🎛 Odabir Kanala (1 Kvat = 1 Kanal):");
                ui.horizontal(|ui| {
                    let names = ["Ch 0 (Square)", "Ch 1 (Triangle)", "Ch 2 (Saw)", "Ch 3 (Noise)"];
                    for i in 0..4 {
                        if ui.selectable_label(self.selected_channel == i, names[i]).clicked() {
                            self.selected_channel = i;
                        }
                    }
                });

                let ch = &mut core.channels[self.selected_channel];
                ui.checkbox(&mut ch.enabled, "Kanal Aktivan");
                ui.add(egui::Slider::new(&mut ch.volume, 0.0..=1.0).text("Glasnoća"));

                if ch.waveform == Waveform::Square {
                    ui.add(egui::Slider::new(&mut ch.pwm_duty, 0.1..=0.9).text("PWM Duty Cycle"));
                }

                ui.group(|ui| {
                    ui.label("📐 ADSR Envelope Podešavanja:");
                    ui.add(egui::Slider::new(&mut ch.adsr.attack, 0.001..=0.5).text("Attack (s)"));
                    ui.add(egui::Slider::new(&mut ch.adsr.decay, 0.01..=1.0).text("Decay (s)"));
                    ui.add(egui::Slider::new(&mut ch.adsr.sustain, 0.0..=1.0).text("Sustain Nivo"));
                    ui.add(egui::Slider::new(&mut ch.adsr.release, 0.01..=2.0).text("Release (s)"));
                });
            });

            // TRACKER MATRIX GRID
            cols[1].vertical(|ui| {
                ui.label("🎼 QuatTracker Rešetka (32 Step Pattern):");
                egui::ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
                    egui::Grid::new("tracker_grid").striped(true).show(ui, |ui| {
                        ui.label("Step");
                        ui.label("Ch 0");
                        ui.label("Ch 1");
                        ui.label("Ch 2");
                        ui.label("Ch 3");
                        ui.end_row();

                        for step_idx in 0..32 {
                            let is_current = core.current_step == step_idx && core.is_playing;
                            let label_text = if is_current { format!("▶ {:02}", step_idx) } else { format!("  {:02}", step_idx) };

                            ui.label(egui::RichText::new(label_text).monospace().color(
                                if is_current { egui::Color32::YELLOW } else { egui::Color32::GRAY }
                            ));

                            for ch_idx in 0..4 {
                                let note_val = &mut core.pattern[step_idx][ch_idx].note_index;
                                ui.add(egui::DragValue::new(note_val).clamp_range(0..=48));
                            }
                            ui.end_row();
                        }
                    });
                });
            });
        });

        ui.separator();
        ui.label(egui::RichText::new(&self.status_msg).monospace().color(egui::Color32::LIGHT_BLUE));
    }
}