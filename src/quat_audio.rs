use eframe::egui;
use crate::kernel::{QuatKernel, QuatByte};
use std::time::{Duration, Instant};

pub struct QuatAudio {
    pub frequency: f32,       // Hz (npr. 440.0 za Ton A)
    pub duration_ms: u64,     // Trajanje piska u milisekundama
    pub is_playing: bool,     // Status zvuka
    pub active_until: Option<Instant>,
    pub wave_phase: f32,      // Za vizuelni osciloskop
    pub volume: f32,          // 0.0 do 1.0
    
    // Memorijske adrese u QuatKernel-u
    pub freq_reg_addr: usize,
    pub ctrl_reg_addr: usize,
}

impl QuatAudio {
    pub fn new() -> Self {
        Self {
            frequency: 440.0,
            duration_ms: 0,
            is_playing: false,
            active_until: None,
            wave_phase: 0.0,
            volume: 0.8,
            freq_reg_addr: 0x80, // RAM adresa za frekvenciju
            ctrl_reg_addr: 0x81, // RAM adresa za start/trajanje
        }
    }

    /// Triguruje zvuk sa datom frekvencijom (Hz) i trajanjem (ms)
    pub fn beep(&mut self, freq: f32, duration_ms: u64) {
        self.frequency = freq.clamp(20.0, 5000.0);
        self.duration_ms = duration_ms;
        self.is_playing = true;
        self.active_until = Some(Instant::now() + Duration::from_millis(duration_ms));
    }

    /// Glavna petlja — sinhronizuje audio sa memorijom Kernela i tajmerom
    pub fn tick(&mut self, kernel: &mut QuatKernel) {
        // 1. Provera hardverskog okidača iz RAM-a (I/O mapirani registri)
        let ram_freq = kernel.ram[self.freq_reg_addr].0 as f32 * 10.0;
        let ram_ctrl = kernel.ram[self.ctrl_reg_addr].0;

        // Ako je registar za kontrolu veći od 0 i trenutno ne svira
        if ram_ctrl > 0 && !self.is_playing {
            let duration = (ram_ctrl & 0x7F) as u64 * 20; // 1-127 -> 20ms do 2540ms
            if ram_freq > 0.0 {
                self.beep(ram_freq, duration);
            }
            // Resetujemo kontrolni registar da ne triguje beskonačno
            kernel.ram[self.ctrl_reg_addr] = QuatByte(0);
        }

        // 2. Provera da li je isteklo vreme piska
        if let Some(until) = self.active_until {
            if Instant::now() >= until {
                self.is_playing = false;
                self.active_until = None;
            }
        }

        // 3. Ažuriranje faze talasa za osciloskop
        if self.is_playing {
            self.wave_phase = (self.wave_phase + (self.frequency / 60.0)) % 1.0;
        } else {
            self.wave_phase = 0.0;
        }
    }

    /// UI Komponenta — Beeper Dashboard & Osciloskop
    pub fn ui(&mut self, ui: &mut egui::Ui, kernel: &mut QuatKernel) {
        ui.heading("🔊 QuatAudio — PC Speaker & Sound Synth");
        ui.separator();

        ui.horizontal(|ui| {
            // Statusna LED dioda za aktivnost zvučnika
            let led_color = if self.is_playing {
                egui::Color32::GREEN
            } else {
                egui::Color32::DARK_GRAY
            };
            
            ui.colored_label(led_color, if self.is_playing { "BEEPING [ON]" } else { "SILENT [OFF]" });
            ui.add_space(20.0);

            ui.label("Jačina zvuka:");
            ui.add(egui::Slider::new(&mut self.volume, 0.0..=1.0).text(""));
        });

        ui.add_space(10.0);

        // Kontrole za ručno testiranje zvučnika
        ui.group(|ui| {
            ui.label("🎛️ Ručne Kontrole Tona");
            ui.horizontal(|ui| {
                ui.label("Frekvencija (Hz):");
                ui.add(egui::Slider::new(&mut self.frequency, 100.0..=2000.0).logarithmic(true));
                
                if ui.button("🔔 Test Beep (200ms)").clicked() {
                    let freq = self.frequency;
                    self.beep(freq, 200);
                }
            });

            ui.horizontal(|ui| {
                if ui.button("🎶 Startup Jingle").clicked() {
                    // Triguruje kratku arpeđo melodiju preko I/O registara
                    kernel.ram[self.freq_reg_addr] = QuatByte(44); // 440Hz
                    kernel.ram[self.ctrl_reg_addr] = QuatByte(5);  // 100ms
                }

                if ui.button("⚠️ Error Sound").clicked() {
                    self.beep(150.0, 300);
                }
            });
        });

        ui.add_space(10.0);

        // 📺 Vizuelni Osciloskop (Kvadratni talas / Square Wave)
        ui.label("📊 Osciloskop Izlaznog Signala (1-bit Pulse):");
        
        let (response, painter) = ui.allocate_painter(
            egui::vec2(ui.available_width(), 100.0),
            egui::Sense::hover(),
        );

        let rect = response.rect;
        painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(10, 15, 20));
        painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0, egui::Color32::GRAY));

        let center_y = rect.center().y;
        let amplitude = 35.0;
        let mut points = Vec::new();
        let num_samples = 100;

        for i in 0..num_samples {
            let x = rect.min.x + (i as f32 / num_samples as f32) * rect.width();
            
            let y = if self.is_playing {
                // Generisanje pravog kvadratnog talasa za vizuelni prikaz
                let phase = (self.wave_phase + (i as f32 * 0.05)) % 1.0;
                if phase > 0.5 {
                    center_y - amplitude
                } else {
                    center_y + amplitude
                }
            } else {
                center_y // Ravna linija kad zvučnik ćuti
            };

            points.push(egui::pos2(x, y));
        }

        // Crtanje osciloskopske linije
        let wave_color = if self.is_playing { egui::Color32::LIGHT_GREEN } else { egui::Color32::DARK_GREEN };
        painter.add(egui::Shape::line(points, egui::Stroke::new(2.0, wave_color)));
    }
}