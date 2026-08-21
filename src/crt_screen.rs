use eframe::egui;
use crate::kernel::{QuatByte, QuatKernel};

// Rezolucija našeg virtuelnog CRT ekrana
pub const CRT_WIDTH: usize = 160;
pub const CRT_HEIGHT: usize = 120;

// ============================================================================
// 1. TIPOVI MASKE, PRESETI I PODEŠAVANJA FIZIKE CRT-A
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShadowMaskType {
    None,
    ApertureGrille, // Trinitron vertikalne linije
    ShadowMask,     // Klasična tačkasta rešetka
    SlotMask,       // Pravougaoni slotovi (stariji televizori)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorPhosphorMode {
    FullColor,      // 16/256 Paleta boja
    GreenPhosphor,  // IBM 5151 Monohromatski zeleni
    AmberPhosphor,  // VT100 Monohromatski ćilibar/zlatni
    PaperWhite,     // Bankarski beli terminal
}

#[derive(Debug, Clone)]
pub struct CrtSettings {
    pub power_on: bool,
    pub power_anim_progress: f32, // 0.0 (Ugašen) do 1.0 (Puna slika)
    pub curvature: f32,           // Zakrivljenje stakla (Barrel Distortion)
    pub scanline_depth: f32,      // Intenzitet crnih linija između redova
    pub phosphor_decay: f32,      // Trag fosfora pri brzom pokretu (0.7 - 0.98)
    pub chromatic_aberration: f32,// Razdvajanje Crvene i Plave na ivicama
    pub vignette: f32,            // Zatamnjenje uglova ekrana
    pub noise_level: f32,         // Analogni šum / sneg na ekranu
    pub beam_brightness: f32,     // Svetlina elektronskog topa
    pub mask_type: ShadowMaskType,
    pub color_mode: ColorPhosphorMode,
    pub vhs_jitter: bool,         // Horizontalno drhtanje slike
}

impl Default for CrtSettings {
    fn default() -> Self {
        Self {
            power_on: true,
            power_anim_progress: 1.0,
            curvature: 0.18,
            scanline_depth: 0.35,
            phosphor_decay: 0.82,
            chromatic_aberration: 1.2,
            vignette: 0.4,
            noise_level: 0.03,
            beam_brightness: 1.1,
            mask_type: ShadowMaskType::ApertureGrille,
            color_mode: ColorPhosphorMode::FullColor,
            vhs_jitter: false,
        }
    }
}

// ============================================================================
// 2. FONT MAPA ZUTIHK ARHAIČNIH 8x8 ZNAKOVA
// ============================================================================

pub const CRT_FONT_8X8: [[u8; 8]; 16] = [
    [0x3E, 0x66, 0x6E, 0x76, 0x66, 0x66, 0x3E, 0x00], // '0'
    [0x1C, 0x3C, 0x1C, 0x1C, 0x1C, 0x1C, 0x7E, 0x00], // '1'
    [0x3E, 0x66, 0x06, 0x1C, 0x30, 0x60, 0x7E, 0x00], // '2'
    [0x3E, 0x66, 0x06, 0x1E, 0x06, 0x66, 0x3E, 0x00], // '3'
    [0x0C, 0x1C, 0x3C, 0x6C, 0xFE, 0x0C, 0x0C, 0x00], // '4'
    [0x7E, 0x60, 0x7C, 0x06, 0x06, 0x66, 0x3E, 0x00], // '5'
    [0x1C, 0x30, 0x60, 0x7C, 0x66, 0x66, 0x3E, 0x00], // '6'
    [0x7E, 0x66, 0x0C, 0x18, 0x18, 0x18, 0x18, 0x00], // '7'
    [0x3E, 0x66, 0x66, 0x3E, 0x66, 0x66, 0x3E, 0x00], // '8'
    [0x3E, 0x66, 0x66, 0x3E, 0x06, 0x0C, 0x38, 0x00], // '9'
    [0x1C, 0x36, 0x63, 0x7F, 0x63, 0x63, 0x63, 0x00], // 'A'
    [0x7C, 0x66, 0x66, 0x7C, 0x66, 0x66, 0x7C, 0x00], // 'B'
    [0x3C, 0x66, 0x60, 0x60, 0x60, 0x66, 0x3C, 0x00], // 'C'
    [0x78, 0x6C, 0x66, 0x66, 0x66, 0x6C, 0x78, 0x00], // 'D'
    [0x7E, 0x60, 0x60, 0x78, 0x60, 0x60, 0x7E, 0x00], // 'E'
    [0x7E, 0x60, 0x60, 0x78, 0x60, 0x60, 0x60, 0x00], // 'F'
];

