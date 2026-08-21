use eframe::egui;
use std::collections::VecDeque;
use crate::kernel::{QuatKernel, QuatByte};
use crate::quat_pic::{QuatPIC, IRQ_PAD}; // Može se mapirati na slobodan IRQ ili dodati IRQ_NET

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModemState {
    Disconnected,
    Dialing(u8), // brojač ciklusa konekcije
    Connected,
}

pub struct QuatPacket {
    pub sender_id: u8,
    pub receiver_id: u8,
    pub payload: Vec<u8>,
}

pub struct QuatNET {
    pub state: ModemState,
    pub baud_rate_ticks: u32,
    pub tick_counter: u32,
    
    // Baferi za prijem i slanje (Ring buffers)
    pub rx_buffer: VecDeque<u8>,
    pub tx_buffer: VecDeque<u8>,
    
    // UI Terminal & Statistika
    pub terminal_log: Vec<String>,
    pub input_text: String,
    pub total_bytes_sent: usize,
    pub total_bytes_received: usize,
    pub packet_loss_rate: f32, // 0.0 do 1.0 (šum na liniji)
    pub echo_mode: bool,        // Loopback test
}

impl QuatNET {
    pub fn new() -> Self {
        let mut net = Self {
            state: ModemState::Disconnected,
            baud_rate_ticks: 10, // Svakih 10 CPU ciklusa šalje 1 bajt
            tick_counter: 0,
            rx_buffer: VecDeque::with_capacity(64),
            tx_buffer: VecDeque::with_capacity(64),
            terminal_log: vec!["[QuatNET] Serial Interface v1.0 Ready.".to_string()],
            input_text: String::new(),
            total_bytes_sent: 0,
            total_bytes_received: 0,
            packet_loss_rate: 0.0,
            echo_mode: true, // Defaultno u loopback modu radi testiranja
        };
        net
    }

    // Poziva se u glavnoj petlji CPU ciklusa
    pub fn tick(&mut self, kernel: &mut QuatKernel, pic: &mut QuatPIC) {
        // Obrađujemo komande upisane u MMIO [0x82]
        let cmd = kernel.ram[0x82].0;
        if cmd != 0 {
            self.execute_command(cmd, kernel);
            kernel.ram[0x82] = QuatByte(0); // Resetuj komandu
        }

        // Animacija spajanja modema
        if let ModemState::Dialing(ref mut count) = self.state {
            *count += 1;
            if *count > 50 {
                self.state = ModemState::Connected;
                self.log("CONNECT 9600 BAUD / QuatNET Server Online.");
            }
            return;
        }

        if self.state != ModemState::Connected {
            return;
        }

        self.tick_counter += 1;
        if self.tick_counter >= self.baud_rate_ticks {
            self.tick_counter = 0;

            // 1. Slanje bajtova iz TX bafera
            if let Some(byte_to_send) = self.tx_buffer.pop_front() {
                self.total_bytes_sent += 1;
                
                // Ako je uklačen loopback/echo mod, odma vraćamo u RX
                if self.echo_mode {
                    // Simulacija šuma/gubitka paketa
                    if rand_simple() > self.packet_loss_rate {
                        self.push_rx(byte_to_send, kernel, pic);
                    } else {
                        self.log(&format!("⚠️ PARITY ERROR: Byte 0x{:02X} lost!", byte_to_send));
                    }
                }
            }
        }

        // Ažuriramo MMIO Registar statusa [0x81]
        let mut status = 0u8;
        if !self.rx_buffer.is_empty() { status |= 1 << 0; } // RX_READY
        if self.tx_buffer.len() < 64 { status |= 1 << 1; }   // TX_EMPTY
        if self.state == ModemState::Connected { status |= 1 << 2; } // CARRIER
        kernel.ram[0x81] = QuatByte(status);

        // Ako u MMIO [0x80] ima upisanih podataka od strane CPU-a
        let mmio_data = kernel.ram[0x80].0;
        if mmio_data != 0 && self.tx_buffer.len() < 64 {
            self.tx_buffer.push_back(mmio_data);
            kernel.ram[0x80] = QuatByte(0); // Očisti nakon preuzimanja
        }
    }

    fn push_rx(&mut self, byte: u8, kernel: &mut QuatKernel, pic: &mut QuatPIC) {
        if self.rx_buffer.len() < 64 {
            self.rx_buffer.push_back(byte);
            self.total_bytes_received += 1;

            // Postavljamo bajt direktno u MMIO [0x80] za CPU
            kernel.ram[0x80] = QuatByte(byte);

            // Okidamo IRQ prekid ako je omogućen (npr. IRQ3 ili custom)
            pic.trigger_irq(3, kernel);
        }
    }

