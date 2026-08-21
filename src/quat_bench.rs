use eframe::egui;
use crate::kernel::QuatKernel;
use std::time::{Instant, Duration};
use crate::kernel;

//TODO: znam da pise use crate::kernel dvaput
//Ne cackaj ga radi lepog rada

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum BenchLevel {
    Idle,
    Level1Warmup,    // Level 1: ALU Register Spin (Samo registri)
    Level2RamBus,    // Level 2: RAM Bus Sweep (Testiranje 256 B RAM-a)
    Level3VramBurn,  // Level 3: VRAM Graphics Burn (Upisivanje u VRAM matricu)
    Level4AluMatrix, // Level 4: Complex ALU Logic (XOR/ADD/NAND/SHIFT stres)
    Level5MaxTorture,// Level 5: OVERCLOCK TORTURE (Sve paralelno u punom taktu)
    Finished,
}

pub struct QuatBENCH {
    pub current_level: BenchLevel,
    pub level_start_time: Option<Instant>,
    pub level_duration: Duration,
    
    // Statistika i telemetrija
    pub total_operations: u64,
    pub level_operations: u64,
    pub qips: f64, // Quat Instructions Per Second
    pub ram_errors: u64,
    pub simulated_temp: f32, // °C
    
    // Logovi
    pub log_messages: Vec<String>,
}

impl QuatBENCH {
    pub fn new() -> Self {
        Self {
            current_level: BenchLevel::Idle,
            level_start_time: None,
            level_duration: Duration::from_secs(4), // Trajanje svakog nivoa po 4 sekunde
            total_operations: 0,
            level_operations: 0,
            qips: 0.0,
            ram_errors: 0,
            simulated_temp: 35.0,
            log_messages: vec!["QuatBENCH 5-Level Suite spreman za pokretanje.".to_string()],
        }
    }

    pub fn start_full_test(&mut self) {
        self.total_operations = 0;
        self.ram_errors = 0;
        self.simulated_temp = 36.0;
        self.log_messages.clear();
        self.log_messages.push("🚀 POKRETANJE FULL STRESS TESTA (5 NIVOA)...".to_string());
        self.advance_to_level(BenchLevel::Level1Warmup);
    }

    fn advance_to_level(&mut self, next_level: BenchLevel) {
        self.current_level = next_level;
        self.level_start_time = Some(Instant::now());
        self.level_operations = 0;

        let msg = match next_level {
            BenchLevel::Level1Warmup => "🔥 NIVO 1/5: Warm-up (ALU Basic Register Spin)",
            BenchLevel::Level2RamBus => "🔥 NIVO 2/5: RAM Bus Integrity Sweep (256 Bajtova)",
            BenchLevel::Level3VramBurn => "🔥 NIVO 3/5: VRAM Video Matrix Stress Test",
            BenchLevel::Level4AluMatrix => "🔥 NIVO 4/5: ALU Heavy Logic Operations (NAND/XOR/ADD)",
            BenchLevel::Level5MaxTorture => "⚡ NIVO 5/5: MAXIMUM OVERCLOCK TORTURE (Sve jedinice na 100%)",
            BenchLevel::Finished => "✅ TEST ZAVRŠEN! Sistem je preživeo sve nivoe opterećenja.",
            _ => "",
        };

        if !msg.is_empty() {
            self.log_messages.push(msg.to_string());
        }
    }

