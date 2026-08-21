use eframe::egui;
use crate::kernel::{QuatKernel, QuatByte};
use std::time::{Duration, Instant};

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum BootStage {
    PowerOff,
    PostCpuCheck,
    RamDiagnostic,
    CrtSplashAnimation,
    MountQuatFS,
    BootComplete,
}

pub struct QuatBIOS {
    pub stage: BootStage,
    pub progress: f32,
    pub log: Vec<String>,
    pub last_tick: Option<Instant>,
    pub frame_step: usize,
    pub auto_boot: bool,
}

impl QuatBIOS {
    pub fn new() -> Self {
        Self {
            stage: BootStage::PowerOff,
            progress: 0.0,
            log: vec!["SYSTEM READY FOR POWER-ON.".to_string()],
            last_tick: None,
            frame_step: 0,
            auto_boot: true,
        }
    }

    pub fn power_on(&mut self, kernel: &mut QuatKernel) {
        self.stage = BootStage::PostCpuCheck;
        self.progress = 0.0;
        self.frame_step = 0;
        self.log.clear();
        self.log.push("⚡ QUAT-BIOS v1.0 (c) 2026 QuatOS Inc.".to_string());
        self.log.push("----------------------------------------".to_string());
        self.last_tick = Some(Instant::now());

        // Reset kerna pri paljenju
        kernel.pc = 0;
        kernel.reg_a = QuatByte(0);
        kernel.reg_b = QuatByte(0);
        kernel.is_running = false;
    }

    pub fn update(&mut self, kernel: &mut QuatKernel) {
        if self.stage == BootStage::PowerOff || self.stage == BootStage::BootComplete {
            return;
        }

        let now = Instant::now();
        let elapsed = self.last_tick.map_or(0, |t| now.duration_since(t).as_millis());

        // Taktovanje boot sekvence na svakih ~120ms radi vizuelnog efekta
        if elapsed > 120 {
            self.last_tick = Some(now);

            match self.stage {
                BootStage::PostCpuCheck => {
                    self.progress = 0.2;
                    self.log.push("[ OK ] CPU Core check: 4^4 Quat-Architecture Active".to_string());
                    self.log.push("[ OK ] Registers REG_A & REG_B initialized".to_string());
                    self.stage = BootStage::RamDiagnostic;
                }

                BootStage::RamDiagnostic => {
                    if self.frame_step < 16 {
                        // Testiranje 256 QuatBajtova u redovima po 16
                        let start_addr = self.frame_step * 16;
                        for i in 0..16 {
                            kernel.ram[start_addr + i] = QuatByte(255); // Flash piksel u VRAM-u za test
                        }
                        self.frame_step += 1;
                        self.progress = 0.2 + (self.frame_step as f32 / 16.0) * 0.3;
                    } else {
                        // Čišćenje RAM-a nakon testa
                        for i in 0..256 {
                            kernel.ram[i] = QuatByte(0);
                        }
                        self.log.push("[ OK ] RAM Diagnostic: 256/256 QuatBytes Verified".to_string());
                        self.frame_step = 0;
                        self.stage = BootStage::CrtSplashAnimation;
                    }
                }

                BootStage::CrtSplashAnimation => {
                    // Iscrtavanje $4^4$ Logotipa u 16x16 Framebuffer (Sredina ekrana)
                    let logo_pixels: [(usize, usize); 20] = [
                        // Crtanje cifre 4
                        (3, 4), (3, 5), (3, 6), (3, 7), (3, 8),
                        (4, 7), (5, 7), (6, 7), (6, 4), (6, 5), (6, 6), (6, 7), (6, 8), (6, 9),
                        // Crtanje gornjeg eksponenta 4 (4^4)
                        (9, 3), (9, 4), (9, 5), (10, 5), (11, 3), (11, 4),
                    ];

                    if self.frame_step < logo_pixels.len() {
                        let (x, y) = logo_pixels[self.frame_step];
                        let addr = y * 16 + x;
                        kernel.ram[addr] = QuatByte(255); // Upisivanje u VRAM
                        self.frame_step += 1;
                        self.progress = 0.5 + (self.frame_step as f32 / logo_pixels.len() as f32) * 0.3;
                    } else {
                        self.log.push("[ OK ] CRT Framebuffer VRAM Initialized".to_string());
                        self.log.push("[ OK ] QuatDSP Audio Core: Online".to_string());
                        self.stage = BootStage::MountQuatFS;
                    }
                }

                BootStage::MountQuatFS => {
                    self.progress = 0.95;
                    self.log.push("[ OK ] Mounting QuatFS Root Partition (0x20..0xFF)...".to_string());
                    self.log.push("✨ QuatOS Kernel Booted Successfully!".to_string());
                    self.progress = 1.0;
                    self.stage = BootStage::BootComplete;
                }

                _ => {}
            }
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, kernel: &mut QuatKernel) {
        self.update(kernel);

        ui.heading("🔌 QuatBIOS - Boot Sequencer");
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("🟢 POWER ON / REBOOT").clicked() {
                self.power_on(kernel);
            }

            if ui.button("🔴 POWER OFF").clicked() {
                self.stage = BootStage::PowerOff;
                self.progress = 0.0;
                self.log.clear();
                self.log.push("SYSTEM SHUTDOWN.".to_string());
                for i in 0..256 {
                    kernel.ram[i] = QuatByte(0);
                }
            }
        });

        ui.add_space(10.0);

        // Status Boot Sekvence
        let status_text = match self.stage {
            BootStage::PowerOff => "Isključen (Power Off)",
            BootStage::PostCpuCheck => "POST: Provera CPU-a...",
            BootStage::RamDiagnostic => "POST: Testiranje RAM memorije...",
            BootStage::CrtSplashAnimation => "GPU: Rendering $4^4$ Logotipa...",
            BootStage::MountQuatFS => "STORAGE: Montiranje QuatFS-a...",
            BootStage::BootComplete => "Sistem je spreman za rad!",
        };

        ui.group(|ui| {
            ui.label(format!("Status: {}", status_text));
            ui.add(egui::ProgressBar::new(self.progress).text(format!("{:.0}%", self.progress * 100.0)));
        });

        ui.add_space(10.0);

        // BIOS Post Log Terminal
        ui.strong("📟 BIOS Diagnostic Terminal Output:");
        egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                for entry in &self.log {
                    ui.monospace(egui::RichText::new(entry).color(egui::Color32::from_rgb(0, 255, 150)));
                }
            });
        });

        if self.stage != BootStage::PowerOff && self.stage != BootStage::BootComplete {
            ui.ctx().request_repaint_after(Duration::from_millis(50));
        }
    }
}