use eframe::egui;
use crate::kernel::{QuatKernel, QuatByte};

// Paleta od 4 boje (Baza 4)
pub const PALETTE: [egui::Color32; 4] = [
    egui::Color32::from_rgb(15, 20, 25),   // 00_4: Pozadina (Tamno siva/crna)
    egui::Color32::from_rgb(0, 110, 70),   // 01_4: Tamno zelena
    egui::Color32::from_rgb(0, 200, 100),  // 02_4: Svetlo zelena
    egui::Color32::from_rgb(180, 255, 120),// 03_4: Bright Phosphor (Žuto-zelena)
];

#[derive(Clone, Copy, Debug)]
pub struct HardwareSprite {
    pub x: u8,
    pub y: u8,
    pub tile_id: u8,
    pub flags: u8, // bit 0: Flip X, bit 1: Flip Y, bit 2: Transparent
}

pub struct QuatGPU {
    pub active_palette: [egui::Color32; 4],
    pub selected_tile: usize,
    pub sprites: [HardwareSprite; 4],
    pub show_grid: bool,
}

impl QuatGPU {
    pub fn new() -> Self {
        Self {
            active_palette: PALETTE,
            selected_tile: 0,
            sprites: [
                HardwareSprite { x: 2, y: 2, tile_id: 1, flags: 0 },
                HardwareSprite { x: 10, y: 4, tile_id: 2, flags: 0 },
                HardwareSprite { x: 6, y: 11, tile_id: 3, flags: 0 },
                HardwareSprite { x: 12, y: 12, tile_id: 0, flags: 0 },
            ],
            show_grid: true,
        }
    }

    // Renderovanje kompletnog $16 \times 16$ ekrana iz VRAM-a u RAM framebuffer (0x00..0xFF)
    pub fn render_frame(&self, kernel: &mut QuatKernel) {
        // 1. Sloj: Renderovanje Tilemap Pozadine ($4 \times 4$ pločice veličine $4 \times 4$ px)
        for tile_y in 0..4 {
            for tile_x in 0..4 {
                let tilemap_idx = tile_y * 4 + tile_x;
                let tile_id = kernel.ram[0x90 + tilemap_idx].0 % 16; // Čitanje iz Tilemap RAM-a

                // Crtanje $4 \times 4$ piksela izabrane pločice
                for py in 0..4 {
                    for px in 0..4 {
                        let screen_x = tile_x * 4 + px;
                        let screen_y = tile_y * 4 + py;
                        let frame_addr = screen_y * 16 + screen_x;

                        // Izračunavanje adrese u Pattern RAM-u (0xA0 pa naviše)
                        let pattern_addr = 0xA0 + (tile_id as usize * 4) + py;
                        let row_byte = kernel.ram[pattern_addr].0;

                        // Čitanje Baza-4 vrednosti piksela iz bajta
                        let color_idx = (row_byte >> ((3 - px) * 2)) & 0x03;
                        kernel.ram[frame_addr] = QuatByte(color_idx * 85); // Mape u jasnost (0..255)
                    }
                }
            }
        }

        // 2. Sloj: Renderovanje Hardverskih Sprajtova iz OAM-a preko pozadine
        for sprite in &self.sprites {
            let sx = sprite.x as usize;
            let sy = sprite.y as usize;
            let tile_id = sprite.tile_id as usize % 16;

            for py in 0..4 {
                for px in 0..4 {
                    let target_x = sx + px;
                    let target_y = sy + py;

                    if target_x < 16 && target_y < 16 {
                        let pattern_addr = 0xA0 + (tile_id * 4) + py;
                        let row_byte = kernel.ram[pattern_addr].0;
                        let color_idx = (row_byte >> ((3 - px) * 2)) & 0x03;

                        // Ako piksel nije providan (index 0), prekriva pozadinu
                        if color_idx > 0 {
                            let frame_addr = target_y * 16 + target_x;
                            kernel.ram[frame_addr] = QuatByte(color_idx * 85);
                        }
                    }
                }
            }
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, kernel: &mut QuatKernel) {
        ui.heading("🎨 QuatGPU - Hardware Sprite & Tile Engine ($16 \\times 16$)");
        ui.separator();

        ui.columns(2, |cols| {
            // Leva kolona: Visual VRAM Preview & CRT Composite
            cols[0].vertical(|ui| {
                ui.strong("📺 GPU Composite Framebuffer Output:");
                ui.add_space(5.0);

                // Dugme za OS kompoziciju kadra
                if ui.button("⚡ Renderuj GPU Kadar u VRAM").clicked() {
                    self.render_frame(kernel);
                }

                ui.add_space(10.0);

                // Prikaz kompozitnog piksel rastera pomoću Painter-a
                let display_size = 220.0;
                let (rect, _) = ui.allocate_exact_size(egui::vec2(display_size, display_size), egui::Sense::hover());
                let painter = ui.painter_at(rect);
                let pixel_size = display_size / 16.0;

                painter.rect_filled(rect, 4.0, self.active_palette[0]);

                for y in 0..16 {
                    for x in 0..16 {
                        let addr = y * 16 + x;
                        let val = kernel.ram[addr].0;
                        let col_idx = (val / 85) as usize % 4;

                        let px_rect = egui::Rect::from_min_size(
                            egui::pos2(rect.min.x + (x as f32) * pixel_size, rect.min.y + (y as f32) * pixel_size),
                            egui::vec2(pixel_size - 0.5, pixel_size - 0.5),
                        );
                        painter.rect_filled(px_rect, 1.0, self.active_palette[col_idx]);
                    }
                }

                if self.show_grid {
                    for i in 1..4 {
                        let pos = rect.min.x + (i as f32) * (display_size / 4.0);
                        painter.line_segment(
                            [egui::pos2(pos, rect.min.y), egui::pos2(pos, rect.max.y)],
                            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 40)),
                        );
                        let pos_y = rect.min.y + (i as f32) * (display_size / 4.0);
                        painter.line_segment(
                            [egui::pos2(rect.min.x, pos_y), egui::pos2(rect.max.x, pos_y)],
                            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 40)),
                        );
                    }
                }

                ui.checkbox(&mut self.show_grid, "Prikaži Tile Grid ($4 \\times 4$)");
            });