// ============================================================================
// 3. CRT SCREEN SUBSISTEM
// ============================================================================

pub struct CrtScreen {
    pub settings: CrtSettings,
    
    // VRAM Sirove Boje u RGB formatu (160x120x3)
    pub vram_rgb: Vec<[u8; 3]>,
    
    // Akumulativni bafer za simulaciju sporog gašenja fosfora (Afterglow decay)
    phosphor_buffer: Vec<[f32; 3]>,
    
    // egui tekstura za prikaz osvežene slike
    texture_handle: Option<egui::TextureHandle>,
    
    // Brojač frame-ova za animacije i VHS šum
    frame_counter: u64,
    random_seed: u32,
}

impl CrtScreen {
    pub fn new() -> Self {
        let pixel_count = CRT_WIDTH * CRT_HEIGHT;
        let mut screen = Self {
            settings: CrtSettings::default(),
            vram_rgb: vec![[0, 0, 0]; pixel_count],
            phosphor_buffer: vec![[0.0, 0.0, 0.0]; pixel_count],
            texture_handle: None,
            frame_counter: 0,
            random_seed: 1337,
        };

        screen.draw_test_pattern();
        screen
    }

    /// Generiše retro CRT demo test sliku sa rešetkom i tekston
    pub fn draw_test_pattern(&mut self) {
        for y in 0..CRT_HEIGHT {
            for x in 0..CRT_WIDTH {
                let idx = y * CRT_WIDTH + x;

                // Okvir i rešetka
                if x == 0 || x == CRT_WIDTH - 1 || y == 0 || y == CRT_HEIGHT - 1 {
                    self.vram_rgb[idx] = [200, 200, 200];
                } else if x % 20 == 0 || y % 20 == 0 {
                    self.vram_rgb[idx] = [30, 40, 60];
                } else {
                    // Gradijent boja u 4-kvatnom duhu
                    let r = ((x as f32 / CRT_WIDTH as f32) * 255.0) as u8;
                    let g = ((y as f32 / CRT_HEIGHT as f32) * 255.0) as u8;
                    let b = 128;
                    self.vram_rgb[idx] = [r / 2, g / 2, b / 2];
                }
            }
        }

        // Ispisujemo "QUAT-OS 256" u VRAM
        self.draw_char_at('C', 10, 10, [255, 255, 0]);
        self.draw_char_at('R', 18, 10, [255, 255, 0]);
        self.draw_char_at('T', 26, 10, [255, 255, 0]);
        self.draw_char_at('0', 42, 10, [0, 255, 255]);
        self.draw_char_at('1', 50, 10, [0, 255, 255]);
    }

    /// Crta jedan 8x8 karakterni znak direktno u VRAM
    pub fn draw_char_at(&mut self, ch: char, start_x: usize, start_y: usize, color: [u8; 3]) {
        let char_idx = match ch {
            '0'..='9' => (ch as u8 - b'0') as usize,
            'A'..='F' => (ch as u8 - b'A' + 10) as usize,
            _ => 0,
        };

        let glyph = CRT_FONT_8X8[char_idx];
        for row in 0..8 {
            let py = start_y + row;
            if py >= CRT_HEIGHT { continue; }
            let line = glyph[row];
            for col in 0..8 {
                let px = start_x + col;
                if px >= CRT_WIDTH { continue; }
                if (line & (0x80 >> col)) != 0 {
                    let idx = py * CRT_WIDTH + px;
                    self.vram_rgb[idx] = color;
                }
            }
        }
    }

