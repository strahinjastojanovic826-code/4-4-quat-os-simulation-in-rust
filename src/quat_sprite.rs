use eframe::egui;
use crate::kernel::QuatKernel;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpriteTool {
    Pencil,
    Eraser,
    BucketFill,
    Picker,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActiveTab {
    SpriteEditor,
    TilemapEditor,
    ExportCode,
}

pub struct QuatSpriteStudio {
    pub active_tab: ActiveTab,
    pub current_tool: SpriteTool,
    pub selected_quat_color: u8, // 0, 1, 2, ili 3 (Kvatne boje)
    
    // Banka od 4 sprajta (svaki sprajt je 8x8 matrica kvatnih vrednosti 0-3)
    pub sprite_bank: [[[u8; 8]; 8]; 4],
    pub active_sprite_idx: usize,

    // 16x16 Tilemap (Svet sastavljen od 4 sprajta iz banke)
    pub tilemap: [[u8; 16]; 16],
    pub selected_tile_to_place: u8,

    pub grid_visible: bool,
    pub status_message: String,
}

impl QuatSpriteStudio {
    pub fn new() -> Self {
        Self {
            active_tab: ActiveTab::SpriteEditor,
            current_tool: SpriteTool::Pencil,
            selected_quat_color: 3, // Podrazumevano bela/najsvetlija boja
            sprite_bank: [[[0; 8]; 8]; 4],
            active_sprite_idx: 0,
            tilemap: [[0; 16]; 16],
            selected_tile_to_place: 0,
            grid_visible: true,
            status_message: "QuatSprite Studio spreman za crtanje.".to_string(),
        }
    }

    /// Konvertuje kvatnu boju (0-3) u egui Color32
    fn quat_to_color(quat: u8) -> egui::Color32 {
        match quat {
            0 => egui::Color32::from_rgb(15, 15, 20),    // Kvat 0: Prazno / Crna pozadina
            1 => egui::Color32::from_rgb(70, 90, 120),   // Kvat 1: Tamno plavo / Tamna
            2 => egui::Color32::from_rgb(0, 200, 150),   // Kvat 2: Svetlo zelena / Akcenat
            3 => egui::Color32::from_rgb(240, 240, 240), // Kvat 3: Beli piksel
            _ => egui::Color32::BLACK,
        }
    }

    // =========================================================================
    // TRANSFORMACIJE SPRAJTA (8x8 Logika)
    // =========================================================================

    pub fn flip_horizontal(&mut self) {
        let sprite = &mut self.sprite_bank[self.active_sprite_idx];
        for y in 0..8 {
            sprite[y].reverse();
        }
        self.status_message = "Sprajt obrnut horizontalno.".to_string();
    }

    pub fn flip_vertical(&mut self) {
        let sprite = &mut self.sprite_bank[self.active_sprite_idx];
        for x in 0..8 {
            let mut col = [0u8; 8];
            for y in 0..8 { col[y] = sprite[y][x]; }
            col.reverse();
            for y in 0..8 { sprite[y][x] = col[y]; }
        }
        self.status_message = "Sprajt obrnut vertikalno.".to_string();
    }

    pub fn rotate_90(&mut self) {
        let sprite = self.sprite_bank[self.active_sprite_idx];
        let mut new_sprite = [[0u8; 8]; 8];
        for y in 0..8 {
            for x in 0..8 {
                new_sprite[x][7 - y] = sprite[y][x];
            }
        }
        self.sprite_bank[self.active_sprite_idx] = new_sprite;
        self.status_message = "Sprajt rotiran za 90°.".to_string();
    }

    pub fn invert_quats(&mut self) {
        let sprite = &mut self.sprite_bank[self.active_sprite_idx];
        for y in 0..8 {
            for x in 0..8 {
                sprite[y][x] = 3 - sprite[y][x];
            }
        }
        self.status_message = "Kvatne boje invertovane.".to_string();
    }

    pub fn bucket_fill(&mut self, start_x: usize, start_y: usize, target_color: u8) {
        let sprite = &mut self.sprite_bank[self.active_sprite_idx];
        let old_color = sprite[start_y][start_x];
        if old_color == target_color { return; }

        let mut queue = vec![(start_x, start_y)];
        while let Some((x, y)) = queue.pop() {
            if sprite[y][x] == old_color {
                sprite[y][x] = target_color;
                if x > 0 { queue.push((x - 1, y)); }
                if x < 7 { queue.push((x + 1, y)); }
                if y > 0 { queue.push((x, y - 1)); }
                if y < 7 { queue.push((x, y + 1)); }
            }
        }
        self.status_message = "Površina popunjena bojom.".to_string();
    }

    // =========================================================================
    // UI RENDERING METODA
    // =========================================================================

    pub fn ui(&mut self, ui: &mut egui::Ui, _kernel: &mut QuatKernel) {
        ui.heading("🎨 QuatSprite Studio & Tilemap World Builder");
        ui.label("Profacionalni 8x8 sprajt editor i generator nivoa za $4^4$ VRAM arhitekturu.");
        ui.separator();

        // TABS MENU
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.active_tab, ActiveTab::SpriteEditor, "✏ 8x8 Sprite Editor");
            ui.selectable_value(&mut self.active_tab, ActiveTab::TilemapEditor, "🗺 16x16 Tilemap Map Builder");
            ui.selectable_value(&mut self.active_tab, ActiveTab::ExportCode, "💻 Assembly / Hex Exporter");
        });

        ui.separator();

        match self.active_tab {
            ActiveTab::SpriteEditor => self.render_sprite_editor(ui),
            ActiveTab::TilemapEditor => self.render_tilemap_editor(ui),
            ActiveTab::ExportCode => self.render_export_tab(ui),
        }

        ui.add_space(10.0);
        ui.label(egui::RichText::new(&self.status_message).italics().color(egui::Color32::LIGHT_BLUE));
    }

    fn render_sprite_editor(&mut self, ui: &mut egui::Ui) {
        ui.columns(2, |cols| {
            // LEVA KOLONA: PLATNO ZANECRTANJE 8x8
            cols[0].vertical(|ui| {
                ui.label(egui::RichText::new("🎯 Platno (8x8 Quat Pixels):").strong());
                ui.add_space(5.0);

                let cell_size = 32.0; // Veličina piksela u editoru
                let (response, painter) = ui.allocate_painter(egui::vec2(cell_size * 8.0, cell_size * 8.0), egui::Sense::click_and_drag());
                let rect = response.rect;

                // Crtanje piksela
                let sprite = &mut self.sprite_bank[self.active_sprite_idx];
                for y in 0..8 {
                    for x in 0..8 {
                        let px_rect = egui::Rect::from_min_size(
                            rect.min + egui::vec2(x as f32 * cell_size, y as f32 * cell_size),
                            egui::vec2(cell_size, cell_size),
                        );

                        let color = Self::quat_to_color(sprite[y][x]);
                        painter.rect_filled(px_rect, 0.0, color);

                        if self.grid_visible {
                            painter.rect_stroke(px_rect, 0.0, egui::Stroke::new(1.0, egui::Color32::from_gray(50)));
                        }
                    }
                }

                // Interakcija miša sa platnom
                if response.clicked() || response.dragged() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let rel_pos = pos - rect.min;
                        let x = (rel_pos.x / cell_size).floor() as i32;
                        let y = (rel_pos.y / cell_size).floor() as i32;

                        if x >= 0 && x < 8 && y >= 0 && y < 8 {
                            let (ux, uy) = (x as usize, y as usize);
                            match self.current_tool {
                                SpriteTool::Pencil => sprite[uy][ux] = self.selected_quat_color,
                                SpriteTool::Eraser => sprite[uy][ux] = 0,
                                SpriteTool::Picker => self.selected_quat_color = sprite[uy][ux],
                                SpriteTool::BucketFill => {
                                    let col = self.selected_quat_color;
                                    self.bucket_fill(ux, uy, col);
                                }
                            }
                        }
                    }
                }
            });

            // DESNA KOLONA: PALETA, ALATI I BANKA SPRAJTOVA
            cols[1].vertical(|ui| {
                ui.group(|ui| {
                    ui.label(egui::RichText::new("🎨 Quat-Paleta Boja (4 Nivoa):").strong());
                    ui.horizontal(|ui| {
                        for q in 0..4u8 {
                            let is_selected = self.selected_quat_color == q;
                            let btn_text = format!("Kvat {}", q);
                            let color = Self::quat_to_color(q);

                            if ui.add(egui::Button::new(btn_text).fill(color)).clicked() {
                                self.selected_quat_color = q;
                            }
                            if is_selected { ui.label("⬅"); }
                        }
                    });
                });

                ui.add_space(10.0);
                ui.group(|ui| {
                    ui.label(egui::RichText::new("🛠 Alati za Crtanje:").strong());
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.current_tool, SpriteTool::Pencil, "✏ Olovka");
                        ui.selectable_value(&mut self.current_tool, SpriteTool::Eraser, "🧹 Gumica");
                        ui.selectable_value(&mut self.current_tool, SpriteTool::BucketFill, "🪣 Kanta");
                        ui.selectable_value(&mut self.current_tool, SpriteTool::Picker, "🧪 Pipeta");
                    });
                    ui.checkbox(&mut self.grid_visible, "Prikaži Mrežu");
                });

                ui.add_space(10.0);
                ui.group(|ui| {
                    ui.label(egui::RichText::new("🔄 Transformacije:").strong());
                    ui.horizontal(|ui| {
                        if ui.button("↔ Flip H").clicked() { self.flip_horizontal(); }
                        if ui.button("↕ Flip V").clicked() { self.flip_vertical(); }
                        if ui.button("🔄 Rotiraj 90°").clicked() { self.rotate_90(); }
                        if ui.button("☯ Invert").clicked() { self.invert_quats(); }
                    });
                });

                ui.add_space(10.0);
                ui.group(|ui| {
                    ui.label(egui::RichText::new("📦 Banka Sprajtova (4 Slot-a):").strong());
                    ui.horizontal(|ui| {
                        for i in 0..4 {
                            if ui.selectable_label(self.active_sprite_idx == i, format!("Sprite #{}", i)).clicked() {
                                self.active_sprite_idx = i;
                            }
                        }
                    });
                });
            });
        });
    }

    fn render_tilemap_editor(&mut self, ui: &mut egui::Ui) {
        ui.label("🗺 16x16 Tilemap Builder (Postavljaj sprajtove u svet):");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Izaberi sprajt iz banke za postavljanje: ");
            for i in 0..4 {
                if ui.selectable_label(self.selected_tile_to_place == i, format!("Sprajt #{}", i)).clicked() {
                    self.selected_tile_to_place = i;
                }
            }
        });

        ui.add_space(10.0);

        // Mreža 16x16 za sklapanje mape
        let cell_size = 18.0;
        let (response, painter) = ui.allocate_painter(egui::vec2(cell_size * 16.0, cell_size * 16.0), egui::Sense::click_and_drag());
        let rect = response.rect;

        for y in 0..16 {
            for x in 0..16 {
                let tile_rect = egui::Rect::from_min_size(
                    rect.min + egui::vec2(x as f32 * cell_size, y as f32 * cell_size),
                    egui::vec2(cell_size, cell_size),
                );

                let sprite_id = self.tilemap[y][x] as usize;
                // Prikazujemo dominantnu boju izabranog sprajta radi brze vizuelizacije
                let preview_color = Self::quat_to_color((sprite_id as u8 + 1) % 4);
                
                painter.rect_filled(tile_rect, 0.0, preview_color);
                painter.rect_stroke(tile_rect, 0.0, egui::Stroke::new(0.5, egui::Color32::GRAY));
            }
        }

        if response.clicked() || response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                let rel = pos - rect.min;
                let x = (rel.x / cell_size).floor() as i32;
                let y = (rel.y / cell_size).floor() as i32;

                if x >= 0 && x < 16 && y >= 0 && y < 16 {
                    self.tilemap[y as usize][x as usize] = self.selected_tile_to_place;
                }
            }
        }
    }

    fn render_export_tab(&mut self, ui: &mut egui::Ui) {
        ui.label("💻 Generisani Bajtkod & Asemblerski Zapis Sprajta:");
        ui.separator();

        let sprite = self.sprite_bank[self.active_sprite_idx];
        let mut asm_code = format!("; --- QUATSPRITE #{} EXPORT (8x8) ---\n", self.active_sprite_idx);
        asm_code.push_str("SPRITE_DATA:\n");

        // Svaka 4 piksela pakujemo u 1 QuatByte (1 Bajt = 4 Kvata)
        let mut raw_bytes: Vec<u8> = Vec::new();
        for y in 0..8 {
            asm_code.push_str("  .DB ");
            for chunk in 0..2 {
                let q0 = sprite[y][chunk * 4];
                let q1 = sprite[y][chunk * 4 + 1];
                let q2 = sprite[y][chunk * 4 + 2];
                let q3 = sprite[y][chunk * 4 + 3];

                // Pakovanje u 8-bitni bajt
                let byte_val = (q3 << 6) | (q2 << 4) | (q1 << 2) | q0;
                raw_bytes.push(byte_val);

                asm_code.push_str(&format!("0x{:02X}", byte_val));
                if chunk == 0 { asm_code.push_str(", "); }
            }
            asm_code.push('\n');
        }

        ui.code_editor(&mut asm_code);

        ui.add_space(10.0);
        if ui.button("📋 Kopiraj Asemblerski Kôd").clicked() {
            ui.output_mut(|o| o.copied_text = asm_code.clone());
            self.status_message = "Kôd uspešno kopiran u Clipboard!".to_string();
        }
    }
}