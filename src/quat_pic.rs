use eframe::egui;
use crate::kernel::{QuatKernel, QuatByte};

// IRQ Linije
pub const IRQ_TIMER: u8 = 0; // Bit 0 (0x01)
pub const IRQ_PAD: u8   = 1; // Bit 1 (0x02)
pub const IRQ_GPU: u8   = 2; // Bit 2 (0x04)
pub const IRQ_SYNTH: u8 = 3; // Bit 3 (0x08)

#[derive(Default, Clone, Copy)]
pub struct QuatPadState {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub a: bool,
    pub b: bool,
    pub select: bool,
    pub start: bool,
}

impl QuatPadState {
    // Pakuje stanje tastera u jedan QuatBajt za MMIO (0x70)
    pub fn to_byte(&self) -> u8 {
        let mut val = 0u8;
        if self.up     { val |= 1 << 0; }
        if self.down   { val |= 1 << 1; }
        if self.left   { val |= 1 << 2; }
        if self.right  { val |= 1 << 3; }
        if self.a      { val |= 1 << 4; }
        if self.b      { val |= 1 << 5; }
        if self.select { val |= 1 << 6; }
        if self.start  { val |= 1 << 7; }
        val
    }
}

pub struct QuatPIC {
    pub pad: QuatPadState,
    pub irq_enabled: u8,
    pub irq_pending: u8,
    pub vector_table: [u8; 4],
    pub master_enable: bool,
    pub last_serviced_irq: Option<u8>,
}

impl QuatPIC {
    pub fn new() -> Self {
        Self {
            pad: QuatPadState::default(),
            irq_enabled: 0b0000_1111, // Po defaultu omogućenih svih 4 IRQ
            irq_pending: 0,
            vector_table: [0x10, 0x14, 0x18, 0x1C], // Default adrese ISR rutina
            master_enable: true,
            last_serviced_irq: None,
        }
    }

    // Okida prekide sa hardverskih periferija
    pub fn trigger_irq(&mut self, irq: u8, kernel: &mut QuatKernel) {
        if irq > 3 { return; }
        let mask = 1 << irq;

        // Postavljamo pending bit u PIC-u i u RAM MMIO (0x72)
        self.irq_pending |= mask;
        kernel.ram[0x72] = QuatByte(self.irq_pending);
    }

    // Poziva se u svakom ciklusu CPU-a da proveri ima li neobrađenih prekida
    pub fn process_interrupts(&mut self, kernel: &mut QuatKernel) {
        if !self.master_enable { return; }

        let active_irqs = self.irq_pending & self.irq_enabled;
        if active_irqs == 0 { return; }

        // Određujemo koga obrađujemo prema prioritetu (IRQ0 ima najveći)
        for irq in 0..4 {
            let mask = 1 << irq;
            if (active_irqs & mask) != 0 {
                // Ako procesor nije već u ISR-u, preusmeravamo PC
                let isr_vector = self.vector_table[irq as usize];
                
                // Skidamo pending bit
                self.irq_pending &= !mask;
                kernel.ram[0x72] = QuatByte(self.irq_pending);

                // Pamćenje opslužene linije i skok CPU-a
                self.last_serviced_irq = Some(irq);
                kernel.pc = isr_vector as u8;
                break; // Obradi samo jedan po ciklusu
            }
        }
    }