    /// Pseudo-random generator za analogni CRT šum
    fn next_random(&mut self) -> f32 {
        self.random_seed = self.random_seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.random_seed as f32) / (u32::MAX as f32)
    }

    /// GLAVNI SHADER & POST-PROCESSING ENGINE
    /// Generiše gotov `egui::ColorImage` primenom matematičkih CRT efekata
    pub fn render_crt_frame(&mut self) -> egui::ColorImage {
        self.frame_counter += 1;
        
        // Animacija paljenja / gašenja (Collapse-to-line)
        if self.settings.power_on {
            self.settings.power_anim_progress = (self.settings.power_anim_progress + 0.08).min(1.0);
        } else {
            self.settings.power_anim_progress = (self.settings.power_anim_progress - 0.08).max(0.0);
        }

        let mut output_pixels = vec![egui::Color32::BLACK; CRT_WIDTH * CRT_HEIGHT];
        let aspect_ratio = CRT_WIDTH as f32 / CRT_HEIGHT as f32;

        // VHS Jitter offset za ovaj frejm
        let jitter_offset_x = if self.settings.vhs_jitter && (self.frame_counter % 6 == 0) {
            ((self.next_random() - 0.5) * 3.0) as i32
        } else {
            0
        };

        for py in 0..CRT_HEIGHT {
            for px in 0..CRT_WIDTH {
                let out_idx = py * CRT_WIDTH + px;

                if self.settings.power_anim_progress <= 0.01 {
                    continue;
                }

                // Normalizovane koordinate ekrana (-1.0 do 1.0)
                let mut uv_x = (px as f32 / CRT_WIDTH as f32) * 2.0 - 1.0;
                let mut uv_y = (py as f32 / CRT_HEIGHT as f32) * 2.0 - 1.0;

                // --- 1. Sfera / Zakrivljenje Ekrana (Barrel Distortion) ---
                let r2 = (uv_x * uv_x) / aspect_ratio + (uv_y * uv_y);
                let dist_factor = 1.0 + r2 * self.settings.curvature;
                uv_x *= dist_factor;
                uv_y *= dist_factor;

                // --- 2. CRT Power Collapse Efekat ---
                if self.settings.power_anim_progress < 0.99 {
                    uv_y /= self.settings.power_anim_progress.max(0.01);
                    uv_x /= (self.settings.power_anim_progress * 2.0).min(1.0).max(0.01);
                }

                // Vraćanje u piksel koordinate sa Jitter-om
                let src_x = (((uv_x + 1.0) * 0.5) * CRT_WIDTH as f32) as i32 + jitter_offset_x;
                let src_y = (((uv_y + 1.0) * 0.5) * CRT_HEIGHT as f32) as i32;

                // Provera granica van zakrivljenog stakla
                if src_x < 0 || src_x >= CRT_WIDTH as i32 || src_y < 0 || src_y >= CRT_HEIGHT as i32 {
                    output_pixels[out_idx] = egui::Color32::from_rgb(10, 10, 12);
                    continue;
                }

                let src_idx = (src_y as usize) * CRT_WIDTH + (src_x as usize);

                // --- 3. Hromatska Aberacija (Razdvajanje RGB snopova) ---
                let ca_shift = (self.settings.chromatic_aberration * (r2 + 0.1)) as i32;
                
                let r_idx = (src_y as usize) * CRT_WIDTH + (src_x - ca_shift).clamp(0, CRT_WIDTH as i32 - 1) as usize;
                let b_idx = (src_y as usize) * CRT_WIDTH + (src_x + ca_shift).clamp(0, CRT_WIDTH as i32 - 1) as usize;

                let raw_r = self.vram_rgb[r_idx][0] as f32;
                let raw_g = self.vram_rgb[src_idx][1] as f32;
                let raw_b = self.vram_rgb[b_idx][2] as f32;

                // --- 4. Akumulacija Fosfora (Phosphor Decay / Afterglow) ---
                let p_decay = self.settings.phosphor_decay;
                let old_p = self.phosphor_buffer[out_idx];
                let cur_r = raw_r.max(old_p[0] * p_decay);
                let cur_g = raw_g.max(old_p[1] * p_decay);
                let cur_b = raw_b.max(old_p[2] * p_decay);
                self.phosphor_buffer[out_idx] = [cur_r, cur_g, cur_b];

                // --- 5. Scanlines (Raster Linije) ---
                let scanline_mult = if py % 2 == 0 {
                    1.0
                } else {
                    1.0 - self.settings.scanline_depth
                };

                // --- 6. RGB Shadow Mask / Aperture Grille ---
                let (mask_r, mask_g, mask_b) = match self.settings.mask_type {
                    ShadowMaskType::None => (1.0, 1.0, 1.0),
                    ShadowMaskType::ApertureGrille => match px % 3 {
                        0 => (1.2, 0.7, 0.7),
                        1 => (0.7, 1.2, 0.7),
                        _ => (0.7, 0.7, 1.2),
                    },
                    ShadowMaskType::ShadowMask => match (px % 2, py % 2) {
                        (0, 0) => (1.3, 0.6, 0.6),
                        (1, 0) => (0.6, 1.3, 0.6),
                        (0, 1) => (0.6, 0.6, 1.3),
                        _ => (0.8, 0.8, 0.8),
                    },
                    ShadowMaskType::SlotMask => (1.0, 1.0, 1.0),
                };

                // --- 7. Vignette (Zatamnjenje na ivicama stakla) ---
                let vig = (1.0 - r2 * self.settings.vignette).clamp(0.2, 1.0);

                // --- 8. Analogni Šum (Sneg) ---
                let noise = (self.next_random() - 0.5) * self.settings.noise_level * 255.0;

                // Spajanje svih efekata
                let mut final_r = (cur_r * scanline_mult * mask_r * vig * self.settings.beam_brightness + noise).clamp(0.0, 255.0);
                let mut final_g = (cur_g * scanline_mult * mask_g * vig * self.settings.beam_brightness + noise).clamp(0.0, 255.0);
                let mut final_b = (cur_b * scanline_mult * mask_b * vig * self.settings.beam_brightness + noise).clamp(0.0, 255.0);

                // --- 9. Monohromatske Palete Fosfora ---
                match self.settings.color_mode {
                    ColorPhosphorMode::FullColor => {},
                    ColorPhosphorMode::GreenPhosphor => {
                        let gray = final_r * 0.3 + final_g * 0.59 + final_b * 0.11;
                        final_r = gray * 0.1;
                        final_g = gray * 1.1;
                        final_b = gray * 0.1;
                    }
                    ColorPhosphorMode::AmberPhosphor => {
                        let gray = final_r * 0.3 + final_g * 0.59 + final_b * 0.11;
                        final_r = gray * 1.1;
                        final_g = gray * 0.75;
                        final_b = gray * 0.1;
                    }
                    ColorPhosphorMode::PaperWhite => {
                        let gray = final_r * 0.3 + final_g * 0.59 + final_b * 0.11;
                        final_r = gray * 0.9;
                        final_g = gray * 0.95;
                        final_b = gray * 1.0;
                    }
                }

                output_pixels[out_idx] = egui::Color32::from_rgb(
                    final_r as u8,
                    final_g as u8,
                    final_b as u8,
                );
            }
        }

        egui::ColorImage {
            size: [CRT_WIDTH, CRT_HEIGHT],
            pixels: output_pixels,
        }
    }

    /// Renders UI layout inside egui
    pub fn ui(&mut self, ui: &mut egui::Ui, kernel: &mut QuatKernel) {
        ui.heading("📺 QuatOS CRT Ekran (Simulacija Katodne Cevi)");
        ui.separator();

        // Generišemo osveženi CRT frame
        let color_image = self.render_crt_frame();
        
        // Ažuriramo egui teksturu
        let texture = ui.ctx().load_texture("crt_texture", color_image, egui::TextureOptions::NEAREST);
        self.texture_handle = Some(texture.clone());

        ui.columns(2, |cols| {
            // KOLONA 1: CRT TV Ekran
            cols[0].vertical(|ui| {
                ui.label("🖥 CRT Katodni Monitor (Real-time Simulation):");
                
                let display_size = egui::vec2(380.0, 285.0); // 4:3 Odnos stranica
                ui.image((texture.id(), display_size));

                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    if ui.button(if self.settings.power_on { "🔴 Ugasi TV" } else { "🟢 Upali TV" }).clicked() {
                        self.settings.power_on = !self.settings.power_on;
                    }
                    if ui.button("🎨 Test Slika").clicked() {
                        self.draw_test_pattern();
                    }
                    if ui.button("🧹 Očisti VRAM").clicked() {
                        self.vram_rgb.fill([0, 0, 0]);
                    }
                });
            });

            // KOLONA 2: PODEŠAVANJA FIZIKE CRT-A
            cols[1].vertical(|ui| {
                ui.label("🎛 Kontrolna Tabla Katodne Cevi:");

                egui::CollapsingHeader::new("⚙ Preseti Ekrana").default_open(true).show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("🕹 Trinitron Arcade").clicked() {
                            self.settings = CrtSettings::default();
                        }
                        if ui.button("📟 IBM Green (5151)").clicked() {
                            self.settings = CrtSettings::default();
                            self.settings.color_mode = ColorPhosphorMode::GreenPhosphor;
                            self.settings.phosphor_decay = 0.92;
                        }
                        if ui.button("📜 VT100 Amber").clicked() {
                            self.settings = CrtSettings::default();
                            self.settings.color_mode = ColorPhosphorMode::AmberPhosphor;
                        }
                        if ui.button("📼 VHS Glitch TV").clicked() {
                            self.settings = CrtSettings::default();
                            self.settings.vhs_jitter = true;
                            self.settings.noise_level = 0.08;
                            self.settings.chromatic_aberration = 3.0;
                        }
                    });
                });

                ui.add_space(5.0);

                egui::CollapsingHeader::new("📐 Optika & Zakrivljenje Stakla").default_open(true).show(ui, |ui| {
                    ui.add(egui::Slider::new(&mut self.settings.curvature, 0.0..=0.5).text("Zakrivljenje Stakla"));
                    ui.add(egui::Slider::new(&mut self.settings.scanline_depth, 0.0..=1.0).text("Scanlines Intenzitet"));
                    ui.add(egui::Slider::new(&mut self.settings.chromatic_aberration, 0.0..=5.0).text("Hromatska Aberacija"));
                    ui.add(egui::Slider::new(&mut self.settings.vignette, 0.0..=1.0).text("Vignette (Uglovi)"));
                });

                ui.add_space(5.0);

                egui::CollapsingHeader::new("⚡ Elektronski Top & Fosfor").default_open(true).show(ui, |ui| {
                    ui.add(egui::Slider::new(&mut self.settings.beam_brightness, 0.5..=2.0).text("Svetlina Snopa"));
                    ui.add(egui::Slider::new(&mut self.settings.phosphor_decay, 0.0..=0.98).text("Fosfor Afterglow"));
                    ui.add(egui::Slider::new(&mut self.settings.noise_level, 0.0..=0.2).text("Analogni Šum"));
                    ui.checkbox(&mut self.settings.vhs_jitter, "VHS Jitter (Drhtanje)");
                });

                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("Rešetka:");
                    ui.selectable_value(&mut self.settings.mask_type, ShadowMaskType::None, "Isključeno");
                    ui.selectable_value(&mut self.settings.mask_type, ShadowMaskType::ApertureGrille, "Grille");
                    ui.selectable_value(&mut self.settings.mask_type, ShadowMaskType::ShadowMask, "Mask");
                });
            });
        });

        ui.separator();
        ui.label(egui::RichText::new(format!(
            "CRT Status: Rezolucija {}x{} | Frame: {} | Power: {:.0}%",
            CRT_WIDTH, CRT_HEIGHT, self.frame_counter, self.settings.power_anim_progress * 100.0
        )).monospace().color(egui::Color32::GREEN));
    }
}