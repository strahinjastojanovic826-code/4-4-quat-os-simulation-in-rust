use eframe::egui;
use crate::kernel::{QuatKernel, QuatByte};
use crate::quat_net::QuatNET;
use crate::quat_pic::QuatPIC;
use crate::QuatDisk;
use crate::QuatVGA;
use crate::QuatDSP;

pub struct QuatDOS {
    pub history: Vec<(String, egui::Color32)>, // (Tekst, Boja)
    pub input_buffer: String,
    pub prompt: String,
    pub command_history: Vec<String>,
    pub history_index: usize,

    // Dodaj ova tri polja:
    pub disk: QuatDisk,
    pub vga: QuatVGA,     // Ako ti se VGA modul u projektu zove QuatGPU, upiši QuatGPU
    pub audio: QuatDSP,   // Ako ti se audio modul zove ChiptuneSource ili QuatAudio, upiši taj tip

}

//pazi u pub fn new
//Dirni nesto sjebaces ne pitaj kakao znam

impl QuatDOS {
    pub fn new() -> Self {
        let mut dos = Self {
            history: Vec::new(),
            input_buffer: String::new(),
            prompt: "C:\\QUAT>".to_string(),
            command_history: Vec::new(),
            history_index: 0,

            // Dodaj inicijalizaciju novih polja:
            disk: QuatDisk::new(),
            vga: QuatVGA::new(),
            audio: QuatDSP::new(),

        };

        // Welcome screen poruka
        dos.print_color("QuatDOS Version 1.00 (C) 2026 QuatEngine Systems", egui::Color32::LIGHT_GREEN);
        dos.print_color("Base RAM: 256 Bytes | Base Arch: 4^4 Quat Architecture", egui::Color32::GRAY);
        dos.print_color("Kucaj 'HELP' ili '?' za listu komandi.\n", egui::Color32::YELLOW);
        dos
    }

    pub fn print(&mut self, text: &str) {
        self.print_color(text, egui::Color32::LIGHT_GRAY);
    }

    pub fn print_color(&mut self, text: &str, color: egui::Color32) {
        for line in text.lines() {
            self.history.push((line.to_string(), color));
        }
        // Održavamo bafer na max 200 linija radi performansi
        if self.history.len() > 200 {
            self.history.drain(0..self.history.len() - 200);
        }
    }