    fn execute_command(&mut self, cmd: u8, _kernel: &mut QuatKernel) {
        match cmd {
            0x01 => {
                self.state = ModemState::Dialing(0);
                self.log("ATDT 555-QUAT... Dialing server...");
            }
            0x02 => {
                self.state = ModemState::Disconnected;
                self.log("ATH0... Line disconnected (NO CARRIER).");
            }
            0x03 => {
                self.rx_buffer.clear();
                self.tx_buffer.clear();
                self.log("Buffers flushed.");
            }
            _ => {}
        }
    }

    fn log(&mut self, msg: &str) {
        self.terminal_log.push(msg.to_string());
        if self.terminal_log.len() > 100 {
            self.terminal_log.remove(0);
        }
    }

    pub fn send_string(&mut self, text: &str) {
        for b in text.bytes() {
            if self.tx_buffer.len() < 64 {
                self.tx_buffer.push_back(b);
            }
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, kernel: &mut QuatKernel) {
        ui.heading("🌐 QuatNET — Serial Modem & Packet Simulator");
        ui.separator();

        ui.columns(2, |cols| {
            // Leva strana: Kontrole modema i linije
            cols[0].vertical(|ui| {
                ui.strong("🔌 Hardware & Modem Controls");
                ui.add_space(5.0);

                // Status Lampice
                ui.horizontal(|ui| {
                    ui.label("Status:");
                    match self.state {
                        ModemState::Disconnected => {
                            ui.colored_label(egui::Color32::RED, "🔴 DISCONNECTED");
                        }
                        ModemState::Dialing(_) => {
                            ui.colored_label(egui::Color32::YELLOW, "🟡 DIALING...");
                        }
                        ModemState::Connected => {
                            ui.colored_label(egui::Color32::GREEN, "🟢 ONLINE (CARRIER DETECT)");
                        }
                    }
                });

                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    if ui.button("📞 ATDT (Connect)").clicked() {
                        self.execute_command(0x01, kernel);
                    }
                    if ui.button("⏹️ ATH (Hangup)").clicked() {
                        self.execute_command(0x02, kernel);
                    }
                    if ui.button("🧹 Flush Bafer").clicked() {
                        self.execute_command(0x03, kernel);
                    }
                });

                ui.add_space(10.0);
                ui.separator();
                ui.strong("⚙️ Parametri Serijske Linije");
                
                ui.add(egui::Slider::new(&mut self.baud_rate_ticks, 1..=100).text("Ticks / Byte (Baud)"));
                ui.add(egui::Slider::new(&mut self.packet_loss_rate, 0.0..=0.5).text("Noise / Packet Loss"));
                ui.checkbox(&mut self.echo_mode, "Loopback Mode (Self-Echo)");

                ui.add_space(10.0);
                ui.separator();
                ui.strong("📊 Statistika Prenosa");
                ui.label(format!("Poslato bajtova: {} B", self.total_bytes_sent));
                ui.label(format!("Primljeno bajtova: {} B", self.total_bytes_received));
                ui.label(format!("TX Buffer: {}/64", self.tx_buffer.len()));
                ui.label(format!("RX Buffer: {}/64", self.rx_buffer.len()));

                ui.add_space(10.0);
                ui.monospace(format!("MMIO [0x80] DATA:   0x{:02X}", kernel.ram[0x80].0));
                ui.monospace(format!("MMIO [0x81] STATUS: 0b{:08b}", kernel.ram[0x81].0));
            });

            // Desna strana: Virtualni Terminal / Packet Monitor
            cols[1].vertical(|ui| {
                ui.strong("🖥️ Serial Terminal Monitor");
                ui.add_space(5.0);

                // Ispis logova u scrollabilni box
                egui::ScrollArea::vertical()
                    .max_height(250.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
                        for log in &self.terminal_log {
                            ui.label(log);
                        }
                    });

                ui.add_space(5.0);
                
                // Input polje za ručno slanje teksta kroz modem
                ui.horizontal(|ui| {
                    let response = ui.add_sized(
                        [ui.available_width() - 60.0, 25.0],
                        egui::TextEdit::singleline(&mut self.input_text).hint_text("Unesi komandu ili tekst...")
                    );

                    let send_clicked = ui.button("Šalji").clicked();
                    let enter_pressed = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                    if (send_clicked || enter_pressed) && !self.input_text.is_empty() {
                        let text_to_send = format!("{}\r\n", self.input_text);
                        self.log(&format!("> {}", self.input_text));
                        self.send_string(&text_to_send);
                        self.input_text.clear();
                    }
                });
            });
        });
    }
}

// Pomoćni generator pseudo-slučajnih brojeva bez eksternih biblioteka
fn rand_simple() -> f32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos();
    (nanos % 100) as f32 / 100.0
}