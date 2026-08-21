use eframe::egui;
use crate::kernel::{QuatKernel, QuatByte};

pub const VGA_WIDTH: usize = 320;
pub const VGA_HEIGHT: usize = 200;
pub const VRAM_SIZE: usize = VGA_WIDTH * VGA_HEIGHT; // 64,000 bajtova

pub struct QuatVGA {
    pub vram: Vec<u8>,                      // 320x200 bajtova (indeksi palete)
    pub palette: [(u8, u8, u8); 256],       // VGA 256-kolorna paleta
    texture: Option<egui::TextureHandle>,   // egui tekstura za prikaz
    pub demo_mode: usize,                   // 0: Off, 1: Plasma, 2: Fire, 3: Starfield
    pub frame_counter: usize,
}

impl QuatVGA {
    pub fn new() -> Self {
        let mut vga = Self {
            vram: vec![0; VRAM_SIZE],
            palette: [(0, 0, 0); 256],
            texture: None,
            demo_mode: 0,
            frame_counter: 0,
        };
        vga.init_default_palette();
        vga.clear(0); // Crni ekran
        vga
    }

    /// Inicijalizacija klasične VGA 256-kolorne palete (VGA Standard Palette)
    fn init_default_palette(&mut self) {
        // Prvih 16 boja su standardne CGA/EGA boje
        let cga_colors: [(u8, u8, u8); 16] = [
            (0, 0, 0),       (0, 0, 170),     (0, 170, 0),     (0, 170, 170),
            (170, 0, 0),     (170, 0, 170),   (170, 85, 0),    (170, 170, 170),
            (85, 85, 85),    (85, 85, 255),    (85, 255, 85),   (85, 255, 255),
            (255, 85, 85),   (255, 85, 255),  (255, 255, 85),  (255, 255, 255),
        ];
        for i in 0..16 {
            self.palette[i] = cga_colors[i];
        }

        // Boje 16..231: 6x6x6 RGB kocka za retro grafiku
        let mut idx = 16;
        for r in 0..6 {
            for g in 0..6 {
                for b in 0..6 {
                    self.palette[idx] = (
                        if r == 0 { 0 } else { r * 40 + 55 },
                        if g == 0 { 0 } else { g * 40 + 55 },
                        if b == 0 { 0 } else { b * 40 + 55 },
                    );
                    idx += 1;
                }
            }
        }

        // Boje 232..255: Grayscale skala (od tamno sive do bele)
        for i in 0..24 {
            let gray = (i * 10 + 15) as u8;
            self.palette[232 + i] = (gray, gray, gray);
        }
    }

    /// Očisti VRAM sa određenom bojom
    pub fn clear(&mut self, color_idx: u8) {
        self.vram.fill(color_idx);
    }

    /// Postavi piksel na (x, y)
    pub fn put_pixel(&mut self, x: usize, y: usize, color_idx: u8) {
        if x < VGA_WIDTH && y < VGA_HEIGHT {
            self.vram[y * VGA_WIDTH + x] = color_idx;
        }
    }