            // Desna kolona: OAM Controls & Pattern RAM Editor
            cols[1].vertical(|ui| {
                ui.strong("👾 OAM Hardverski Sprajtovi (Max 4):");
                ui.add_space(5.0);

                egui::Grid::new("oam_grid").striped(true).show(ui, |ui| {
                    ui.strong("ID");
                    ui.strong("X");
                    ui.strong("Y");
                    ui.strong("Tile ID");
                    ui.end_row();

                    for (idx, sprite) in self.sprites.iter_mut().enumerate() {
                        ui.label(format!("#{}", idx));
                        ui.add(egui::DragValue::new(&mut sprite.x).clamp_range(0..=12));
                        ui.add(egui::DragValue::new(&mut sprite.y).clamp_range(0..=12));
                        ui.add(egui::DragValue::new(&mut sprite.tile_id).clamp_range(0..=15));
                        ui.end_row();
                    }
                });

                ui.add_space(15.0);
                ui.strong("🖌️ Pattern RAM Editor ($4 \\times 4$ Pixels Tile):");
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("Pločica:");
                    ui.add(egui::Slider::new(&mut self.selected_tile, 0..=15).text("Tile ID"));
                });

                ui.add_space(5.0);

                // Izmena Baza-4 piksela direktno u Pattern RAM-u (0xA0 + Tile * 4)
                let base_pattern_addr = 0xA0 + (self.selected_tile * 4);
                
                egui::Grid::new("tile_editor_grid").spacing(egui::vec2(4.0, 4.0)).show(ui, |ui| {
                    for py in 0..4 {
                        let row_addr = base_pattern_addr + py;
                        let mut row_byte = kernel.ram[row_addr].0;

                        for px in 0..4 {
                            let shift = (3 - px) * 2;
                            let mut pixel_val = (row_byte >> shift) & 0x03;

                            let btn_color = self.active_palette[pixel_val as usize];
                            if ui.add(egui::Button::new("").fill(btn_color).min_size(egui::vec2(24.0, 24.0))).clicked() {
                                pixel_val = (pixel_val + 1) % 4; // Rotacija boje na klik (0 -> 1 -> 2 -> 3 -> 0)
                                row_byte = (row_byte & !(0x03 << shift)) | (pixel_val << shift);
                                kernel.ram[row_addr] = QuatByte(row_byte);
                            }
                        }
                        ui.end_row();
                    }
                });

                ui.add_space(5.0);
                ui.monospace(format!("VRAM Adresa Pločice: 0x{:02X}", base_pattern_addr));
            });
        });
    }
}