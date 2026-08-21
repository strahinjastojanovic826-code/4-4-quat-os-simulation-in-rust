use eframe::egui;
use crate::kernel::QuatKernel;
use std::fs::File;
use std::io::{Read, Write};

pub struct QuatTAPE {
    pub current_filename: String,
    pub tape_status: String,
    pub is_spinning: bool,
    pub spin_frame: usize,
}

impl QuatTAPE {
    pub fn new() -> Self {
        Self {
            current_filename: "my_program.quat".to_string(),
            tape_status: "Kasetofon u pripravnosti (READY).".to_string(),
            is_spinning: false,
            spin_frame: 0,
        }
    }

    /// Snima kompletan RAM (256 bajtova) i VRAM (64 bajta) u fajl
    pub fn save_to_tape(&mut self, kernel: &QuatKernel) {
        let mut data = Vec::new();
        // Dodajemo magični zaglavlje (Header): 'Q', 'U', 'A', 'T'
        data.extend_from_slice(b"QUAT");
        // Dampujemo RAM i VRAM
        data.extend(kernel.ram.iter().map(|b| b.0));
        data.extend(kernel.ram.iter().map(|b| b.0));

        match File::create(&self.current_filename) {
            Ok(mut file) => {
                if file.write_all(&data).is_ok() {
                    self.tape_status = format!("✅ Uspešno snimljeno na traku: '{}' ({} bajtova)", self.current_filename, data.len());
                    self.is_spinning = true;
                } else {
                    self.tape_status = "❌ Greška pri pisanju na disk!".to_string();
                }
            }
            Err(_) => self.tape_status = "❌ Ne mogu da kreiram fajl!".to_string(),
        }
    }

    /// Učitava RAM i VRAM iz fajla u Kernel
    pub fn load_from_tape(&mut self, kernel: &mut QuatKernel) {
        match File::open(&self.current_filename) {
            Ok(mut file) => {
                let mut buffer = Vec::new();
                if file.read_to_end(&mut buffer).is_ok() {
                    // Provera zaglavlja
                    if buffer.len() >= 4 + 256 + 64 && &buffer[0..4] == b"QUAT" {
                        for (i, &byte) in buffer[4..260].iter().enumerate() {
                         kernel.ram[i] = crate::kernel::QuatByte(byte);
                        }
                        for (i, &byte) in buffer[260..324].iter().enumerate() {
                         kernel.vram[i] = crate::kernel::QuatByte(byte);
                        }
                        self.tape_status = format!("✅ Uspešno učitano sa trake: '{}'", self.current_filename);
                        self.is_spinning = true;
                    } else {
                        self.tape_status = "❌ Neispravan format Quat trake ili oštećen fajl!".to_string();
                    }
                }
            }
            Err(_) => self.tape_status = format!("❌ Fajl '{}' nije pronađen!", self.current_filename),
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, kernel: &mut QuatKernel) {
        ui.heading("📟 QuatTAPE: Magnetic Tape & Disk I/O Storage");
        ui.label("Emulacija perzistentne memorije za snimanje i učitavanje kompletnog stanja mašine.");
        ui.separator();

        // RETRO ANIMACIJA TRAKE
        ui.horizontal(|ui| {
            let reel_icons = ["◐", "◓", "◑", "◒"];
            if self.is_spinning {
                self.spin_frame = (self.spin_frame + 1) % 4;
                ui.ctx().request_repaint();
            }
            let icon = if self.is_spinning { reel_icons[self.spin_frame] } else { "⊙" };

            ui.label(egui::RichText::new(format!(" [ {} ]  📼  [ {} ] ", icon, icon)).size(24.0).color(egui::Color32::YELLOW));
            ui.label(egui::RichText::new("C-60 QUAT MAGNETIC TAPE").strong().monospace());
        });

        ui.add_space(10.0);

        // KONTROLE ZA FAJL
        ui.group(|ui| {
            ui.label(egui::RichText::new("📁 Naziv Fajla / Trake:").strong());
            ui.text_edit_singleline(&mut self.current_filename);

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("💾 SAVE (Snimi Stanje RAM/VRAM)").clicked() {
                    self.save_to_tape(kernel);
                }
                if ui.button("📂 LOAD (Učitaj Stanje u RAM/VRAM)").clicked() {
                    self.load_from_tape(kernel);
                }
                if ui.button("⏹ STOP REEL").clicked() {
                    self.is_spinning = false;
                }
            });
        });

        ui.add_space(10.0);

        // STATUS
        ui.group(|ui| {
            ui.label("Status Kasetofona:");
            ui.label(egui::RichText::new(&self.tape_status).monospace().color(egui::Color32::LIGHT_GREEN));
        });

        ui.add_space(10.0);
        ui.collapsing("🔍 Pregled Zaglavlja & Memorijskog Dump-a", |ui| {
            ui.label(format!("RAM Prva 4 kvata: {:?}", &kernel.ram[0..4]));
            ui.label(format!("VRAM Prva 4 kvata: {:?}", &kernel.vram[0..4]));
        });
    }
}