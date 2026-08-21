use eframe::egui;
use crate::kernel::QuatKernel;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AluOp {
    Add,
    Sub,
    And,
    Or,
    Xor,
    Nand,
    Shl,
    Shr,
}

impl AluOp {
    pub fn name(&self) -> &'static str {
        match self {
            AluOp::Add => "ADD (+)",
            AluOp::Sub => "SUB (-)",
            AluOp::And => "AND (&)",
            AluOp::Or => "OR (|)",
            AluOp::Xor => "XOR (^)",
            AluOp::Nand => "NAND (~&)",
            AluOp::Shl => "SHL (<<)",
            AluOp::Shr => "SHR (>>)",
        }
    }
}

pub struct QuatALU {
    // 4-Kvatni Registri (Vrednosti 0 do 255 -> 8 bita / 4 kvata od po 2 bita)
    pub reg_a: u8,
    pub reg_b: u8,
    pub selected_op: AluOp,
    
    // Konverter Input Polja (Stringovi radi lakšeg kucanja)
    pub input_base4: String,
    pub input_dec: String,
    pub input_hex: String,
    pub input_bin: String,
    
    pub active_tab: usize, // 0: Base Converter, 1: ALU Register Inspector
    pub status_msg: String,
}

impl QuatALU {
    pub fn new() -> Self {
        let mut alu = Self {
            reg_a: 0b10_01_11_00, // Primer inicijalnih kvata (2, 1, 3, 0)
            reg_b: 0b01_00_10_01,
            selected_op: AluOp::Add,
            input_base4: String::new(),
            input_dec: "42".to_string(),
            input_hex: "2A".to_string(),
            input_bin: "00101010".to_string(),
            active_tab: 0,
            status_msg: "QuatALU & Base Converter spreman za rad.".to_string(),
        };
        alu.sync_from_dec(42);
        alu
    }

    /// Prevara konverzije iz dekadnog broja u sve ostale osnove
    fn sync_from_dec(&mut self, val: u8) {
        self.input_dec = val.to_string();
        self.input_hex = format!("{:02X}", val);
        self.input_bin = format!("{:08b}", val);
        self.input_base4 = Self::to_quat_string(val);
    }

    /// Pretvara u8 broj u 4-kvatni Base-4 string (npr. 42 -> "0222")
    fn to_quat_string(val: u8) -> String {
        let q3 = (val >> 6) & 0x03;
        let q2 = (val >> 4) & 0x03;
        let q1 = (val >> 2) & 0x03;
        let q0 = val & 0x03;
        format!("{}{}{}{}", q3, q2, q1, q0)
    }