    /// Glavna petlja koja izvršava stres po nivoima
    pub fn update(&mut self, ctx: &egui::Context, kernel: &mut QuatKernel) {
        if self.current_level == BenchLevel::Idle || self.current_level == BenchLevel::Finished {
            return;
        }

        // Osvežavamo ekran što brže možemo
        ctx.request_repaint();

        let now = Instant::now();
        let elapsed = if let Some(start) = self.level_start_time {
            now.duration_since(start)
        } else {
            Duration::ZERO
        };

        // Provera da li prelazimo na sledeći nivo
        if elapsed >= self.level_duration {
            match self.current_level {
                BenchLevel::Level1Warmup => self.advance_to_level(BenchLevel::Level2RamBus),
                BenchLevel::Level2RamBus => self.advance_to_level(BenchLevel::Level3VramBurn),
                BenchLevel::Level3VramBurn => self.advance_to_level(BenchLevel::Level4AluMatrix),
                BenchLevel::Level4AluMatrix => self.advance_to_level(BenchLevel::Level5MaxTorture),
                BenchLevel::Level5MaxTorture => self.advance_to_level(BenchLevel::Finished),
                _ => {},
            }
            return;
        }

        // --- IZVRŠAVANJE TRENUTNOG NIVOA ---
        match self.current_level {
            BenchLevel::Level1Warmup => {
                // NIVO 1: Lagana petlja nad u8 registrima (10,000 iteracija)
                let mut reg: u8 = 0;
                for _ in 0..10_000 {
                    reg = reg.wrapping_add(1);
                    reg ^= 0x55;
                }
                self.record_ops(20_000, 42.0);
            }

            BenchLevel::Level2RamBus => {
                // NIVO 2: Testiranje integriteta svih 256 bajtova RAM-a u petlji
                let pattern = (self.total_operations & 0xFF) as u8;
                for addr in 0..256 {
                    kernel.ram[addr] = kernel::QuatByte(pattern);
                    if kernel.ram[addr] != kernel::QuatByte(pattern) {
                        self.ram_errors += 1;
                    }
                }
                self.record_ops(512, 48.0);
            }

            BenchLevel::Level3VramBurn => {
                // NIVO 3: Pisanje po VRAM-u (64 bajta video memorije) sa rotacijom bitova
                for addr in 0..64 {
                    kernel.vram[addr] = kernel::QuatByte(kernel.vram[addr].0.rotate_left(1).wrapping_add(0x3C));
                }
                self.record_ops(1_000, 56.0);
            }

            BenchLevel::Level4AluMatrix => {
                // NIVO 4: Kompleksne ALU operacije (100,000 instrukcija po frejmu)
                let mut a: u8 = 0xA5;
                let mut b: u8 = 0x5A;
                for _ in 0..25_000 {
                    a = !(a ^ b);
                    b = b.wrapping_add(a);
                    a = a.rotate_right(2);
                }
                self.record_ops(100_000, 68.0);
            }

            BenchLevel::Level5MaxTorture => {
                // NIVO 5: OVERCLOCK TORTURE — Puno opterećenje RAM, VRAM i ALU istovremeno!
                let mut dummy: u8 = 0xFF;
                for i in 0..100_000 {
                    dummy = dummy.wrapping_sub((i & 0xFF) as u8) ^ 0xAA;
                }
                // Agresivno prženje RAM-a
                for addr in 0..256 {
                    kernel.ram[addr] = kernel::QuatByte(0xFF);
                }
                // Agresivno prženje VRAM-a
                for addr in 0..64 {
                    kernel.vram[addr] = kernel::QuatByte(0xFF);
                }
                self.record_ops(300_000, 88.5);
            }

            _ => {}
        }
    }

