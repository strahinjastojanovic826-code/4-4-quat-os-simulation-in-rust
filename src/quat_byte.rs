use eframe::egui;
use crate::kernel::QuatKernel;

#[derive(Debug, Clone, PartialEq)]
pub enum ByteCategory {
    DSPChiptuneSong,
    AssemblySpeedrun,
    SpriteArt,
    TechPitch,
}

pub struct QuatByteAudition {
    pub category: ByteCategory,
    pub user_input: String,
    pub buzzers: [bool; 4], // 4 Quat zujalice
    pub simon_dialogue: String,
    pub simon_mood: &'static str, // "Skeptičan", "Besan", "Zgađen", "Malo Impresioniran"
    pub audition_active: bool,
    pub score: u8,
    pub stage_timer: f32,
}

impl QuatByteAudition {
    pub fn new() -> Self {
        Self {
            category: ByteCategory::DSPChiptuneSong,
            user_input: String::new(),
            buzzers: [false; 4],
            simon_dialogue: "Izađi na binu. Nemam ceo dan, registri se pregrevaju. Šta si nam spremio?".to_string(),
            simon_mood: "Skeptičan",
            audition_active: false,
            score: 0,
            stage_timer: 0.0,
        }
    }

    /// Sajmonov generator brutalnih replika i argumenata
    pub fn simon_evaluate(&mut self) {
        let active_buzzers = self.buzzers.iter().filter(|&&b| b).count();

        if active_buzzers == 4 {
            self.simon_dialogue = "DOSTA! 4 CRVENA X-A! Ovo je bio najgori nastup u istoriji $4^4$ arhitekture. Napusti binu odmah!".to_string();
            self.simon_mood = "Zgađen";
            self.audition_active = false;
            return;
        }

        match self.category {
            ByteCategory::DSPChiptuneSong => {
                if self.user_input.to_lowercase().contains("bas") || self.user_input.to_lowercase().contains("ritam") {
                    self.simon_dialogue = "Hteo si ritam? Dobio sam glavobolju! Tvoj DSP šum zvuči kao pokvareni veš mašina u ERAM-u.".to_string();
                    self.buzz_next();
                    self.simon_mood = "Besan";
                } else {
                    self.simon_dialogue = "Hm... nije potpuno grozno. Zvuči kao osrednja igra za Commodore 64. Nastavi, slušam...".to_string();
                    self.simon_mood = "Malo Impresioniran";
                    self.score += 10;
                }
            }
            ByteCategory::AssemblySpeedrun => {
                if self.user_input.contains("NOP") || self.user_input.trim().is_empty() {
                    self.simon_dialogue = "Trošiš moje procesorske cikluse na NOP instrukcije?! Zujalica! Nemaš pojma sa registrima!".to_string();
                    self.buzz_next();
                    self.simon_mood = "Besan";
                } else if self.user_input.contains("MOV") || self.user_input.contains("JMP") {
                    self.simon_dialogue = "Konačno neko ko razume sintaksu. Ali optimizacija ti je nula. Zasto si koristio B registar umesto A?!".to_string();
                    self.score += 25;
                    self.simon_mood = "Skeptičan";
                } else {
                    self.simon_dialogue = "Šta je ovo?! Sintaksna greška! Moji akumulatori krvare od ovog koda!".to_string();
                    self.buzz_next();
                }
            }
            ByteCategory::SpriteArt => {
                self.simon_dialogue = "Tvoj sprajt ima 4 boje i sve četiri su pogrešne. Izgleda kao artifikacija pregrejanog VRAM-a.".to_string();
                self.buzz_next();
                self.simon_mood = "Zgađen";
            }
            ByteCategory::TechPitch => {
                if self.user_input.len() > 30 {
                    self.simon_dialogue = "Previše reči! Tvome konceptu treba više od 256B RAM-a, a to je neoprostivo u mom studiju!".to_string();
                    self.buzz_next();
                    self.simon_mood = "Besan";
                } else {
                    self.simon_dialogue = "Kratko, ali da li je izvodljivo na 1.00 MHz? Dokaži mi ili letiš napolje!".to_string();
                    self.score += 15;
                }
            }
        }
    }

    fn buzz_next(&mut self) {
        for b in self.buzzers.iter_mut() {
            if !*b {
                *b = true;
                break;
            }
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, _kernel: &mut QuatKernel) {
        ui.heading("🎬 QGT: Quat's Got Byte (Sajmonov Audicioni Sud)");
        ui.label("Pokaži svoj retro-byte pre nego što ti Sajmon Kvat spali procesor argumentima!");
        ui.separator();

        // 1. PRIKAZ SAJMONOVIH ZUJALICA (4 QUAT BUZZERS)
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("SUDIJSKI BUZZERI:").strong());
            ui.add_space(10.0);
            for (idx, &b) in self.buzzers.iter().enumerate() {
                let color = if b { egui::Color32::RED } else { egui::Color32::DARK_GRAY };
                let text = if b { "✖ X ✖" } else { "  O  " };
                
                ui.group(|ui| {
                    ui.label(egui::RichText::new(format!("Q{}: {}", idx, text)).color(color).strong().size(18.0));
                });
            }
        });

        ui.add_space(15.0);

        // 2. SAJMONOV PANEL & MOOD
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("👨‍⚖️ Sajmon Kvat:").strong().size(16.0));
                ui.label(format!(" Status: [{}]", self.simon_mood));
            });
            ui.separator();
            ui.label(egui::RichText::new(format!("\"{}\"", self.simon_dialogue)).italics().size(14.0).color(egui::Color32::LIGHT_YELLOW));
        });

        ui.add_space(15.0);

        // 3. KONTROLE NASTUPA / AUDICIJE
        if !self.audition_active {
            ui.group(|ui| {
                ui.label(egui::RichText::new("1. Izaberi kategoriju sa kojom izlaziš na binu:").strong());
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.category, ByteCategory::DSPChiptuneSong, "🎵 DSP Chiptune");
                    ui.selectable_value(&mut self.category, ByteCategory::AssemblySpeedrun, "⚡ ASM Speedrun");
                    ui.selectable_value(&mut self.category, ByteCategory::SpriteArt, "🎨 Sprajt Dizajn");
                    ui.selectable_value(&mut self.category, ByteCategory::TechPitch, "💡 256B Ideja");
                });

                ui.add_space(10.0);
                if ui.button("🚀 IZAĐI NA BINU (Započni Audiciju)").clicked() {
                    self.audition_active = true;
                    self.buzzers = [false; 4];
                    self.score = 0;
                    self.simon_dialogue = "Reflektori su na tebi. Pokaži šta znaš ili me ubedi argumentima!".to_string();
                    self.simon_mood = "Skeptičan";
                }
            });
        } else {
            ui.group(|ui| {
                ui.label(egui::RichText::new("🎤 Tvoj Nastup u Realnom Vremenu:").strong());
                ui.label("Unesi tvoj izvođenje/kod/tekst pred Sajmonom:");
                
                let re = ui.text_edit_singleline(&mut self.user_input);
                
                ui.horizontal(|ui| {
                    if ui.button("🔥 Izvedi / Pošalji Sajmonu").clicked() || (re.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                        self.simon_evaluate();
                    }

                    if ui.button("🏳 Predaj se (Napusti binu)").clicked() {
                        self.audition_active = false;
                        self.simon_dialogue = "Mudrom odlukom si izbegao potpunu sramotu. Sledeći!".to_string();
                    }
                });
            });
        }

        ui.add_space(15.0);
        ui.label(format!("🏆 Ukupni Osvojeni Bodovi: {} / 100", self.score));
    }
}