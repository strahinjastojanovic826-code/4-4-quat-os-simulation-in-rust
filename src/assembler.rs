use eframe::egui;
use crate::kernel::{QuatByte, QuatKernel};
use std::collections::HashMap;

/// Predstavlja jednu tokenizovanu instrukciju ili labelu u prvom prolazu
#[derive(Debug, Clone)]
enum LineType {
    Label(String),
    Instruction {
        op: String,
        args: Vec<String>,
        line_num: usize,
    },
    Define {
        name: String,
        value: u8,
    },
}

pub struct Assembler {
    pub source_code: String,
    pub compiled_bytecode: Vec<QuatByte>,
    pub label_table: HashMap<String, u8>,
    pub error_log: Vec<String>,
    pub status_message: String,
    pub show_hex: bool,
    pub auto_load_to_ram: bool,
}

impl Assembler {
    pub fn new() -> Self {
        let sample_code = r#"// === QuatOS Demo Asemblerski Program ===
// Crta kvadrat i daje zvučni signal

%DEFINE COLOR_CYAN 11
%DEFINE BEEP_FREQ 440

start:
    CLS
    SET A, 10
    SET B, 15

draw_loop:
    DRAW A, B, 20, 20, COLOR_CYAN
    BEEP BEEP_FREQ, 150
    ADD A, 5
    JMP check_limit

check_limit:
    // Ako smo prešli poziciju 50, skoči na kraj
    // U realnom sistemu ovde ide CMP / JZ
    JMP end_prog

end_prog:
    BEEP 880, 300
    NOP
"#;

        Self {
            source_code: sample_code.to_string(),
            compiled_bytecode: Vec::new(),
            label_table: HashMap::new(),
            error_log: Vec::new(),
            status_message: "Asembler spreman.".to_string(),
            show_hex: false,
            auto_load_to_ram: false,
        }
    }

