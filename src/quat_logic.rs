use eframe::egui;
use crate::kernel::{QuatKernel, QuatByte};
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct BusSample {
    pub cycle: usize,
    pub clk: bool,
    pub rw: bool, // true = Read (Zeleno), false = Write (Crveno)
    pub pc_addr: u8,
    pub data_bus: u8,
    pub reg_a: u8,
    pub reg_b: u8,
}

pub struct QuatLogic {
    pub history: VecDeque<BusSample>,
    pub max_samples: usize,
    pub clock_cycle: usize,
    pub break_pc: String,
    pub break_data: String,
    pub breakpoint_hit: bool,
    pub is_sampling: bool,
    pub status_msg: String,
}

impl QuatLogic {
    pub fn new() -> Self {
        Self {
            history: VecDeque::new(),
            max_samples: 24, // Broj vremenskih slotova prikazanih na ekranu
            clock_cycle: 0,
            break_pc: String::new(),
            break_data: String::new(),
            breakpoint_hit: false,
            is_sampling: true,
            status_msg: "QuatLogic spreman. Magistrala pod nadzorom.".to_string(),
        }
    }

    // Uzimanje uzorka signala sa magistrale u svakom taktu CPU-a
    pub fn sample_bus(&mut self, kernel: &QuatKernel, rw: bool) {
        if !self.is_sampling {
            return;
        }

        self.clock_cycle += 1;
        let pc = kernel.pc;
        let data = if (pc as usize) < 256 { kernel.ram[pc as usize].0 } else { 0 };

        // Provera Hardware Breakpoint uslova (PC Adresa)
        if let Ok(target_pc) = self.break_pc.trim().parse::<u8>() {
            if pc == target_pc {
                self.breakpoint_hit = true;
                self.is_sampling = false;
                self.status_msg = format!("🎯 BREAKPOINT TRAP! PC dostigao 0x{:02X}", pc);
            }
        }

        // Provera Hardware Breakpoint uslova (Vrednost na Data Bus-u)
        if let Ok(target_data) = self.break_data.trim().parse::<u8>() {
            if data == target_data {
                self.breakpoint_hit = true;
                self.is_sampling = false;
                self.status_msg = format!("🎯 BREAKPOINT TRAP! Data Bus ima vrednost {} (0x{:02X})", data, data);
            }
        }

        let sample = BusSample {
            cycle: self.clock_cycle,
            clk: self.clock_cycle % 2 == 0,
            rw,
            pc_addr: pc,
            data_bus: data,
            reg_a: kernel.reg_a.0,
            reg_b: kernel.reg_b.0,
        };

        if self.history.len() >= self.max_samples {
            self.history.pop_front();
        }
        self.history.push_back(sample);
    }

