use eframe::egui;
use crate::quat_audio::QuatAudio;

pub const SECTOR_SIZE: usize = 512;
pub const TOTAL_SECTORS: usize = 2880; // 1.44 MB
pub const FLOPPY_SIZE: usize = TOTAL_SECTORS * SECTOR_SIZE;

pub struct QuatDisk {
    pub image: Option<Vec<u8>>,      // Raw 1.44MB bafer
    pub label: String,               // Labela diskete (npr. "QUAT_OS_BOOT")
    pub current_track: u8,           // 0 - 79
    pub current_head: u8,            // 0 - 1
    pub motor_on: bool,
    pub write_protected: bool,
    pub activity_led: bool,
    pub led_timer: usize,
    pub active_sector: usize,        // LBA sektor koji se trenutno pregleda
    pub status_msg: String,
}

impl QuatDisk {
    pub fn new() -> Self {
        let mut disk = Self {
            image: None,
            label: String::new(),
            current_track: 0,
            current_head: 0,
            motor_on: false,
            write_protected: false,
            activity_led: false,
            led_timer: 0,
            active_sector: 0,
            status_msg: String::from("Drajv A: Prazan (Ubacite 3.5\" disketu)"),
        };

        // Automatski kreiramo i ubacujemo bootable disketu na startu
        disk.insert_blank_disk("QUAT_DOS_BOOT");
        disk
    }

    /// Ubacivanje prazne ili formatirane diskete
    pub fn insert_blank_disk(&mut self, label: &str) {
        let mut raw = vec![0u8; FLOPPY_SIZE];
        
        // Upisujemo lažni FAT12 Boot Sektor na Sektor 0 radi autentifičnosti
        let boot_sig = b"QUATBOOT1.0";
        raw[0.. boot_sig.len()].copy_from_slice(boot_sig);
        raw[510] = 0x55; // Standardni boot marker
        raw[511] = 0xAA;

        self.image = Some(raw);
        self.label = label.to_string();
        self.status_msg = format!("Disketa '{}' uspešno ubijena u Drajv A:", label);
    }

    /// Izbacivanje diskete (Eject)
    pub fn eject(&mut self) {
        self.image = None;
        self.label.clear();
        self.status_msg = String::from("Disketa izbačena.");
    }

    /// Pomeranje glave diska (Koračni motor) + Zvučni efekat!
    pub fn seek_track(&mut self, target_track: u8, audio: &mut QuatAudio) {
        let target = target_track.min(79);
        if target != self.current_track {
            let steps = (target as i16 - self.current_track as i16).abs() as u64;
            self.current_track = target;
            self.trigger_led(10);

            // Zvučni efekat koračnog motora: viši ton za brzi korak glave
            audio.beep(120.0 + (target as f32 * 5.0), 15 * steps.max(1));
        }
    }

    /// Čitanje LBA sektora
    pub fn write_sector(&mut self, lba: usize, data: &[u8], audio: &mut QuatAudio) {
    if self.image.is_none() {
        return;
    }

    // 1. Prvo pozovemo metode koje menjaju stanje drajva (seek, led)
    let track = (lba / (18 * 2)) as u8;
    self.seek_track(track, audio);
    self.trigger_led(20);

    // 2. Tek onda otvorimo mutabilnu referencu na sam bafer i upišemo
    let start = lba * SECTOR_SIZE;
    if let Some(ref mut img) = self.image {
        img[start..start + data.len()].copy_from_slice(data);
    }
}

    /// Upisivanje u LBA sektor
    pub fn read_sector(&mut self, lba: usize, audio: &mut QuatAudio) -> Option<&[u8]> {
    // 1. Provera da li uopšte imamo disk
    if self.image.is_none() {
        return None;
    }

    // 2. Prvo odradimo pomeranje glave i LED bez držanja reference na image
    let track = (lba / (18 * 2)) as u8;
    self.seek_track(track, audio);
    self.trigger_led(15);

    // 3. Tek sada bezbedno uzmemo referencu i vratimo isečak
    let start = lba * SECTOR_SIZE;
    let img = self.image.as_ref().unwrap();
    Some(&img[start..start + SECTOR_SIZE])
}