    // Ažuriranje QuatPAD stanja sa tastature / UI-ja
    pub fn update_pad(&mut self, new_state: QuatPadState, kernel: &mut QuatKernel) {
        let old_byte = self.pad.to_byte();
        let new_byte = new_state.to_byte();

        if old_byte != new_byte {
            self.pad = new_state;
            kernel.ram[0x70] = QuatByte(new_byte); // Upis u MMIO

            // Na promenu stanja tastera okidamo IRQ_PAD (IRQ 1)
            self.trigger_irq(IRQ_PAD, kernel);
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, kernel: &mut QuatKernel) {
        ui.heading("🕹️ QuatPAD & Interrupt Controller (QuatPIC)");
        ui.separator();

        // Hvatanje fizicke tastature za QuatPAD (WASD / Strelice + J/K)
        ui.input(|i| {
            let mut state = self.pad;
            state.up     = i.key_down(egui::Key::ArrowUp) || i.key_down(egui::Key::W);
            state.down   = i.key_down(egui::Key::ArrowDown) || i.key_down(egui::Key::S);
            state.left   = i.key_down(egui::Key::ArrowLeft) || i.key_down(egui::Key::A);
            state.right  = i.key_down(egui::Key::ArrowRight) || i.key_down(egui::Key::D);
            state.a      = i.key_down(egui::Key::J);
            state.b      = i.key_down(egui::Key::K);
            state.select = i.key_down(egui::Key::U);
            state.start  = i.key_down(egui::Key::I);
            self.update_pad(state, kernel);
        });

        ui.columns(2, |cols| {
            // Leva kolona: QuatPAD Virtualni Kontroler
            cols[0].vertical(|ui| {
                ui.strong("🎮 Virtualni Gamepad (Tastatura: WASD / J K U I)");
                ui.add_space(10.0);

                let mut current_pad = self.pad;

                // Layout kontrolera
                ui.vertical_centered(|ui| {
                    // D-Pad Up
                    let up_btn = ui.add_sized([40.0, 30.0], egui::Button::new(if current_pad.up { "▲" } else { "△" }));
                    if up_btn.clicked() { current_pad.up = !current_pad.up; }

                    ui.horizontal(|ui| {
                        ui.add_space(20.0);
                        let left_btn = ui.add_sized([40.0, 30.0], egui::Button::new(if current_pad.left { "◄" } else { "◁" }));
                        if left_btn.clicked() { current_pad.left = !current_pad.left; }

                        ui.add_space(35.0);

                        let right_btn = ui.add_sized([40.0, 30.0], egui::Button::new(if current_pad.right { "►" } else { "▷" }));
                        if right_btn.clicked() { current_pad.right = !current_pad.right; }
                    });

                    // D-Pad Down
                    let down_btn = ui.add_sized([40.0, 30.0], egui::Button::new(if current_pad.down { "▼" } else { "▽" }));
                    if down_btn.clicked() { current_pad.down = !current_pad.down; }
                });

                ui.add_space(15.0);

                // Action Buttons
                ui.horizontal(|ui| {
                    ui.add_space(30.0);
                    let select_btn = ui.add_sized([45.0, 20.0], egui::Button::new("SELECT"));
                    if select_btn.clicked() { current_pad.select = !current_pad.select; }

                    let start_btn = ui.add_sized([45.0, 20.0], egui::Button::new("START"));
                    if start_btn.clicked() { current_pad.start = !current_pad.start; }

                    ui.add_space(20.0);
                    let btn_b = ui.add_sized([35.0, 35.0], egui::Button::new("B").fill(egui::Color32::from_rgb(180, 50, 50)));
                    if btn_b.clicked() { current_pad.b = !current_pad.b; }

                    let btn_a = ui.add_sized([35.0, 35.0], egui::Button::new("A").fill(egui::Color32::from_rgb(50, 180, 80)));
                    if btn_a.clicked() { current_pad.a = !current_pad.a; }
                });

                self.update_pad(current_pad, kernel);

                ui.add_space(15.0);
                ui.monospace(format!("MMIO [0x70] QuatPAD Registar: 0b{:08b}", self.pad.to_byte()));
            });

            // Desna kolona: QuatPIC Status & Direct Trigger Test
            cols[1].vertical(|ui| {
                ui.strong("⚡ Interrupt Controller Status (QuatPIC)");
                ui.add_space(5.0);

                ui.checkbox(&mut self.master_enable, "Master Interrupt Enable (MIE)");
                ui.add_space(5.0);

                egui::Grid::new("pic_irq_grid").striped(true).show(ui, |ui| {
                    ui.strong("IRQ");
                    ui.strong("Izvor");
                    ui.strong("Omogućen");
                    ui.strong("Pending");
                    ui.strong("Vector ISR");
                    ui.strong("Test");
                    ui.end_row();

                    let irq_names = ["IRQ0: Timer", "IRQ1: QuatPAD", "IRQ2: QuatGPU", "IRQ3: QuatSynth"];

                    for i in 0..4 {
                        let mask = 1 << i;
                        let mut enabled = (self.irq_enabled & mask) != 0;
                        let pending = (self.irq_pending & mask) != 0;

                        ui.label(format!("#{}", i));
                        ui.label(irq_names[i as usize]);

                        if ui.checkbox(&mut enabled, "").changed() {
                            if enabled { self.irq_enabled |= mask; } else { self.irq_enabled &= !mask; }
                            kernel.ram[0x71] = QuatByte(self.irq_enabled);
                        }

                        // LED indikator za pending IRQ
                        let led_color = if pending { egui::Color32::RED } else { egui::Color32::DARK_GRAY };
                        ui.colored_label(led_color, if pending { "● ACTIVE" } else { "○ idle" });

                        ui.add(egui::DragValue::new(&mut self.vector_table[i as usize]).clamp_range(0..=255));

                        if ui.button("⚡ Okidač").clicked() {
                            self.trigger_irq(i, kernel);
                        }
                        ui.end_row();
                    }
                });

                ui.add_space(15.0);
                ui.separator();
                
                if let Some(irq) = self.last_serviced_irq {
                    ui.colored_label(egui::Color32::GREEN, format!("Poslednji obrađen prekid: IRQ{} -> Jump na PC: 0x{:02X}", irq, self.vector_table[irq as usize]));
                } else {
                    ui.label("Nema aktivno obrađenih prekida.");
                }

                ui.add_space(10.0);
                if ui.button("🧹 Očisti sve Pending prekide").clicked() {
                    self.irq_pending = 0;
                    kernel.ram[0x72] = QuatByte(0);
                    self.last_serviced_irq = None;
                }
            });
        });
    }
}