use eframe::egui;
use crate::kernel::QuatKernel;
use crate::assembler::Assembler;

pub struct QuatVM {
    pub code_input: String,
    pub compile_error: Option<String>,
}

impl QuatVM {
    pub fn new() -> Self {
        let sample_code = "// Primer QuatOS Asemblerskog Koda\n\
SET A, 12\n\
SET B, 30\n\
ADD\n\
// Rezultat u Reg A će biti 42 ([0][2][2][2] u kvatima)";

        Self {
            code_input: sample_code.to_string(),
            compile_error: None,
        }
    }

    pub fn compile_and_load(&mut self, kernel: &mut QuatKernel) {
        match Assembler::new().compile(&self.code_input) {
            Ok(bytecode) => {
                self.compile_error = None;
                kernel.pc = 0;
                
                // Resetuj RAM i upiši nov bajtkod
                for i in 0..256 {
                    kernel.ram[i] = crate::kernel::QuatByte(0);
                }

                for (idx, byte) in bytecode.iter().enumerate() {
                    if idx < 256 {
                        kernel.ram[idx] = *byte;
                    }
                }
                kernel.log(format!("Kompajlirano {} QuatBajtova u RAM!", bytecode.len()));
            }
            Err(err) => {
                self.compile_error = Some(err);
            }
        }
    }

    pub fn execute_step(kernel: &mut QuatKernel) {
    if kernel.pc as usize >= 255 {
        kernel.is_running = false;
        return;
    }

    let opcode = kernel.ram[kernel.pc as usize].0;
    kernel.pc += 1;

    match opcode {
        0 => {} // NOP
        1 => {  // SET A, <val>
            let val = kernel.ram[kernel.pc as usize].0;
            kernel.reg_a = crate::kernel::QuatByte(val);
            kernel.pc += 1;
        }
        2 => {  // SET B, <val>
            let val = kernel.ram[kernel.pc as usize].0;
            kernel.reg_b = crate::kernel::QuatByte(val);
            kernel.pc += 1;
        }
        3 => {  // ADD
            kernel.reg_a = crate::kernel::QuatByte(kernel.reg_a.0.wrapping_add(kernel.reg_b.0));
        }
        4 => {  // SUB
            kernel.reg_a = crate::kernel::QuatByte(kernel.reg_a.0.wrapping_sub(kernel.reg_b.0));
        }
        5 => {  // JMP <addr>
            let addr = kernel.ram[kernel.pc as usize].0;
            kernel.pc = addr;
        }

        // --- HARDVERSKO IZVRŠAVANJE CRT DRAJVERA ---
        6 => {  // CLS - Očisti svih 256 bajtova VRAM-a
            for i in 0..256 {
                kernel.ram[i] = crate::kernel::QuatByte(0);
            }
        }
        7 => {  // DRAW X, Y - Direktno crtanje piksela
            let x = kernel.ram[kernel.pc as usize].0 as usize;
            kernel.pc += 1;
            let y = kernel.ram[kernel.pc as usize].0 as usize;
            kernel.pc += 1;

            if x < 16 && y < 16 {
                let addr = y * 16 + x;
                kernel.ram[addr] = crate::kernel::QuatByte(255);
            }
        }
        8 => {  // DRAW (Koristi Reg A kao X, i Reg B kao Y)
            let x = kernel.reg_a.0 as usize;
            let y = kernel.reg_b.0 as usize;

            if x < 16 && y < 16 {
                let addr = y * 16 + x;
                kernel.ram[addr] = crate::kernel::QuatByte(255);
            }
        }
        _ => {}
    }
}

    pub fn ui(&mut self, ui: &mut egui::Ui, kernel: &mut QuatKernel) {
        ui.heading("⚙️ Quat-VM Asembler & Bajtkod IDE");
        ui.separator();

        ui.columns(2, |cols| {
            // Leva kolona: Code Editor
            cols[0].vertical(|ui| {
                ui.strong("✍️ Quat-Assembly Editor:");
                ui.add_space(5.0);
                
                ui.add(
                    egui::TextEdit::multiline(&mut self.code_input)
                        .font(egui::TextStyle::Monospace)
                        .code_editor()
                        .desired_rows(15)
                        .desired_width(f32::INFINITY)
                );

                ui.add_space(5.0);
                if ui.button("🛠️ Compajliraj i Učitaj u RAM").clicked() {
                    self.compile_and_load(kernel);
                }

                if let Some(err) = &self.compile_error {
                    ui.label(egui::RichText::new(err).color(egui::Color32::RED));
                }
            });

            // Desna kolona: VM Prikaz stanja
            cols[1].vertical(|ui| {
                ui.strong("💻 VM Izvršavanje & Registri:");
                ui.add_space(5.0);

                let q_a = kernel.reg_a.to_quats();
                let q_b = kernel.reg_b.to_quats();

                ui.group(|ui| {
                    ui.label(format!("PC (Program Counter): {:02X}", kernel.pc));
                    ui.label(format!("Reg A: {} | Kvat: [{}{}{}{}]", kernel.reg_a.0, q_a[0], q_a[1], q_a[2], q_a[3]));
                    ui.label(format!("Reg B: {} | Kvat: [{}{}{}{}]", kernel.reg_b.0, q_b[0], q_b[1], q_b[2], q_b[3]));
                });

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("▶ Izvrši Takt (Step)").clicked() {
                        Self::execute_step(kernel);
                    }
                });
            });
        });
    }
}