    fn trigger_led(&mut self, duration_ticks: usize) {
        self.activity_led = true;
        self.led_timer = duration_ticks;
    }

    pub fn tick(&mut self) {
        if self.led_timer > 0 {
            self.led_timer -= 1;
            if self.led_timer == 0 {
                self.activity_led = false;
            }
        }
    }

    /// Graphical UI — Kontrolna Tabla sa Prikazom Sektora
    pub fn ui(&mut self, ui: &mut egui::Ui, audio: &mut QuatAudio) {
        ui.heading("💾 QuatDisk — 3.5\" Virtual Floppy Drive (A:)");
        ui.label("Kapacitet: 1.44 MB (1,474,560 B) | 80 Traka | 2 Glave | 18 Sektora/Traka");
        ui.separator();

        // 1. Vizuelni Drajv Slot & LED Indikator
        ui.group(|ui| {
            ui.horizontal(|ui| {
                // LED Lampica
                let led_color = if self.activity_led {
                    egui::Color32::RED
                } else {
                    egui::Color32::from_rgb(40, 40, 40)
                };
                
                ui.colored_label(led_color, "🔴 DRIVE A:");
                
                if let Some(_) = &self.image {
                    ui.label(format!("STATUS: [ UBAČENA DISKETA: \"{}\" ]", self.label));
                } else {
                    ui.label("STATUS: [ PRAZAN SLOT ]");
                }
            });

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                if self.image.is_some() {
                    if ui.button("⏏️ Eject Disk").clicked() {
                        self.eject();
                        audio.beep(300.0, 100);
                    }
                } else {
                    if ui.button("📥 Ubaci Praznu Disketu").clicked() {
                        self.insert_blank_disk("NEW_FLOPPY");
                        audio.beep(600.0, 80);
                    }
                }

                ui.checkbox(&mut self.write_protected, "🔒 Write Protect Switch");
            });
        });

        ui.add_space(10.0);

        // 2. Status Mehaničke Glave Diska
        ui.group(|ui| {
            ui.label("⚙️ Mehanička Pozicija Glave Čitača");
            ui.horizontal(|ui| {
                ui.label(format!("Traka (Cylinder): {:02} / 79", self.current_track));
                ui.add_space(10.0);
                ui.label(format!("Glava (Side): {}", self.current_head));
            });

            // Slajder za ručni Seek Test
            let mut track_tmp = self.current_track;
            if ui.add(egui::Slider::new(&mut track_tmp, 0..=79).text("Traka")).changed() {
                self.seek_track(track_tmp, audio);
            }
        });

        ui.add_space(10.0);

        // 3. Inspektor Sektora (Hex & ASCII Viewer)
        ui.group(|ui| {
            ui.label("🔍 Hex & ASCII Inspektor Sektora");
            ui.horizontal(|ui| {
                ui.label("Sektor (LBA 0..2879):");
                ui.add(egui::DragValue::new(&mut self.active_sector).clamp_range(0..=2879));
                
                if ui.button("📖 Pročitaj Sektor").clicked() {
                    self.read_sector(self.active_sector, audio);
                }
            });

            ui.add_space(5.0);

            // Prikaz Sektora ako je disk u drajvu
            if let Some(ref img) = self.image {
                let start = self.active_sector * SECTOR_SIZE;
                let sector_bytes = &img[start..start + SECTOR_SIZE];

                egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
                    
                    for row in 0..16 {
                        let row_start = row * 16;
                        let row_bytes = &sector_bytes[row_start..row_start + 16];
                        
                        let hex_str: Vec<String> = row_bytes.iter().map(|b| format!("{:02X}", b)).collect();
                        let ascii_str: String = row_bytes.iter().map(|&b| if b >= 32 && b <= 126 { b as char } else { '.' }).collect();

                        ui.label(format!("{:04X}: {}  |{}|", row_start, hex_str.join(" "), ascii_str));
                    }
                });
            } else {
                ui.colored_label(egui::Color32::GRAY, "Nema medijuma za prikaz podataka.");
            }
        });

        ui.add_space(5.0);
        ui.label(&self.status_msg);
    }
}