    pub fn execute_command(
        &mut self,
        cmd_raw: &str,
        kernel: &mut QuatKernel,
        net: &mut QuatNET,
        pic: &mut QuatPIC,
        

    ) {
        let trimmed = cmd_raw.trim();
        if trimmed.is_empty() {
            return;
        }

        // Zabeleži komandu u log i istoriju
        self.print_color(&format!("{} {}", self.prompt, trimmed), egui::Color32::WHITE);
        self.command_history.push(trimmed.to_string());
        self.history_index = self.command_history.len();

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        let command = parts[0].to_uppercase();

        match command.as_str() {
            "HELP" | "?" => {
                self.print_color("=== QUATDOS KOMANDE ===", egui::Color32::GOLD);
                self.print("  CLS             - Čisti ekran terminala");
                self.print("  VER             - Verzija sistema");
                self.print("  MEM             - Status RAM-a i CPU registara");
                self.print("  PEEK <hex_addr> - Čitanje bajta iz memorije (npr: PEEK 0x70)");
                self.print("  POKE <addr> <val> - Upis u memoriju (npr: POKE 0x10 0xFF)");
                self.print("  RUN <hex_addr>  - Skok na adresu i start CPU-a");
                self.print("  HALT            - Pauziranje CPU-a");
                self.print("  NET <tekst>     - Slanje teksta u QuatNET modem");
                self.print("  IRQ <0-3>       - Test okidač hardverskog prekida");
            }

            "CLS" | "CLEAR" => {
                self.history.clear();
            }

            "BASIC" | "QBASIC" => {
                self.print_color("🔤 Pokrećem QuatBASIC Interpreter v1.0...", egui::Color32::LIGHT_BLUE);
                self.print_color("Prebacujem na QuatBASIC tab...", egui::Color32::GREEN);
               // Ovde možete promeniti aktivni tab u App strukturi na "QuatBASIC"
             }

            "VER" => {
                self.print_color("QuatDOS v1.00 [Kernel v4.2 - Rust Engine]", egui::Color32::LIGHT_BLUE);
            }

            "MEM" => {
    self.print_color("--- CPU & MEMORY STATUS ---", egui::Color32::LIGHT_GREEN);
    self.print_color(&format!("PC (Program Counter): 0x{:02X}", kernel.pc), egui::Color32::LIGHT_GREEN);
    self.print_color(&format!("ACC (Akumulator):     0x{:02X}", kernel.reg_a.0), egui::Color32::LIGHT_GREEN);

    let z_flag = if kernel.reg_a.0 == 0 { 1 } else { 0 };
    let c_flag = 0;
    self.print_color(&format!("FLAGS (Zero/Carry):   Z:{} C:{}", z_flag, c_flag), egui::Color32::LIGHT_GREEN);

    let non_zero_count = kernel.ram.iter().filter(|b| b.0 != 0).count();
    self.print_color(&format!("RAM Upotreba:          {}/256 bajtova zauzeto", non_zero_count), egui::Color32::LIGHT_GREEN);
}

            "PEEK" => {
                if parts.len() < 2 {
                    self.print_color("Sintaksa: PEEK <adresa>", egui::Color32::LIGHT_RED);
                } else if let Ok(addr) = parse_hex_or_dec(parts[1]) {
                    if addr < 256 {
                        let val = kernel.ram[addr].0;
                        self.print_color(&format!("RAM[0x{:02X}] = 0x{:02X} (dec: {}, bin: 0b{:08b})", addr, val, val, val), egui::Color32::GREEN);
                    } else {
                        self.print_color("Greška: Adresa van opsega (0x00-0xFF)", egui::Color32::LIGHT_RED);
                    }
                } else {
                    self.print_color("Greška: Nevaljana adresa!", egui::Color32::LIGHT_RED);
                }
            }

            "POKE" => {
                if parts.len() < 3 {
                    self.print_color("Sintaksa: POKE <adresa> <vrednost>", egui::Color32::LIGHT_RED);
                } else {
                    let addr_res = parse_hex_or_dec(parts[1]);
                    let val_res = parse_hex_or_dec(parts[2]);

                    match (addr_res, val_res) {
                        (Ok(addr), Ok(val)) if addr < 256 && val <= 255 => {
                            kernel.ram[addr] = QuatByte(val as u8);
                            self.print_color(&format!("UPIS: RAM[0x{:02X}] <- 0x{:02X}", addr, val), egui::Color32::GREEN);
                        }
                        _ => self.print_color("Greška: Nevažeći parametri za POKE!", egui::Color32::LIGHT_RED),
                    }
                }
            }

            "RUN" => {
                if parts.len() >= 2 {
                    if let Ok(addr) = parse_hex_or_dec(parts[1]) {
                        kernel.pc = addr as u8;
                        kernel.is_running = true;
                        self.print_color(&format!("🚀 CPU Pokrenut sa adrese 0x{:02X}", addr), egui::Color32::GREEN);
                    }
                } else {
                    kernel.is_running = true;
                    self.print_color("🚀 CPU Nastavlja rad...", egui::Color32::GREEN);
                }
            }

            "HALT" | "STOP" => {
                kernel.is_running = false;
                self.print_color("⏹️ CPU Zaustavljen.", egui::Color32::YELLOW);
            }

            "NET" => {
                if parts.len() > 1 {
                    let msg = parts[1..].join(" ");
                    net.send_string(&format!("{}\r\n", msg));
                    self.print_color(&format!("[NET OUT]: {}", msg), egui::Color32::from_rgb(0, 255, 255));
                } else {
                    self.print_color("Sintaksa: NET <poruka za slanje>", egui::Color32::LIGHT_RED);
                }
            }

            "IRQ" => {
                if parts.len() >= 2 {
                    if let Ok(irq) = parts[1].parse::<u8>() {
                        if irq <= 3 {
                            pic.trigger_irq(irq, kernel);
                            self.print_color(&format!("⚡ Trigorvan IRQ{}", irq), egui::Color32::YELLOW);
                        } else {
                            self.print_color("IRQ mora biti između 0 i 3", egui::Color32::LIGHT_RED);
                        }
                    }
                }
            }

            "DIR" => {
    if parts.len() >= 2 && parts[1].to_uppercase() == "A:" {
        if let Some(ref img) = self.disk.image {
            self.print_color(&format!(" Volume in drive A is {}", self.disk.label), egui::Color32::WHITE);
            self.print(" Directory of A:\\");
            self.print("");
            self.print("COMMAND  COM        45,056 04-09-95  12:00p");
            self.print("AUTOEXEC BAT           128 04-09-95  12:01p");
            self.print("CONFIG   SYS           256 04-09-95  12:01p");
            self.print_color("         3 File(s)     45,440 bytes", egui::Color32::LIGHT_GRAY);
            self.print_color("                  1,429,120 bytes free", egui::Color32::LIGHT_GRAY);
        } else {
            self.print_color("Not ready reading drive A: Abort, Retry, Fail?", egui::Color32::RED);
        }
        } else {
            self.print("Directory of C:\\ ...");
    }
}
         "FORMAT" => {
        if parts.len() >= 2 && parts[1].to_uppercase() == "A:" {
        self.disk.insert_blank_disk("FORMATTED");
        self.audio.beep(200.0, 500);
        self.print_color("Insert new diskette for drive A:", egui::Color32::YELLOW);
        self.print_color("Formatting 1.44M... Format complete.", egui::Color32::GREEN);
         } else {
        self.print("Upotreba: FORMAT A:");
     }
 }

        "MODE" => {
    if parts.len() >= 2 && parts[1] == "13H" {
        self.vga.demo_mode = 1; // Pokreni Plasma Demo kao pozdravni grafički mod
        self.print_color("🖥️ Prebačeno u Mode 13h (320x200x256). Grafički režim aktivan!", egui::Color32::GREEN);
    } else {
        self.print("Upotreba: MODE 13H");
    }
}
"DEMO" => {
    if parts.len() >= 2 {
        match parts[1] {
            "PLASMA" => self.vga.demo_mode = 1,
            "FIRE" => self.vga.demo_mode = 2,
            "STARS" => self.vga.demo_mode = 3,
            _ => self.vga.demo_mode = 0,
        }
        self.print_color(&format!("🎬 Pokrenut VGA demo: {}", parts[1]), egui::Color32::YELLOW);
    } else {
        self.print("Dostupni demosi: DEMO PLASMA, DEMO FIRE, DEMO STARS, DEMO OFF");
    }
}
"DRAW" => {
    // Primer komande: DRAW RECT x y w h color
    if parts.len() >= 6 {
        let x: usize = parts[1].parse().unwrap_or(10);
        let y: usize = parts[2].parse().unwrap_or(10);
        let w: usize = parts[3].parse().unwrap_or(50);
        let h: usize = parts[4].parse().unwrap_or(50);
        let color: u8 = parts[5].parse().unwrap_or(14); // 14 = Žuta
        self.vga.demo_mode = 0; // Isključi demo da vidimo nacrtano
        self.vga.draw_rect(x, y, w, h, color);
        self.print_color("🎨 Nacrtan pravougaonik u VRAM-u!", egui::Color32::LIGHT_BLUE);
    } else {
        self.print("Upotreba: DRAW x y w h color_index");
    }
}

       "BEEP" => {
    if parts.len() >= 3 {
        let freq = parts[1].parse::<f32>().unwrap_or(440.0);
        let dur = parts[2].parse::<u64>().unwrap_or(200);
        self.audio.beep(freq, dur);
        self.print_color(&format!("🔊 Beep: {}Hz ({}ms)", freq, dur), egui::Color32::LIGHT_GREEN);
    } else if parts.len() == 2 {
        let freq = parts[1].parse::<f32>().unwrap_or(440.0);
        self.audio.beep(freq, 200);
        self.print_color(&format!("🔊 Beep: {}Hz (200ms)", freq), egui::Color32::LIGHT_GREEN);
    } else {
        self.audio.beep(440.0, 200);
        self.print_color("🔊 Standard Beep: 440Hz", egui::Color32::LIGHT_GREEN);
    }
}

            _ => {
                self.print_color(&format!("Nepoznata komanda: '{}'. Kucaj 'HELP' za pomoć.", command), egui::Color32::LIGHT_RED);
            }
        }
    }

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        kernel: &mut QuatKernel,
        net: &mut QuatNET,
        pic: &mut QuatPIC,
    ) {
        ui.heading("💻 QuatDOS — Interactive CLI Shell");
        ui.separator();

        // Terminal Ekran Box (Retro tamna pozadina)
        egui::Frame::canvas(ui.style())
            .fill(egui::Color32::from_rgb(15, 15, 20))
            .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY))
            .show(ui, |ui| {
                ui.set_min_height(320.0);
                
                // Prikaz istorije konzole sa automatskim scroll-om na dno
                egui::ScrollArea::vertical()
                    .max_height(320.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
                        
                        for (line, color) in &self.history {
                            ui.colored_label(*color, line);
                        }
                    });
            });

        ui.add_space(8.0);

        // Input Linija za kucanje komandi
        ui.horizontal(|ui| {
            ui.colored_label(egui::Color32::GREEN, &self.prompt);

            let response = ui.add_sized(
                [ui.available_width() - 80.0, 24.0],
                egui::TextEdit::singleline(&mut self.input_buffer)
                    .text_color(egui::Color32::WHITE)
                    .hint_text("Kucaj komandu...")
            );

            // Fokusiraj input odmah po otvaranju
            if ui.memory(|m| m.focused()).is_none() {
                response.request_focus();
            }

            // Kretanje kroz istoriju komandi na Strelica Gore / Dole
            if response.has_focus() {
                ui.input(|i| {
                    if i.key_pressed(egui::Key::ArrowUp) && !self.command_history.is_empty() {
                        if self.history_index > 0 {
                            self.history_index -= 1;
                            self.input_buffer = self.command_history[self.history_index].clone();
                        }
                    }
                    if i.key_pressed(egui::Key::ArrowDown) {
                        if self.history_index < self.command_history.len() {
                            self.history_index += 1;
                            if self.history_index < self.command_history.len() {
                                self.input_buffer = self.command_history[self.history_index].clone();
                            } else {
                                self.input_buffer.clear();
                            }
                        }
                    }
                });
            }

            let enter_pressed = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            let izvrši_clicked = ui.button("Izvrši").clicked();

            if enter_pressed || izvrši_clicked {
                let cmd = self.input_buffer.clone();
                self.input_buffer.clear();
                self.execute_command(&cmd, kernel, net, pic);
                response.request_focus();
            }
        });
    }
}

// Pomoćna funkcija za parsiranje dekadnih ili heksadecimalnih ulaza (npr. "0x70" ili "112")
fn parse_hex_or_dec(s: &str) -> Result<usize, ()> {
    if let Some(stripped) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        usize::from_str_radix(stripped, 16).map_err(|_| ())
    } else {
        s.parse::<usize>().map_err(|_| ())
    }
}