    pub fn reset_trace(&mut self) {
        self.history.clear();
        self.clock_cycle = 0;
        self.breakpoint_hit = false;
        self.is_sampling = true;
        self.status_msg = "Sistemski tragovi (Traces) resetovani.".to_string();
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, kernel: &mut QuatKernel) {
        ui.heading("🔬 QuatLogic - Interactive Hardware Debugger & Logic Analyzer");
        ui.separator();

        // 1. Kontrolna tabla & Breakpoints
        ui.group(|ui| {
            ui.horizontal(|ui| {
                if ui.button(if self.is_sampling { "⏸ Pauziraj Uzorkovanje" } else { "▶ Nastavi Uzorkovanje" }).clicked() {
                    self.is_sampling = !self.is_sampling;
                }

                if ui.button("🔄 Resetuj Analizator").clicked() {
                    self.reset_trace();
                }

                ui.separator();

                ui.label("🎯 Trap PC:");
                ui.add(egui::TextEdit::singleline(&mut self.break_pc).desired_width(40.0));

                ui.label("🎯 Trap Data:");
                ui.add(egui::TextEdit::singleline(&mut self.break_data).desired_width(40.0));

                if self.breakpoint_hit {
                    ui.label(egui::RichText::new("⚠️ PREKID AKTIVAN").color(egui::Color32::RED).strong());
                }
            });
        });

        ui.add_space(10.0);

        // 2. Trenutno stanje magistrale (Digital Probe Readings)
        let latest = self.history.back();
        ui.columns(4, |cols| {
            cols[0].group(|ui| {
                ui.label("CLK Signal");
                let clk_val = latest.map_or(false, |s| s.clk);
                ui.strong(if clk_val { "HIGH (1)" } else { "LOW (0)" });
            });
            cols[1].group(|ui| {
                ui.label("R/W Linija");
                let rw_val = latest.map_or(true, |s| s.rw);
                ui.strong(if rw_val { "READ (1)" } else { "WRITE (0)" });
            });
            cols[2].group(|ui| {
                ui.label("Address Bus (PC)");
                let pc_val = latest.map_or(0, |s| s.pc_addr);
                let q = QuatByte(pc_val).to_quats();
                ui.monospace(format!("0x{:02X} [{}{}{}{}_4]", pc_val, q[0], q[1], q[2], q[3]));
            });
            cols[3].group(|ui| {
                ui.label("Data Bus (RAM[PC])");
                let data_val = latest.map_or(0, |s| s.data_bus);
                let q = QuatByte(data_val).to_quats();
                ui.monospace(format!("0x{:02X} [{}{}{}{}_4]", data_val, q[0], q[1], q[2], q[3]));
            });
        });

        ui.add_space(10.0);
        ui.strong("📈 Digital Waveform Oscilloscope (Vremenski dijagram linija):");
        ui.add_space(5.0);

        // 3. Iscrtavanje digitalnih talasnih oblika pomoću egui Painter-a
        let frame_height = 180.0;
        let (rect, _response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), frame_height),
            egui::Sense::hover(),
        );

        let painter = ui.painter_at(rect);
        // Pozadina osciloskopa
        painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(15, 20, 25));
        painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 60, 80)));

        if !self.history.is_empty() {
            let sample_count = self.history.len();
            let step_x = rect.width() / (self.max_samples as f32);

            let clk_base_y = rect.min.y + 30.0;
            let rw_base_y = rect.min.y + 75.0;
            let addr_base_y = rect.min.y + 120.0;

            // Oznake kanala na osciloskopu
            painter.text(
                egui::pos2(rect.min.x + 10.0, clk_base_y - 10.0),
                egui::Align2::LEFT_CENTER,
                "CLK",
                egui::FontId::monospace(12.0),
                egui::Color32::YELLOW,
            );
            painter.text(
                egui::pos2(rect.min.x + 10.0, rw_base_y - 10.0),
                egui::Align2::LEFT_CENTER,
                "R/W",
                egui::FontId::monospace(12.0),
                egui::Color32::from_rgb(0, 255, 255),
            );
            painter.text(
                egui::pos2(rect.min.x + 10.0, addr_base_y - 10.0),
                egui::Align2::LEFT_CENTER,
                "BUS",
                egui::FontId::monospace(12.0),
                egui::Color32::GREEN,
            );

            for i in 0..sample_count {
                let sample = &self.history[i];
                let x1 = rect.min.x + (i as f32) * step_x + 50.0;
                let x2 = x1 + step_x;

                // 1. CLK Talas (Kvadratni signal)
                let clk_y = if sample.clk { clk_base_y - 12.0 } else { clk_base_y + 8.0 };
                painter.line_segment(
                    [egui::pos2(x1, clk_y), egui::pos2(x2, clk_y)],
                    egui::Stroke::new(2.0, egui::Color32::YELLOW),
                );
                if i > 0 {
                    let prev_clk_y = if self.history[i - 1].clk { clk_base_y - 12.0 } else { clk_base_y + 8.0 };
                    painter.line_segment(
                        [egui::pos2(x1, prev_clk_y), egui::pos2(x1, clk_y)],
                        egui::Stroke::new(2.0, egui::Color32::YELLOW),
                    );
                }

                // 2. R/W Talas
                let rw_y = if sample.rw { rw_base_y - 12.0 } else { rw_base_y + 8.0 };
                let rw_color = if sample.rw { egui::Color32::GREEN } else { egui::Color32::RED };
                painter.line_segment(
                    [egui::pos2(x1, rw_y), egui::pos2(x2, rw_y)],
                    egui::Stroke::new(2.0, rw_color),
                );

                // 3. Address/Data Bus (Blokovski prikaz vrednosti)
                let bus_box = egui::Rect::from_min_max(
                    egui::pos2(x1 + 1.0, addr_base_y - 12.0),
                    egui::pos2(x2 - 1.0, addr_base_y + 12.0),
                );
                painter.rect_filled(bus_box, 2.0, egui::Color32::from_rgb(30, 50, 40));
                painter.rect_stroke(bus_box, 2.0, egui::Stroke::new(1.0, egui::Color32::GREEN));

                painter.text(
                    bus_box.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{:02X}", sample.pc_addr),
                    egui::FontId::monospace(10.0),
                    egui::Color32::WHITE,
                );
            }
        }

        ui.add_space(10.0);
        ui.separator();
        ui.monospace(format!("Status Analizatora: {}", self.status_msg));
    }
}