    /// Glavni dvoprolazni (2-pass) kompilator
    pub fn compile(&mut self, source: &str) -> Result<Vec<QuatByte>, String> {
        self.error_log.clear();
        self.label_table.clear();

        let mut parsed_lines: Vec<LineType> = Vec::new();
        let mut defines: HashMap<String, u8> = HashMap::new();
        let mut current_address: u8 = 0;

        // ==========================================
        // PROLAZ 1: Tokenizacija, Makroi & Labele
        // ==========================================
        for (idx, line_raw) in source.lines().enumerate() {
            let line_num = idx + 1;
            
            // Odbaci komentare (podržava // i ;)
            let line_no_comment = line_raw
                .split("//")
                .next()
                .unwrap_or("")
                .split(';')
                .next()
                .unwrap_or("")
                .trim();

            if line_no_comment.is_empty() {
                continue;
            }

            // Obrada direktive %DEFINE name val
            if line_no_comment.starts_with("%DEFINE") || line_no_comment.starts_with("%define") {
                let parts: Vec<&str> = line_no_comment.split_whitespace().collect();
                if parts.len() == 3 {
                    let name = parts[1].to_uppercase();
                    if let Ok(val) = parts[2].parse::<u8>() {
                        defines.insert(name, val);
                    } else {
                        self.error_log.push(format!("Linija {}: Nevaljala vrednost za %DEFINE", line_num));
                    }
                } else {
                    self.error_log.push(format!("Linija {}: Sintaksa %DEFINE zahteva ime i u8 vrednost", line_num));
                }
                continue;
            }

            // Provera da li je linija Labela (npr. "start:")
            if line_no_comment.ends_with(':') {
                let label_name = line_no_comment.trim_end_matches(':').trim().to_uppercase();
                if label_name.is_empty() {
                    self.error_log.push(format!("Linija {}: Prazno ime labele", line_num));
                } else {
                    self.label_table.insert(label_name.clone(), current_address);
                    parsed_lines.push(LineType::Label(label_name));
                }
                continue;
            }

            // Inače je regularna instrukcija
            let parts: Vec<&str> = line_no_comment.split_whitespace().collect();
            let op = parts[0].to_uppercase();
            let args: Vec<String> = parts[1..]
                .iter()
                .map(|s| s.trim_matches(',').trim().to_string())
                .collect();

            // Proračun veličine instrukcije u bajtovima radi mapiranja adresa labela
            let instr_len = match op.as_str() {
                "NOP" | "CLS" | "RET" | "HALT" => 1,
                "SET" | "BEEP" | "JMP" | "JZ" | "JNZ" | "CALL" | "PUSH" | "POP" => 2,
                "ADD" | "SUB" => 1,
                "DRAW" => if args.len() >= 4 { 6 } else { 1 },
                _ => 1,
            };

            parsed_lines.push(LineType::Instruction { op, args, line_num });
            current_address = current_address.saturating_add(instr_len);
        }

        if !self.error_log.is_empty() {
            return Err("Greške u prvom prolazu kompilacije.".to_string());
        }

        // ==========================================
        // PROLAZ 2: Generisanje Bajtkoda i Razrešavanje JMP/Labela
        // ==========================================
        let mut bytecode: Vec<QuatByte> = Vec::new();

        for item in parsed_lines {
            if let LineType::Instruction { op, args, line_num } = item {
                // Pomoćna funkcija za evaluaciju argumenata (Broj, %DEFINE ili Labela)
                let parse_val = |arg: &String, defs: &HashMap<String, u8>, labels: &HashMap<String, u8>| -> Result<u8, String> {
                    let arg_up = arg.to_uppercase();
                    if let Ok(v) = arg.parse::<u8>() {
                        Ok(v)
                    } else if let Some(&v) = defs.get(&arg_up) {
                        Ok(v)
                    } else if let Some(&addr) = labels.get(&arg_up) {
                        Ok(addr)
                    } else {
                        Err(format!("Linija {}: Nepoznat simbol, labela ili vrednost '{}'", line_num, arg))
                    }
                };

                match op.as_str() {
                    "NOP" => bytecode.push(QuatByte::new(0)),
                    "SET" => {
                        if args.len() < 2 {
                            self.error_log.push(format!("Linija {}: SET zahteva registar i vrednost", line_num));
                            continue;
                        }
                        let reg = match args[0].to_uppercase().as_str() {
                            "A" => 1,
                            "B" => 2,
                            _ => {
                                self.error_log.push(format!("Linija {}: Nepoznat registar '{}'", line_num, args[0]));
                                0
                            }
                        };
                        let val = match parse_val(&args[1], &defines, &self.label_table) {
                            Ok(v) => v,
                            Err(e) => { self.error_log.push(e); 0 }
                        };
                        bytecode.push(QuatByte::new(1)); // Opcode 1: SET
                        bytecode.push(QuatByte::new(reg));
                        bytecode.push(QuatByte::new(val));
                    }
                    "ADD" => bytecode.push(QuatByte::new(3)),
                    "SUB" => bytecode.push(QuatByte::new(4)),
                    "JMP" => {
                        if args.is_empty() {
                            self.error_log.push(format!("Linija {}: JMP zahteva ciljnu adresu ili labelu", line_num));
                            continue;
                        }
                        let addr = match parse_val(&args[0], &defines, &self.label_table) {
                            Ok(v) => v,
                            Err(e) => { self.error_log.push(e); 0 }
                        };
                        bytecode.push(QuatByte::new(5)); // Opcode 5: JMP
                        bytecode.push(QuatByte::new(addr));
                    }
                    "CLS" => bytecode.push(QuatByte::new(6)),
                    "DRAW" => {
                        if args.len() >= 5 {
                            // Fiksni DRAW X Y W H COLOR
                            let x = parse_val(&args[0], &defines, &self.label_table).unwrap_or(0);
                            let y = parse_val(&args[1], &defines, &self.label_table).unwrap_or(0);
                            let w = parse_val(&args[2], &defines, &self.label_table).unwrap_or(10);
                            let h = parse_val(&args[3], &defines, &self.label_table).unwrap_or(10);
                            let c = parse_val(&args[4], &defines, &self.label_table).unwrap_or(14);

                            bytecode.push(QuatByte::new(7)); // Opcode 7: Direct DRAW
                            bytecode.push(QuatByte::new(x));
                            bytecode.push(QuatByte::new(y));
                            bytecode.push(QuatByte::new(w));
                            bytecode.push(QuatByte::new(h));
                            bytecode.push(QuatByte::new(c));
                        } else {
                            // Register DRAW
                            bytecode.push(QuatByte::new(8));
                        }
                    }
                    "BEEP" => {
                        let freq = if !args.is_empty() { parse_val(&args[0], &defines, &self.label_table).unwrap_or(100) } else { 100 };
                        let dur = if args.len() > 1 { parse_val(&args[1], &defines, &self.label_table).unwrap_or(50) } else { 50 };
                        bytecode.push(QuatByte::new(9)); // Opcode 9: BEEP
                        bytecode.push(QuatByte::new(freq));
                        bytecode.push(QuatByte::new(dur));
                    }
                    "HALT" => bytecode.push(QuatByte::new(15)),
                    unknown => {
                        self.error_log.push(format!("Linija {}: Nepoznata komanda '{}'", line_num, unknown));
                    }
                }
            }
        }

        if !self.error_log.is_empty() {
            Err("Greške tokom generisanja bajtkoda.".to_string())
        } else {
            Ok(bytecode)
        }
    }

