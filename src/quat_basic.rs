use eframe::egui;
use std::collections::{BTreeMap, HashMap};
use crate::quat_audio::QuatAudio;
use crate::quat_vga::QuatVGA;

pub struct QuatBASIC {
    pub lines: BTreeMap<usize, String>, // Broj linije -> Kod (npr. 10 -> "PRINT HI")
    pub input_line: String,             // Konzola za unos naredbi (npr. "RUN", "LIST", ili "10 PRINT 1")
    pub console_output: Vec<String>,     // Tekstualni izlaz iz BASIC programa
    pub variables: HashMap<String, f32>,// Tabela promenljivih (npr. A = 10.0)
    pub is_running: bool,
    pub status_msg: String,
}

impl QuatBASIC {
    pub fn new() -> Self {
        let mut basic = Self {
            lines: BTreeMap::new(),
            input_line: String::new(),
            console_output: Vec::new(),
            variables: HashMap::new(),
            is_running: false,
            status_msg: String::from("QuatBASIC v1.0 Ready."),
        };

        // Učitavamo klasični demo program pri pokretanju
        basic.load_demo_program();
        basic
    }

    /// Pre-loaded demo program (Beeper + VGA PSET Test)
    pub fn load_demo_program(&mut self) {
        self.lines.clear();
        self.lines.insert(10, "REM *** QuatBASIC Demo Program ***".into());
        self.lines.insert(20, "CLS".into());
        self.lines.insert(30, "PRINT \"Pokrecem QuatBASIC Hardverski Test...\"".into());
        self.lines.insert(40, "BEEP 440, 150".into());
        self.lines.insert(50, "BEEP 880, 200".into());
        self.lines.insert(60, "PRINT \"Crtam po QuatVGA VRAM-u...\"".into());
        self.lines.insert(70, "PSET 160, 100, 14".into()); // Žuta tačka na sredini
        self.lines.insert(80, "PRINT \"Gotovo! Ready.\"".into());
    }

    /// Pokretanje čitavog BASIC programa od prve linije
    pub fn run(&mut self, audio: &mut QuatAudio, vga: &mut QuatVGA) {
        self.console_output.clear();
        self.variables.clear();
        self.console_output.push("Running...".into());

        let line_numbers: Vec<usize> = self.lines.keys().cloned().collect();
        let mut pc_idx = 0;

        while pc_idx < line_numbers.len() {
            let line_num = line_numbers[pc_idx];
            if let Some(code) = self.lines.get(&line_num).cloned() {
                // Ako GOTO vrati novu liniju, skačemo na nju
                if let Some(goto_target) = self.execute_line(&code, audio, vga) {
                    if let Some(new_idx) = line_numbers.iter().position(|&l| l == goto_target) {
                        pc_idx = new_idx;
                        continue;
                    } else {
                        self.console_output.push(format!("ERR: Linija {} ne postoji!", goto_target));
                        break;
                    }
                }
            }
            pc_idx += 1;
        }

        self.console_output.push("OK".into());
    }

