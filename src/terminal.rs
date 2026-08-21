use eframe::egui;
use crate::kernel::QuatKernel;

pub struct TerminalLine {
    pub text: String,
    pub color: egui::Color32,
}

pub struct QuatTerminal {
    pub input_buffer: String,
    pub history: Vec<TerminalLine>,
    pub cmd_history: Vec<String>,
    pub cmd_index: usize,
}

impl QuatTerminal {
    pub fn new() -> Self {
        let mut term = Self {
            input_buffer: String::new(),
            history: Vec::new(),
            cmd_history: Vec::new(),
            cmd_index: 0,
        };

        term.print("QuatOS Shell v4.4 (256-State Architecture)", egui::Color32::GREEN);
        term.print("Kucajte 'help' za spisak dostupnih komandi.\n", egui::Color32::GRAY);
        term
    }

    pub fn print(&mut self, text: impl Into<String>, color: egui::Color32) {
        self.history.push(TerminalLine { text: text.into(), color });
    }

    pub fn execute(&mut self, kernel: &mut QuatKernel) {
        let input = self.input_buffer.trim().to_string();
        if input.is_empty() { return; }

        self.print(format!("quat@kernel:~ $ {}", input), egui::Color32::WHITE);
        self.cmd_history.push(input.clone());
        self.cmd_index = self.cmd_history.len();
        self.input_buffer.clear();

        let parts: Vec<&str> = input.split_whitespace().collect();
        let cmd = parts[0].to_lowercase();

        match cmd.as_str() {
            "help" => {
                self.print("=== QuatShell Komande ===", egui::Color32::YELLOW);
                self.print("  help          - Prikazuje ovaj meni", egui::Color32::LIGHT_GRAY);
                self.print("  clear         - Briše ekran terminala", egui::Color32::LIGHT_GRAY);
                self.print("  sysinfo       - Prikazuje stanje CPU registara", egui::Color32::LIGHT_GRAY);
                self.print("  quat <0-255>  - Prevodi dekadni broj u [Q3 Q2 Q1 Q0] format", egui::Color32::LIGHT_GRAY);
                self.print("  mem <offset>  - Čita 8 QuatBajtova iz RAM memorije", egui::Color32::LIGHT_GRAY);
                self.print("  step          - Izvršava 1 takt kerna", egui::Color32::LIGHT_GRAY);
                self.print("  echo <tekst>  - Ispisuje tekst u konzoli", egui::Color32::LIGHT_GRAY);
                self.print("  matrix        - Simulacija Quat-Matrix toka podataka", egui::Color32::LIGHT_GREEN);
            }

            "clear" => {
                self.history.clear();
            }

            "sysinfo" => {
                let q_a = kernel.reg_a.to_quats();
                let q_b = kernel.reg_b.to_quats();
                self.print(format!("PC (Program Counter): {:02X}", kernel.pc), egui::Color32::LIGHT_BLUE);
                self.print(format!("RegA: {:03} | Quats: [{}{}{}{}]", kernel.reg_a.0, q_a[0], q_a[1], q_a[2], q_a[3]), egui::Color32::LIGHT_BLUE);
                self.print(format!("RegB: {:03} | Quats: [{}{}{}{}]", kernel.reg_b.0, q_b[0], q_b[1], q_b[2], q_b[3]), egui::Color32::LIGHT_BLUE);
            }

            "quat" => {
                if parts.len() > 1 {
                    if let Ok(val) = parts[1].parse::<u8>() {
                        let q3 = (val >> 6) & 3;
                        let q2 = (val >> 4) & 3;
                        let q1 = (val >> 2) & 3;
                        let q0 = val & 3;
                        self.print(format!("Dekadno: {} -> Kvatarni format: [{}] [{}] [{}] [{}]", val, q3, q2, q1, q0), egui::Color32::GREEN);
                    } else {
                        self.print("Greška: Unesite broj između 0 i 255.", egui::Color32::RED);
                    }
                } else {
                    self.print("Upotreba: quat <0-255>", egui::Color32::YELLOW);
                }
            }

            "mem" => {
                let offset = parts.get(1).and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
                self.print(format!("RAM Ispis od adrese {:02X}:", offset), egui::Color32::YELLOW);
                for i in offset..(offset + 8).min(256) {
                    let q = kernel.ram[i].to_quats();
                    self.print(format!("  [{:02X}]: RAW={:03} | Quats={}{}{}{}", i, kernel.ram[i].0, q[0], q[1], q[2], q[3]), egui::Color32::LIGHT_GRAY);
                }
            }

            "step" => {
                kernel.step();
                self.print("Izvršen 1 takt kerna.", egui::Color32::GREEN);
            }

            "echo" => {
                let msg = parts[1..].join(" ");
                self.print(msg, egui::Color32::LIGHT_GRAY);
            }

            "matrix" => {
                self.print("3102 0123 3301 1230 0031 2103", egui::Color32::GREEN);
                self.print("0012 3321 1023 2210 3102 0321", egui::Color32::GREEN);
                self.print("Quat-Stream Sinhronizovan.", egui::Color32::GREEN);
            }

            _ => {
                self.print(format!("Nepoznata komanda: '{}'. Kucajte 'help'.", cmd), egui::Color32::RED);
            }
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, kernel: &mut QuatKernel) {
        ui.heading("🖥️ QuatOS Terminal (QuatShell)");
        ui.separator();

        // Crna retro pozadina za konzolu
        egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
            egui::ScrollArea::vertical().max_height(400.0).stick_to_bottom(true).show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                for line in &self.history {
                    ui.label(egui::RichText::new(&line.text).monospace().color(line.color));
                }
            });
        });

        ui.add_space(5.0);

        // Komandna linija na dnu
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("quat@kernel:~ $").monospace().color(egui::Color32::GREEN));
            let response = ui.text_edit_singleline(&mut self.input_buffer);

            if (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) || ui.button("Pošalji ⏎").clicked() {
                self.execute(kernel);
                response.request_focus();
            }
        });
    }
}