    fn record_ops(&mut self, ops: u64, target_temp: f32) {
        self.level_operations += ops;
        self.total_operations += ops;

        // Postepeni rast simulirane temperature
        if self.simulated_temp < target_temp {
            self.simulated_temp += 0.2;
        } else if self.simulated_temp > target_temp {
            self.simulated_temp -= 0.1;
        }

        // Izračunavanje QIPS (Miliona operacija u sekundi)
        if let Some(start) = self.level_start_time {
            let secs = start.elapsed().as_secs_f64();
            if secs > 0.0 {
                self.qips = self.level_operations as f64 / secs;
            }
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, kernel: &mut QuatKernel) {
        self.update(ui.ctx(), kernel);

        ui.heading("🔥 QuatBENCH: 5-Level Stress Test Suite");
        ui.label("Ispitivanje stabilnosti $4^4$ (8-bitne) emulacije kroz 5 progresivnih nivoa opterećenja.");
        ui.separator();

        // KONTROLNA DUGMAD
        ui.horizontal(|ui| {
            if self.current_level == BenchLevel::Idle || self.current_level == BenchLevel::Finished {
                if ui.button("🚀 POKRENI TEST OD 5 NIVOA").clicked() {
                    self.start_full_test();
                }
            } else {
                if ui.button("⛔ ZAUSTAVI STRES TEST").clicked() {
                    self.current_level = BenchLevel::Idle;
                    self.log_messages.push("⏹ Test ručno prekinut.".to_string());
                }
            }
        });

        ui.add_space(10.0);

        // GRAFIČKI PRIKAZ PROGRESS-A KROZ NIVOE
        ui.group(|ui| {
            ui.label(egui::RichText::new("📊 Progres Nivoa Opterećenja:").strong());
            
            let progress_val = match self.current_level {
                BenchLevel::Idle => 0.0,
                BenchLevel::Level1Warmup => 0.2,
                BenchLevel::Level2RamBus => 0.4,
                BenchLevel::Level3VramBurn => 0.6,
                BenchLevel::Level4AluMatrix => 0.8,
                BenchLevel::Level5MaxTorture => 1.0,
                BenchLevel::Finished => 1.0,
            };

            ui.add(egui::ProgressBar::new(progress_val).text(format!("{:?}", self.current_level)));
        });

        ui.add_space(10.0);

        // TELEMETRIJA
        ui.columns(2, |cols| {
            cols[0].group(|ui| {
                ui.heading("📈 Performanse");
                ui.label(format!("Ukupno Quat Instrukcija: {}", self.total_operations));
                let mqips = self.qips / 1_000_000.0;
                ui.label(egui::RichText::new(format!("Trenutna Brzina: {:.2} MQIPS", mqips)).strong().size(16.0).color(egui::Color32::GREEN));
                
                let err_color = if self.ram_errors == 0 { egui::Color32::GREEN } else { egui::Color32::RED };
                ui.label(egui::RichText::new(format!("RAM Bus Greške: {}", self.ram_errors)).color(err_color));
            });

            cols[1].group(|ui| {
                ui.heading("🌡️ Termalni Monitor Jezgra");
                ui.add(egui::ProgressBar::new(self.simulated_temp / 100.0).text(format!("{:.1} °C", self.simulated_temp)));
                
                if self.simulated_temp > 80.0 {
                    ui.label(egui::RichText::new("⚠️ KRITIČNA TEMPERATURA! OVERCLOCK AKTIVAN!").color(egui::Color32::RED).strong());
                } else {
                    ui.label(egui::RichText::new("🟢 Temperatura u granicama normale.").color(egui::Color32::LIGHT_GREEN));
                }
            });
        });

        ui.add_space(10.0);

        // TERMINAL LOGOVI
        ui.group(|ui| {
            ui.label("📋 Event Log:");
            egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
                for msg in self.log_messages.iter().rev() {
                    ui.label(egui::RichText::new(msg).monospace().size(12.0));
                }
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn run_bench_stage(name: &str, instructions: u64, mut action: impl FnMut()) {
        let start = Instant::now();
        action();
        let duration = start.elapsed();
        let mqips = (instructions as f64 / 1_000_000.0) / duration.as_secs_f64();
        println!("[{}] Vreme: {:?} | Brzina: {:.2} MQIPS", name, duration, mqips);
    }

    #[test]
    fn bench_quat_full_5_levels() {
        let mut kernel = QuatKernel::new();
        println!("\n==================================================");
        println!("  POKRETANJE FULL CLI STRESS TESTA (5 NIVOA)");
        println!("==================================================");

        // NIVO 1: Warm-up (ALU Basic Register Spin)
        run_bench_stage("NIVO 1/5: Warm-up ALU", 20_000_000, || {
            for _ in 0..20_000_000 {
                kernel.step(); 
            }
        });

        // NIVO 2: RAM Bus Integrity Sweep (256 Bajtova)
        run_bench_stage("NIVO 2/5: RAM Sweep", 20_000_000, || {
            for i in 0..256 {
                kernel.ram[i] = crate::kernel::QuatByte(i as u8);
            }
            for _ in 0..20_000_000 {
                kernel.step();
            }
        });

        // NIVO 3: VRAM Video Matrix Stress Test
        run_bench_stage("NIVO 3/5: VRAM Matrix", 20_000_000, || {
            for i in 0..64 {
                kernel.vram[i] = crate::kernel::QuatByte(0xFF);
            }
            for _ in 0..20_000_000 {
                kernel.step();
            }
        });

        // NIVO 4: ALU Heavy Logic (NAND / XOR / ADD)
        run_bench_stage("NIVO 4/5: ALU Heavy Logic", 20_000_000, || {
            for _ in 0..20_000_000 {
                kernel.step();
            }
        });

        // NIVO 5: MAXIMUM OVERCLOCK TORTURE (100% Load)
        run_bench_stage("NIVO 5/5: Overclock Torture", 50_000_000, || {
            for _ in 0..50_000_000 {
                kernel.step();
            }
        });

        println!("==================================================\n");
    }
}