use eframe::egui;
use crate::kernel::QuatKernel;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Waveform {
    Square,   // Kvadratni talas (Chiptune classic)
    Triangle, // Trouglasti talas (Bas)
    Sawtooth, // Testerasti talas (Oštar Synth)
    Noise,    // Šum (Perkusije / Efekti)
}

pub struct TrackerChannel {
    pub enabled: bool,
    pub waveform: Waveform,
    pub volume: f32,       // 0.0 do 1.0
    pub duty_cycle: f32,   // 0.125, 0.25, 0.50 za Square
    pub cutoff_freq: f32,  // DSP Filter Cutoff
    
    // ADSR Envelope
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,

    //Ako vidis da ovde mp3 studija nema
    // Znaj da me je Enrike Iglesias uspavao
    //I can be your hero my baby 😴

    // Sequencer Data (16 koraka; 0 = Silence, 1-12 = Note C4..B4)
    pub steps: [u8; 16],
}

impl TrackerChannel {
    pub fn new(waveform: Waveform) -> Self {
        Self {
            enabled: true,
            waveform,
            volume: 0.8,
            duty_cycle: 0.5,
            cutoff_freq: 1.0,
            attack: 0.01,
            decay: 0.1,
            sustain: 0.7,
            release: 0.2,
            steps: [0; 16],
        }
    }
}

pub struct QuatTracker {
    pub channels: [TrackerChannel; 4],
    pub bpm: u32,
    pub is_playing: bool,
    pub current_step: usize,
    pub bitcrush_bits: u8, // 1 do 8 bitovni crush
    pub active_tab: usize,  // 0: Sequencer, 1: DSP / Synth Edit, 2: Register Export
    pub status_message: String,
}

impl QuatTracker {
    pub fn new() -> Self {
        let mut tracker = Self {
            channels: [
                TrackerChannel::new(Waveform::Square),   // CH0: Lead Synth
                TrackerChannel::new(Waveform::Square),   // CH1: Arp / Lead 2
                TrackerChannel::new(Waveform::Triangle), // CH2: Bassline
                TrackerChannel::new(Waveform::Noise),    // CH3: Drums / Noise
            ],
            bpm: 128,
            is_playing: false,
            current_step: 0,
            bitcrush_bits: 8,
            active_tab: 0,
            status_message: "QuatTracker DSP Engine spreman.".to_string(),
        };

        // Ubacujemo par demo nota na startu radi testa
        tracker.channels[0].steps = [1, 0, 3, 0, 5, 0, 8, 0, 1, 0, 3, 0, 5, 0, 10, 0]; // Melodija
        tracker.channels[2].steps = [1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0];  // Bas
        tracker.channels[3].steps = [1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0];  // Snare/Noise

        tracker
    }