    /// Konvertuje Base-4 string u u8 vrednost
    fn parse_quat_string(s: &str) -> Result<u8, &'static str> {
        if s.len() > 4 { return Err("Maksimum 4 kvata!"); }
        let mut val: u8 = 0;
        for ch in s.chars() {
            if let Some(digit) = ch.to_digit(4) {
                val = (val << 2) | (digit as u8);
            } else {
                return Err("Nevažeći kvat! Dozvoljene cifre: 0, 1, 2, 3");
            }
        }
        Ok(val)
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, _kernel: &mut QuatKernel) {
        ui.heading("🧮 QuatALU: Arithmetic Logic Unit & Base Converter");
        ui.label("Hardware-level inspektor registara, konverter brojevnih sistema i simulator ALU kapija.");
        ui.separator();

        // TAB SELEKTOR
        ui.horizontal(|ui| {
            if ui.selectable_label(self.active_tab == 0, "🔄 Multi-Base Convertor").clicked() { self.active_tab = 0; }
            if ui.selectable_label(self.active_tab == 1, "⚡ ALU Register Inspector").clicked() { self.active_tab = 1; }
        });

        ui.separator();

        match self.active_tab {
            0 => self.render_converter_tab(ui),
            1 => self.render_alu_tab(ui),
            _ => {}
        }

        ui.add_space(15.0);
        ui.label(egui::RichText::new(&self.status_msg).italics().color(egui::Color32::LIGHT_BLUE));
    }

    // =========================================================================
    // 1. MULTI-BASE CONVERTOR TAB
    // =========================================================================
    fn render_converter_tab(&mut self, ui: &mut egui::Ui) {
        ui.label("🔢 Konverzija realnog vremena između 4 osnovebrojevnih sistema:");
        ui.add_space(10.0);

        egui::Grid::new("base_converter_grid").spacing([15.0, 10.0]).show(ui, |ui| {
            // QUATERNARY (BASE 4)
            ui.label(egui::RichText::new("Base-4 (Kvatni):").strong().color(egui::Color32::GREEN));
            let res_q = ui.text_edit_singleline(&mut self.input_base4);
            if res_q.changed() {
                match Self::parse_quat_string(&self.input_base4) {
                    Ok(v) => {
                        self.sync_from_dec(v);
                        self.status_msg = format!("Konvertovan Kvatni unos u decimalni: {}", v);
                    }
                    Err(e) => self.status_msg = format!("Greška: {}", e),
                }
            }
            ui.label("Cifre: [0, 1, 2, 3]");
            ui.end_row();

            // DECIMAL (BASE 10)
            ui.label(egui::RichText::new("Base-10 (Decimalni):").strong().color(egui::Color32::KHAKI));
            let res_d = ui.text_edit_singleline(&mut self.input_dec);
            if res_d.changed() {
                if let Ok(v) = self.input_dec.parse::<u8>() {
                    self.sync_from_dec(v);
                    self.status_msg = "Decimalni unos ažuriran.".to_string();
                } else {
                    self.status_msg = "Unesite validan 8-bitni broj (0 - 255).".to_string();
                }
            }
            ui.label("Opseg: 0..255");
            ui.end_row();

            // HEXADECIMAL (BASE 16)
            ui.label(egui::RichText::new("Base-16 (Heksadecimalni):").strong().color(egui::Color32::from_rgb(255, 128, 255)));
            let res_h = ui.text_edit_singleline(&mut self.input_hex);
            if res_h.changed() {
                if let Ok(v) = u8::from_str_radix(&self.input_hex, 16) {
                    self.sync_from_dec(v);
                    self.status_msg = "Heksadecimalni unos ažuriran.".to_string();
                }
            }
            ui.label("Format: 00..FF");
            ui.end_row();

            // BINARY (BASE 2)
            ui.label(egui::RichText::new("Base-2 (Binarni):").strong().color(egui::Color32::from_rgb(255, 128, 255)));
            let res_b = ui.text_edit_singleline(&mut self.input_bin);
            if res_b.changed() {
                if let Ok(v) = u8::from_str_radix(&self.input_bin, 2) {
                    self.sync_from_dec(v);
                    self.status_msg = "Binarni unos ažuriran.".to_string();
                }
            }
            ui.label("Format: 8 bita (0/1)");
            ui.end_row();
        });

        ui.add_space(15.0);
        ui.group(|ui| {
            ui.label(egui::RichText::new("💡 Brzi Kvatni Priručnik:").strong());
            ui.label("1 Kvat (2 bita) obuhvata opseg vrednosti od 0 do 3.");
            ui.label("4 Kvata (8 bita / 1 Bajt) obuhvataju opseg od 0000_4 (0_10) do 3333_4 (255_10).");
        });
    }

    // =========================================================================
    // 2. ALU REGISTER INSPECTOR TAB
    // =========================================================================
    fn render_alu_tab(&mut self, ui: &mut egui::Ui) {
        ui.label("⚡ Hardverska simulacija izvršavanja operacija na nivou registara:");
        ui.add_space(10.0);

        // KONTROLA REGISTARA A i B
        ui.columns(2, |cols| {
            cols[0].group(|ui| {
                ui.heading("Registari A (REG_A)");
                ui.add(egui::Slider::new(&mut self.reg_a, 0..=255).text("Vrednost"));
                ui.label(format!("Kvatni zapis:  {}", Self::to_quat_string(self.reg_a)));
                ui.label(format!("Binarni zapis: {:08b}", self.reg_a));
                ui.label(format!("Hex zapis:     0x{:02X}", self.reg_a));
            });

            cols[1].group(|ui| {
                ui.heading("Registari B (REG_B)");
                ui.add(egui::Slider::new(&mut self.reg_b, 0..=255).text("Vrednost"));
                ui.label(format!("Kvatni zapis:  {}", Self::to_quat_string(self.reg_b)));
                ui.label(format!("Binarni zapis: {:08b}", self.reg_b));
                ui.label(format!("Hex zapis:     0x{:02X}", self.reg_b));
            });
        });

        ui.add_space(10.0);
        ui.label(egui::RichText::new("Izaberi ALU Operaciju:").strong());

        ui.horizontal(|ui| {
            let ops = [
                AluOp::Add, AluOp::Sub, AluOp::And, AluOp::Or, 
                AluOp::Xor, AluOp::Nand, AluOp::Shl, AluOp::Shr
            ];

            for op in ops.iter() {
                if ui.selectable_label(self.selected_op == *op, op.name()).clicked() {
                    self.selected_op = *op;
                }
            }
        });

        ui.separator();

        // PRORAČUN REZULTATA I FLAGS
        let (raw_res, overflow) = match self.selected_op {
            AluOp::Add => self.reg_a.overflowing_add(self.reg_b),
            AluOp::Sub => self.reg_a.overflowing_sub(self.reg_b),
            AluOp::And => (self.reg_a & self.reg_b, false),
            AluOp::Or  => (self.reg_a | self.reg_b, false),
            AluOp::Xor => (self.reg_a ^ self.reg_b, false),
            AluOp::Nand => (!(self.reg_a & self.reg_b), false),
            AluOp::Shl => (self.reg_a.wrapping_shl(1), self.reg_a & 0x80 != 0),
            AluOp::Shr => (self.reg_a.wrapping_shr(1), self.reg_a & 0x01 != 0),
        };

        let zero_flag = raw_res == 0;
        let negative_flag = (raw_res & 0x80) != 0;

        ui.group(|ui| {
            ui.heading("📊 ALU Rezultat (Output Register Output):");
            ui.add_space(5.0);

            egui::Grid::new("alu_result_grid").spacing([20.0, 5.0]).show(ui, |ui| {
                ui.label("Dekadni Rezultat:");
                ui.label(egui::RichText::new(format!("{}", raw_res)).strong().size(18.0).color(egui::Color32::GREEN));
                ui.end_row();

                ui.label("Kvatni (Base-4) Rezultat:");
                ui.label(egui::RichText::new(Self::to_quat_string(raw_res)).strong().size(18.0).color(egui::Color32::YELLOW));
                ui.end_row();

                ui.label("Hex Rezultat:");
                ui.label(egui::RichText::new(format!("0x{:02X}", raw_res)).strong().size(18.0).color(egui::Color32::from_rgb(128, 255, 255)));
                ui.end_row();

                ui.label("Binarni Rezultat:");
                ui.label(egui::RichText::new(format!("{:08b}", raw_res)).strong().size(18.0).color(egui::Color32::from_rgb(255, 128, 255)));
                ui.end_row();
            });

            ui.separator();
            ui.label(egui::RichText::new("🏁 Hardware Status Flags (Statusni Registar CPU-a):").strong());

            ui.horizontal(|ui| {
                let flag_style = |active: bool, text: &str| {
                    if active {
                        egui::RichText::new(text).strong().background_color(egui::Color32::RED).color(egui::Color32::WHITE)
                    } else {
                        egui::RichText::new(text).color(egui::Color32::GRAY)
                    }
                };

                ui.label(flag_style(zero_flag, " [ Z: Zero ] "));
                ui.label(flag_style(overflow, " [ C/O: Carry/Overflow ] "));
                ui.label(flag_style(negative_flag, " [ N: Negative ] "));
            });
        });
    }
}