    /// Renders UI layout inside egui
    pub fn ui(&mut self, ui: &mut egui::Ui, kernel: &mut QuatKernel) {
        ui.heading("⚡ QuatOS Multi-Pass Assembler & IDE");
        ui.separator();

        // Gornji kontrolni panel sa komandnim dugmadima
        ui.horizontal(|ui| {
            if ui.button("🚀 Kompajliraj Kod").clicked() {
                match self.compile(&self.source_code.clone()) {
                    Ok(bc) => {
                        self.compiled_bytecode = bc;
                        self.status_message = format!("Uspeshno! Generisano {} QuatByte-ova.", self.compiled_bytecode.len());
                        if self.auto_load_to_ram {
                            // Učitavanje u VM RAM ako je opcija aktivna
                            for (i, byte) in self.compiled_bytecode.iter().enumerate() {
                                if i < kernel.ram.len() {
                                    kernel.ram[i] = *byte;
                                }
                            }
                            self.status_message.push_str(" Učitano direktno u RAM!");
                        }
                    }
                    Err(err) => {
                        self.status_message = format!("Kompilacija neuspešna: {}", err);
                    }
                }
            }

            if ui.button("📥 Učitaj u RAM").clicked() {
                if self.compiled_bytecode.is_empty() {
                    self.status_message = "Nema kompajliranog bajtkoda za učitavanje!".to_string();
                } else {
                    for (i, byte) in self.compiled_bytecode.iter().enumerate() {
                        if i < kernel.ram.len() {
                            kernel.ram[i] = *byte;
                        }
                    }
                    self.status_message = format!("Učitano {} bajtova na adresu 0x00 u RAM-u.", self.compiled_bytecode.len());
                }
            }

            ui.checkbox(&mut self.auto_load_to_ram, "Auto-load u RAM");
            ui.checkbox(&mut self.show_hex, "Prikaži Hex umesto Quat-a");

            if ui.button("🧹 Očisti").clicked() {
                self.source_code.clear();
                self.compiled_bytecode.clear();
                self.error_log.clear();
                self.status_message = "Sadržaj očišćen.".to_string();
            }
        });

        ui.separator();

        // Statusna traka
        ui.label(egui::RichText::new(&self.status_message).strong().color(
            if self.error_log.is_empty() { egui::Color32::GREEN } else { egui::Color32::RED }
        ));

        ui.add_space(5.0);

        // Dvo-kolonski raspored: Levo Editor, Desno Vizuelizacija Bajtkoda & Tabela Simbola
        ui.columns(2, |columns| {
            // KOLONA 1: Tekstualni Editor koda
            columns[0].vertical(|ui| {
                ui.label("📝 Izvorni Asemblerski Kod:");
                egui::ScrollArea::vertical()
                    .id_source("code_editor_scroll")
                    .max_height(400.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.source_code)
                                .font(egui::TextStyle::Monospace)
                                .code_editor()
                                .desired_width(f32::INFINITY)
                                .desired_rows(22),
                        );
                    });
            });

            // KOLONA 2: Generisani Bajtkod, Labele i Dnevnik Grešaka
            columns[1].vertical(|ui| {
                ui.label("⚙ Generisani Bajtkod & Tabela Simbola:");

                egui::CollapsingHeader::new("🔍 Tabela Labela (Simbola)").default_open(true).show(ui, |ui| {
                    if self.label_table.is_empty() {
                        ui.label("Nema definisanih labela.");
                    } else {
                        egui::Grid::new("label_grid").striped(true).show(ui, |ui| {
                            ui.label("Labela");
                            ui.label("Adresa u RAM-u");
                            ui.end_row();
                            for (lbl, addr) in &self.label_table {
                                ui.label(lbl);
                                ui.label(format!("0x{:02X} ({})", addr, addr));
                                ui.end_row();
                            }
                        });
                    }
                });

                ui.add_space(5.0);

                egui::CollapsingHeader::new("💾 Generisana Memorija (Bytecode)").default_open(true).show(ui, |ui| {
                    if self.compiled_bytecode.is_empty() {
                        ui.label("Bajtkod je prazan. Klikni na 'Kompajliraj Kod'.");
                    } else {
                        egui::ScrollArea::vertical()
                            .id_source("bytecode_scroll")
                            .max_height(200.0)
                            .show(ui, |ui| {
                                ui.horizontal_wrapped(|ui| {
                                    for (idx, byte) in self.compiled_bytecode.iter().enumerate() {
                                        let q = byte.to_quats();
                                        let display_str = if self.show_hex {
                                            format!("[{:02X}]", byte.0)
                                        } else {
                                            format!("[{}{}{}{}]", q[0], q[1], q[2], q[3])
                                        };

                                        ui.label(
                                            egui::RichText::new(format!("{:02X}:{}", idx, display_str))
                                                .monospace()
                                                .background_color(egui::Color32::from_gray(40)),
                                        );
                                    }
                                });
                            });
                    }
                });

                // Panel za prikaz grešaka pri kompilaciji
                if !self.error_log.is_empty() {
                    ui.add_space(5.0);
                    ui.label(egui::RichText::new("🚨 Greške pri kompilaciji:").color(egui::Color32::RED).strong());
                    egui::ScrollArea::vertical()
                        .id_source("error_scroll")
                        .max_height(100.0)
                        .show(ui, |ui| {
                            for err in &self.error_log {
                                ui.label(egui::RichText::new(err).color(egui::Color32::LIGHT_RED).monospace());
                            }
                        });
                }
            });
        });
    }
}