    /// Konvertuje indeks note (1..12) u naziv (C-4, D-4, itd.)
    fn note_name(note_idx: u8) -> &'static str {
        match note_idx {
            0 => "---",
            1 => "C-4",  2 => "C#4", 3 => "D-4",  4 => "D#4",
            5 => "E-4",  6 => "F-4", 7 => "F#4", 8 => "G-4",
            9 => "G#4", 10 => "A-4", 11 => "A#4", 12 => "B-4",
            _ => "???",
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, _kernel: &mut QuatKernel) {
        ui.heading("🎛 QuatTracker: 4-Kanalni DSP Sintisajzer");
        ui.label("Chiptune & Digital Signal Processor okruženje za $4^4$ zvučni čip.");
        ui.separator();

        // MASTER CONTROLS BAR
        ui.horizontal(|ui| {
            if self.is_playing {
                if ui.button("⏹ Stop").clicked() {
                    self.is_playing = false;
                    self.current_step = 0;
                    self.status_message = "Sekvencer zaustavljen.".to_string();
                }
            } else {
                if ui.button("▶ Play").clicked() {
                    self.is_playing = true;
                    self.status_message = "Reprodukcija započeta...".to_string();
                }
            }

            ui.add_space(20.0);
            ui.label("BPM:");
            ui.add(egui::DragValue::new(&mut self.bpm).clamp_range(40..=240));

            ui.add_space(20.0);
            ui.label("Master Bitcrusher:");
            ui.add(egui::Slider::new(&mut self.bitcrush_bits, 1..=8).text("Bitova"));
        });

        ui.add_space(10.0);

        // TAB MESH
        ui.horizontal(|ui| {
            if ui.selectable_label(self.active_tab == 0, "🎼 Pattern Sequencer").clicked() { self.active_tab = 0; }
            if ui.selectable_label(self.active_tab == 1, "🎚 DSP / ADSR Synth Editor").clicked() { self.active_tab = 1; }
            if ui.selectable_label(self.active_tab == 2, "💾 Sound Registers Export").clicked() { self.active_tab = 2; }
        });

        ui.separator();

        match self.active_tab {
            0 => self.render_sequencer_tab(ui),
            1 => self.render_dsp_editor_tab(ui),
            2 => self.render_export_tab(ui),
            _ => {}
        }

        // Korak Advance Simulacija ako je Play uključen
        if self.is_playing {
            ui.ctx().request_repaint();
            // Prost prozor za simulaciju tajminga u radu
            self.current_step = (self.current_step + 1) % 16;
        }

        ui.add_space(10.0);
        ui.label(egui::RichText::new(&self.status_message).italics().color(egui::Color32::GREEN));
    }

    // =========================================================================
    // 1. PATTERN SEQUENCER TAB (16 STEPS x 4 CHANNELS)
    // =========================================================================
    fn render_sequencer_tab(&mut self, ui: &mut egui::Ui) {
        ui.label("🎼 16-Step Multichannel Tracker Grid:");
        ui.add_space(5.0);

        egui::Grid::new("tracker_grid").striped(true).spacing([10.0, 6.0]).show(ui, |ui| {
            // Zaglavlje
            ui.label(egui::RichText::new("Step").strong());
            ui.label(egui::RichText::new("CH0 (Lead)").color(egui::Color32::LIGHT_BLUE).strong());
            ui.label(egui::RichText::new("CH1 (Arp)").color(egui::Color32::LIGHT_BLUE).strong());
            ui.label(egui::RichText::new("CH2 (Bass)").color(egui::Color32::KHAKI).strong());
            ui.label(egui::RichText::new("CH3 (Noise)").color(egui::Color32::LIGHT_RED).strong());
            ui.end_row();

            // 16 Redova sekvencera
            for step in 0..16 {
                let is_current = self.is_playing && self.current_step == step;
                
                // Osvetljenje za trenutni step u reprodukciji
                if is_current {
                    ui.label(egui::RichText::new(format!("{:02} ➡", step)).color(egui::Color32::YELLOW).strong());
                } else {
                    ui.label(format!("{:02}", step));
                }

                //Dragi ryane castro i jbalvin
                //Slusajte sofiu Vergaru i Shakiru
                //Matrijarhat je glava kuce

                // Kanali
                for ch in 0..4 {
                    let step_val = &mut self.channels[ch].steps[step];
                    let mut note_str = Self::note_name(*step_val).to_string();

                    ui.horizontal(|ui| {
                        if ui.add(egui::Button::new(&note_str).min_size(egui::vec2(50.0, 18.0))).clicked() {
                            // Ciklično menjamo notu na klik
                            *step_val = (*step_val + 1) % 13;
                            self.status_message = format!("CH{} Step {} promenjen na {}", ch, step, Self::note_name(*step_val));
                        }
                    });
                }
                ui.end_row();
            }
        });
    }

