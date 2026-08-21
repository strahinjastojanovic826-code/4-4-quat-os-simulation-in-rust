use eframe::egui;
use crate::kernel::{QuatKernel, QuatByte};

#[derive(Clone, Debug)]
pub struct QuatFile {
    pub name: String,
    pub start_block: u8,
    pub size: u8,
}

pub struct QuatFS {
    pub files: Vec<QuatFile>,
    pub new_filename: String,
    pub new_file_data: String,
    pub status_msg: String,
}

impl QuatFS {
    pub fn new() -> Self {
        let mut fs = Self {
            files: Vec::new(),
            new_filename: String::new(),
            new_file_data: String::new(),
            status_msg: "QuatFS inicijalizovan (Format: 4^4 Block-Alloc)".to_string(),
        };

        // Fabrički fajlovi na QuatFS-u
        fs.files.push(QuatFile { name: "boot.sys".to_string(), start_block: 0x20, size: 16 });
        fs.files.push(QuatFile { name: "logo.art".to_string(), start_block: 0x30, size: 32 });
        fs.files.push(QuatFile { name: "hello.qasm".to_string(), start_block: 0x50, size: 8 });

        fs
    }

    // Formatiranje diska — brisanje Inode tabele i bloka sa podacima u RAM-u
    pub fn format_disk(&mut self, kernel: &mut QuatKernel) {
        self.files.clear();
        for i in 0x20..256 {
            kernel.ram[i] = QuatByte(0);
        }
        self.status_msg = "Disk uspešno formatiran! Svi sektori su vraćeni na 0000_4.".to_string();
    }

    // Upisivanje fajla u RAM (Data Blokove)
    pub fn create_file(&mut self, kernel: &mut QuatKernel) {
        let name = self.new_filename.trim().to_string();
        if name.is_empty() {
            self.status_msg = "Greška: Ime fajla ne može biti prazno!".to_string();
            return;
        }

        let bytes = self.new_file_data.as_bytes();
        let size = bytes.len() as u8;

        if size > 32 {
            self.status_msg = "Greška: Fajl ne može biti veći od 32 QuatBajta!".to_string();
            return;
        }

        // Tražemo slobodan blok (od adrese 0x60 pa naviše)
        let last_end = self.files.iter().map(|f| f.start_block + f.size).max().unwrap_or(0x60);
        let start_block = last_end;

        if (start_block as usize + size as usize) > 255 {
            self.status_msg = "Greška: Nema dovoljno prostora na QuatFS disku!".to_string();
            return;
        }

        // Upiši podatke direktno u RAM memoriju diska
        for (idx, &b) in bytes.iter().enumerate() {
            kernel.ram[start_block as usize + idx] = QuatByte(b);
        }

        self.files.push(QuatFile { name, start_block, size });
        self.status_msg = format!("Fajl zapisan od bloka 0x{:02X} (Veličina: {} QB)", start_block, size);
        self.new_filename.clear();
        self.new_file_data.clear();
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, kernel: &mut QuatKernel) {
        ui.heading("💽 QuatFS - Virtuelni Fajl Sistem ($4^4$ Architecture)");
        ui.separator();

        // 1. Pregled zauzeća diska
        let total_used: u16 = self.files.iter().map(|f| f.size as u16).sum();
        let free_space = 224 - total_used; // 224 B od ukupno 256 (32 su rezervisana za MBR)

        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.strong(format!("Iskorišćeno prostora: {} / 224 QB", total_used));
                ui.separator();
                ui.label(format!("Slobodno: {} QB", free_space));
            });
            ui.add(egui::ProgressBar::new(total_used as f32 / 224.0).text(format!("{:.1}%", (total_used as f32 / 224.0) * 100.0)));
        });

        ui.add_space(10.0);

        ui.columns(2, |cols| {
            // Leva kolona: Inode Tabela & Upravljanje Fajlovima
            cols[0].vertical(|ui| {
                ui.strong("📁 Inode Tabela & Fajlovi:");
                ui.add_space(5.0);

                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    egui::Grid::new("fs_files_grid").striped(true).min_col_width(70.0).show(ui, |ui| {
                        ui.strong("Ime Fajla");
                        ui.strong("Sektor");
                        ui.strong("Veličina");
                        ui.strong("Akcija");
                        ui.end_row();

                        let mut to_delete = None;
                        for (idx, file) in self.files.iter().enumerate() {
                            ui.label(&file.name);
                            ui.label(format!("0x{:02X}", file.start_block));
                            ui.label(format!("{} QB", file.size));

                            if ui.small_button("🗑️ Obrši").clicked() {
                                to_delete = Some(idx);
                            }
                            ui.end_row();
                        }

                        if let Some(idx) = to_delete {
                            self.files.remove(idx);
                            self.status_msg = "Fajl uklonjen iz Inode tabele.".to_string();
                        }
                    });
                });

                ui.add_space(10.0);
                if ui.button("⚠️ Formatiraj QuatFS Disk").clicked() {
                    self.format_disk(kernel);
                }
            });

            // Desna kolona: Novi Fajl Form
            cols[1].vertical(|ui| {
                ui.strong("✏️ Zapiši Novi Fajl na Disk:");
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("Ime fajla:");
                    ui.text_edit_singleline(&mut self.new_filename);
                });

                ui.add_space(5.0);
                ui.label("Sadržaj (Tekst ili Hex bajtovi):");
                ui.add(egui::TextEdit::multiline(&mut self.new_file_data).desired_rows(4));

                ui.add_space(5.0);
                if ui.button("💾 Zapiši na Disk (RAM Sektor)").clicked() {
                    self.create_file(kernel);
                }
            });
        });

        ui.add_space(10.0);
        ui.separator();
        ui.monospace(format!("Status QuatFS: {}", self.status_msg));
    }
}