    /// Izvršavanje pojedinačne komande
    fn execute_line(&mut self, code: &str, audio: &mut QuatAudio, vga: &mut QuatVGA) -> Option<usize> {
        let trimmed = code.trim();
        if trimmed.is_empty() || trimmed.starts_with("REM") {
            return None;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        let cmd = parts[0].to_uppercase();

        match cmd.as_str() {
            "PRINT" => {
                if trimmed.len() > 6 {
                    let arg = trimmed[6..].trim();
                    if arg.starts_with('"') && arg.ends_with('"') {
                        // Ispis teksta u navodnicima
                        self.console_output.push(arg[1..arg.len()-1].to_string());
                    } else if let Some(val) = self.variables.get(arg) {
                        // Ispis vrednosti promenljive
                        self.console_output.push(format!("{}", val));
                    } else {
                        self.console_output.push(arg.to_string());
                    }
                }
            }
            "CLS" => {
                self.console_output.clear();
                vga.clear(0);
            }
            "BEEP" => {
                // Primer: BEEP 440, 200
                let args_str = trimmed[4..].replace(" ", "");
                let sub_parts: Vec<&str> = args_str.split(',').collect();
                let freq = sub_parts.get(0).and_then(|s| s.parse::<f32>().ok()).unwrap_or(440.0);
                let dur = sub_parts.get(1).and_then(|s| s.parse::<u64>().ok()).unwrap_or(200);
                audio.beep(freq, dur);
            }
            "PSET" => {
                // Primer: PSET x, y, color
                let args_str = trimmed[4..].replace(" ", "");
                let sub_parts: Vec<&str> = args_str.split(',').collect();
                if sub_parts.len() >= 3 {
                    let x = sub_parts[0].parse::<usize>().unwrap_or(0);
                    let y = sub_parts[1].parse::<usize>().unwrap_or(0);
                    let col = sub_parts[2].parse::<u8>().unwrap_or(15);
                    vga.put_pixel(x, y, col);
                }
            }
            "LET" => {
                // Primer: LET A = 10
                if parts.len() >= 4 && parts[2] == "=" {
                    let var_name = parts[1].to_string();
                    let val = parts[3].parse::<f32>().unwrap_or(0.0);
                    self.variables.insert(var_name, val);
                }
            }
            "GOTO" => {
                if parts.len() >= 2 {
                    if let Ok(target) = parts[1].parse::<usize>() {
                        return Some(target);
                    }
                }
            }
            _ => {
                // Podrška za direktnu dodelu bez LET (npr. A = 5)
                if parts.len() >= 3 && parts[1] == "=" {
                    let var_name = parts[0].to_string();
                    let val = parts[2].parse::<f32>().unwrap_or(0.0);
                    self.variables.insert(var_name, val);
                } else {
                    self.console_output.push(format!("Syntax error: {}", cmd));
                }
            }
        }

        None
    }

    /// Obrada unosa iz komandne linije (Direct Mode / Program Entry)
    pub fn process_input(&mut self, audio: &mut QuatAudio, vga: &mut QuatVGA) {
        let input = self.input_line.trim().to_string();
        if input.is_empty() { return; }

        self.input_line.clear();

        // Slučaj 1: Unos sa brojem linije (npr. "10 PRINT HI")
        let first_word = input.split_whitespace().next().unwrap_or("");
        if let Ok(line_num) = first_word.parse::<usize>() {
            let code_part = input[first_word.len()..].trim().to_string();
            if code_part.is_empty() {
                self.lines.remove(&line_num); // Prazna linija briše taj broj
            } else {
                self.lines.insert(line_num, code_part);
            }
            return;
        }

        // Slučaj 2: Direktne komande (RUN, LIST, NEW, CLS)
        match input.to_uppercase().as_str() {
            "RUN" => self.run(audio, vga),
            "LIST" => {
                self.console_output.push("--- PROGRAM LISTING ---".into());
                for (num, code) in &self.lines {
                    self.console_output.push(format!("{:04} {}", num, code));
                }
            }
            "NEW" => {
                self.lines.clear();
                self.variables.clear();
                self.console_output.clear();
                self.console_output.push("Ready.".into());
            }
            "CLS" => self.console_output.clear(),
            _ => {
                // Izvršavanje jednokratne komande u Direct Mode-u
                self.execute_line(&input, audio, vga);
            }
        }
    }

    /// Graphical UI — QBASIC / GW-BASIC Editor & Output Console
    pub fn ui(&mut self, ui: &mut egui::Ui, audio: &mut QuatAudio, vga: &mut QuatVGA) {
        ui.heading("🔤 QuatBASIC Interpreter (QBASIC Environment)");
        ui.label("GW-BASIC / QBASIC Kompatibilan | Povezan sa QuatAudio i QuatVGA");
        ui.separator();

        // Kontrolna traka
        ui.horizontal(|ui| {
            if ui.button("▶️ RUN Program").clicked() {
                self.run(audio, vga);
            }
            if ui.button("📋 LIST").clicked() {
                self.input_line = "LIST".into();
                self.process_input(audio, vga);
            }
            if ui.button("📄 NEW").clicked() {
                self.input_line = "NEW".into();
                self.process_input(audio, vga);
            }
            if ui.button("💾 Load Demo").clicked() {
                self.load_demo_program();
            }
        });

        ui.add_space(10.0);

        // Side-by-Side: Editor izvornog koda sa leve strane, Terminal izlaz sa desne
        ui.columns(2, |cols| {
            // Leva kolona: Izvorni Kod
            cols[0].group(|ui| {
                ui.label("📝 Izvorni Kod (Source Code):");
                egui::ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
                    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
                    if self.lines.is_empty() {
                        ui.label("(Prazno. Unesite linije koda u formatu: 10 PRINT \"HI\")");
                    } else {
                        for (num, code) in &self.lines {
                            ui.label(format!("{:04} {}", num, code));
                        }
                    }
                });
            });

            // Desna kolona: Terminal / Output Izlaz
            cols[1].group(|ui| {
                ui.label("🖥️ Izlazna Konzola (Output Console):");
                egui::ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
                    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
                    for out_line in &self.console_output {
                        ui.colored_label(egui::Color32::GREEN, out_line);
                    }
                });
            });
        });

        ui.add_space(10.0);

        // Komandna linija za unos BASIC naredbi
        ui.horizontal(|ui| {
            ui.label("BASIC>");
            let response = ui.add_sized(
                [ui.available_width() - 80.0, 24.0],
                egui::TextEdit::singleline(&mut self.input_line).hint_text("Npr: 10 PRINT \"Zdravo!\" ili RUN"),
            );

            if (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) || ui.button("Unesi").clicked() {
                self.process_input(audio, vga);
                response.request_focus();
            }
        });
    }
}