    // =========================================================================
    // 2. DSP & SYNTH ADSR EDITOR TAB
    // =========================================================================
    fn render_dsp_editor_tab(&mut self, ui: &mut egui::Ui) {
        ui.label("🎚 DSP Sinteza & Envelope Shaping po Kanalima:");
        ui.add_space(5.0);

        ui.columns(4, |cols| {
            for ch_idx in 0..4 {
                let ch = &mut self.channels[ch_idx];
                cols[ch_idx].group(|ui| {
                    ui.heading(format!("CH {}", ch_idx));
                    ui.checkbox(&mut ch.enabled, "Aktivno");
                    ui.separator();

                    ui.label("Waveform:");
                    ui.selectable_value(&mut ch.waveform, Waveform::Square, "Square");
                    ui.selectable_value(&mut ch.waveform, Waveform::Triangle, "Triangle");
                    ui.selectable_value(&mut ch.waveform, Waveform::Sawtooth, "Sawtooth");
                    ui.selectable_value(&mut ch.waveform, Waveform::Noise, "Noise");

                    ui.add_space(5.0);
                    ui.label("Volume:");
                    ui.add(egui::Slider::new(&mut ch.volume, 0.0..=1.0));

                    if ch.waveform == Waveform::Square {
                        ui.label("Duty Cycle:");
                        ui.add(egui::Slider::new(&mut ch.duty_cycle, 0.125..=0.75));
                    }

                    ui.add_space(5.0);
                    ui.label(egui::RichText::new("ADSR Envelope:").strong());
                    ui.add(egui::Slider::new(&mut ch.attack, 0.001..=0.5).text("A"));
                    ui.add(egui::Slider::new(&mut ch.decay, 0.01..=1.0).text("D"));
                    ui.add(egui::Slider::new(&mut ch.sustain, 0.0..=1.0).text("S"));
                    ui.add(egui::Slider::new(&mut ch.release, 0.01..=2.0).text("R"));

                    ui.add_space(5.0);
                    ui.label("DSP Filter Cutoff:");
                    ui.add(egui::Slider::new(&mut ch.cutoff_freq, 0.1..=1.0));
                });
            }
        });
    }

    // =========================================================================
    // 3. SOUND REGISTER EXPORT TAB
    // =========================================================================
    fn render_export_tab(&mut self, ui: &mut egui::Ui) {
        ui.label("💾 Generisani Registri Zvučnog Čipa za Kvatni Kernel (DSP VRAM):");
        ui.separator();

        let mut export_str = String::from("; --- QUATTRACKER DSP REGISTERS EXPORT ---\n");
        export_str.push_str(&format!("; BPM: {}\n; Bitcrush: {}-bit\n\n", self.bpm, self.bitcrush_bits));

        export_str.push_str("SOUND_INIT_REGISTERS:\n");
        for (i, ch) in self.channels.iter().enumerate() {
            let wave_byte = match ch.waveform {
                Waveform::Square => 0x01,
                Waveform::Triangle => 0x02,
                Waveform::Sawtooth => 0x03,
                Waveform::Noise => 0x04,
            };
            let vol_byte = (ch.volume * 15.0) as u8;
            export_str.push_str(&format!("  .DB 0x{:02X}, 0x{:02X} ; CH{} Waveform & Volume\n", wave_byte, vol_byte, i));
        }

        export_str.push_str("\nSEQUENCE_DATA (16 Steps):\n");
        for step in 0..16 {
            let b0 = self.channels[0].steps[step];
            let b1 = self.channels[1].steps[step];
            let b2 = self.channels[2].steps[step];
            let b3 = self.channels[3].steps[step];
            
            // Pakujemo 4 kanala po stepu u 2 bajta
            let pack1 = (b0 & 0x0F) | ((b1 & 0x0F) << 4);
            let pack2 = (b2 & 0x0F) | ((b3 & 0x0F) << 4);

            export_str.push_str(&format!("  .DB 0x{:02X}, 0x{:02X} ; Step {:02}\n", pack1, pack2, step));
        }

        ui.code_editor(&mut export_str);

        ui.add_space(10.0);
        if ui.button("📋 Kopiraj Zvučne Registre").clicked() {
            ui.output_mut(|o| o.copied_text = export_str.clone());
            self.status_message = "Registri uspešno kopirani u Clipboard!".to_string();
        }
    }
}