    /// Crtanje pravougaonika
    pub fn draw_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color_idx: u8) {
        for dy in 0..h {
            for dx in 0..w {
                self.put_pixel(x + dx, y + dy, color_idx);
            }
        }
    }

    /// Takt VGA kontrolera — izvršava ugrađene demoscene efekte!
    pub fn tick(&mut self, _kernel: &mut QuatKernel) {
        self.frame_counter = self.frame_counter.wrapping_add(1);
        let t = self.frame_counter as f32 * 0.05;

        match self.demo_mode {
            1 => self.render_plasma_demo(t),
            2 => self.render_retro_fire_demo(),
            3 => self.render_starfield_demo(),
            _ => {} // Ako je 0, VRAM ostaje onakav kakav je nacrtan
        }
    }

    /// Legendarni Retro Plasma Efekat sa demoscene iz 1994. godine
    fn render_plasma_demo(&mut self, t: f32) {
        for y in 0..VGA_HEIGHT {
            for x in 0..VGA_WIDTH {
                let fx = x as f32 * 0.03;
                let fy = y as f32 * 0.03;

                let v1 = (fx + t).sin();
                let v2 = (fy + t * 1.2).sin();
                let v3 = ((fx + fy + t) * 0.5).sin();
                let v4 = (((fx * fx + fy * fy).sqrt()) + t).sin();

                let color = ((v1 + v2 + v3 + v4 + 4.0) * 28.0) as u8;
                self.vram[y * VGA_WIDTH + x] = color;
            }
        }
    }

    /// Klasični DOOM/PSX vatreni efekat
    fn render_retro_fire_demo(&mut self) {
        // Dno ekrana natapamo sa nasumičnim izvorom vatre
        for x in 0..VGA_WIDTH {
            let rnd = (self.frame_counter.wrapping_mul(x + 13) % 2 == 0) as u8;
            self.vram[(VGA_HEIGHT - 1) * VGA_WIDTH + x] = if rnd == 1 { 255 } else { 200 };
        }

        // Propagacija plamena ka gore sa rasejavanjem
        for y in 1..VGA_HEIGHT {
            for x in 0..VGA_WIDTH {
                let src_idx = y * VGA_WIDTH + x;
                let pixel = self.vram[src_idx];
                if pixel == 0 {
                    self.vram[(y - 1) * VGA_WIDTH + x] = 0;
                } else {
                    let decay = (self.frame_counter.wrapping_add(x + y) % 3) as u8;
                    let dst_x = if x > 0 && decay == 1 { x - 1 } else { x };
                    let new_pixel = pixel.saturating_sub(decay * 4);
                    self.vram[(y - 1) * VGA_WIDTH + dst_x] = new_pixel;
                }
            }
        }
    }

    /// 3D Zvezdano Polje (Starfield)
    fn render_starfield_demo(&mut self) {
        self.clear(0);
        let speed = self.frame_counter;
        for i in 0..150 {
            let star_x = (i * 37 + speed * (i % 4 + 1)) % VGA_WIDTH;
            let star_y = (i * 91) % VGA_HEIGHT;
            let star_color = match i % 3 {
                0 => 15, // Jaka bela
                1 => 7,  // Svetlo siva
                _ => 8,  // Tamno siva (u daljini)
            };
            self.put_pixel(star_x, star_y, star_color);
        }
    }

    /// Renderovanje VRAM-a u egui prozor
    pub fn ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("📺 QuatVGA — Virtual Graphics Adapter (Mode 13h)");
        ui.label("Rezolucija: 320x200 @ 256 Boja | VRAM: 64 KB (0xA000 Segment)");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Demoscene Demos:");
            if ui.selectable_label(self.demo_mode == 0, "⏸️ Off (Custom VRAM)").clicked() { self.demo_mode = 0; }
            if ui.selectable_label(self.demo_mode == 1, "🌀 Plasma").clicked() { self.demo_mode = 1; }
            if ui.selectable_label(self.demo_mode == 2, "🔥 Retro Fire").clicked() { self.demo_mode = 2; }
            if ui.selectable_label(self.demo_mode == 3, "✨ Starfield").clicked() { self.demo_mode = 3; }

            if ui.button("🧹 Clear VRAM").clicked() {
                self.demo_mode = 0;
                self.clear(0);
            }
        });

        ui.add_space(8.0);

        // Prevođenje 8-bitnih indeksa iz VRAM-a u RGBA piksele za egui
        let mut rgba_pixels = vec![0u8; VGA_WIDTH * VGA_HEIGHT * 4];
        for i in 0..VRAM_SIZE {
            let color_idx = self.vram[i] as usize;
            let (r, g, b) = self.palette[color_idx];
            let offset = i * 4;
            rgba_pixels[offset] = r;
            rgba_pixels[offset + 1] = g;
            rgba_pixels[offset + 2] = b;
            rgba_pixels[offset + 3] = 255; // Alpha
        }

        // Kreiranje ili ažuriranje egui teksture sa NEAREST filtriranjem (pixel art izgled)
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            [VGA_WIDTH, VGA_HEIGHT],
            &rgba_pixels,
        );

        let texture = self.texture.get_or_insert_with(|| {
            ctx.load_texture(
                "vga_framebuffer",
                color_image.clone(),
                egui::TextureOptions::NEAREST, // Retro retro pikseli bez zamućenja!
            )
        });
        texture.set(color_image, egui::TextureOptions::NEAREST);

        // Prikaz sa skalirom u egui prozoru uz očuvanje 4:3 / 16:10 razmere
        let available_width = ui.available_width();
        let display_height = available_width * (VGA_HEIGHT as f32 / VGA_WIDTH as f32);
        
        ui.centered_and_justified(|ui| {
            ui.image((texture.id(), egui::vec2(available_width, display_height)));
        });
    }
}