use eframe::egui;
use rand::Rng; // Koristimo rand za proceduralne elemente (ako imaš u Cargo.toml, ili prosti pseudo-PRNG)

#[derive(Debug, Clone, PartialEq)]
pub enum ActiveGame {
    None,
    QuatCombat,
    SiliconRunner,
    SpaceTrader,
}

pub struct GamesManager {
    pub active_game: ActiveGame,
    // 1. QuatCombat State
    pub player_hp: i32,
    pub player_heat: i32,
    pub enemy_hp: i32,
    pub combat_log: Vec<String>,

    // 2. SiliconRunner State
    pub runner_pos: (usize, usize),
    pub runner_hp: i32,
    pub runner_bytes: u32,
    pub grid: [[char; 8]; 8],
    pub dungeon_log: String,

    // 3. SpaceTrader State
    pub credits: u32,
    pub cargo_silicon: u32,
    pub current_planet: usize,
    pub market_prices: [u32; 4],
    pub trader_log: String,
}

impl GamesManager {
    pub fn new() -> Self {
        let mut mgr = Self {
            active_game: ActiveGame::None,
            player_hp: 100,
            player_heat: 0,
            enemy_hp: 100,
            combat_log: vec!["Dobrodošao u QuatCombat Arenu! Choose your action.".to_string()],

            runner_pos: (0, 0),
            runner_hp: 50,
            runner_bytes: 0,
            grid: [['.'; 8]; 8],
            dungeon_log: "Sišao si u korumpirani VRAM sektor...".to_string(),

            credits: 100,
            cargo_silicon: 0,
            current_planet: 0,
            market_prices: [10, 25, 40, 15],
            trader_log: "Sleteo si na Planetu Q0. Tržište je stabilno.".to_string(),
        };
        mgr.init_runner_grid();
        mgr
    }

    fn init_runner_grid(&mut self) {
        self.grid = [['.'; 8]; 8];
        self.grid[0][0] = 'P'; // Player
        self.grid[7][7] = 'E'; // Exit
        // Par prepreka i korumpiranih bajtova
        self.grid[2][3] = 'X';
        self.grid[4][1] = 'X';
        self.grid[5][5] = 'B'; // Byte pickup
        self.grid[1][6] = 'B';
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("🎮 $4^4$ System Games Studio");
        ui.label("Napredne 8-bitne simulacije i strategije prilagođene 4-kvatnom procesoru.");
        ui.separator();

        // Meni za izbor igara
        ui.horizontal(|ui| {
            if ui.selectable_label(self.active_game == ActiveGame::QuatCombat, "⚔ QuatCombat (Taktika)").clicked() {
                self.active_game = ActiveGame::QuatCombat;
            }
            if ui.selectable_label(self.active_game == ActiveGame::SiliconRunner, "🤖 SiliconRunner (Cyberpunk RPG)").clicked() {
                self.active_game = ActiveGame::SiliconRunner;
            }
            if ui.selectable_label(self.active_game == ActiveGame::SpaceTrader, "🚀 SpaceTrader256 (Ekonomija)").clicked() {
                self.active_game = ActiveGame::SpaceTrader;
            }
        });

        ui.add_space(10.0);

        // Prikaz izabrane igre
        match self.active_game {
            ActiveGame::None => {
                ui.label("Izaberi igru sa gornjeg menija da započneš simulaciju.");
            }
            ActiveGame::QuatCombat => self.render_quat_combat(ui),
            ActiveGame::SiliconRunner => self.render_silicon_runner(ui),
            ActiveGame::SpaceTrader => self.render_space_trader(ui),
        }
    }

    // =========================================================================
    // 1. QUATCOMBAT (Tactical Mech Arena)
    // =========================================================================
    fn render_quat_combat(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label(egui::RichText::new("⚔ QuatCombat: Mech Tactical Engine").strong().size(16.0));
            ui.separator();

            ui.columns(2, |cols| {
                cols[0].group(|ui| {
                    ui.label(egui::RichText::new("🤖 Tvoj Mech ($4^4$ Frame)").strong());
                    ui.label(format!("HP: {}/100", self.player_hp));
                    ui.label(format!("Toplota (Heat): {}%", self.player_heat));
                });

                cols[1].group(|ui| {
                    ui.label(egui::RichText::new("🔴 AI Prototip (Enemy)").strong());
                    ui.label(format!("HP: {}/100", self.enemy_hp));
                    ui.label("Štitovi: Aktivni");
                });
            });

            ui.add_space(10.0);

            // Komande za borbu
            ui.horizontal(|ui| {
                if ui.button("⚡ Plazma Laser (20 DMG, +15 Heat)").clicked() {
                    if self.player_heat <= 85 {
                        self.enemy_hp -= 20;
                        self.player_heat += 15;
                        self.combat_log.push("Pogodio si protivnika Plazma Laserom! (-20 HP)".to_string());
                        self.enemy_turn();
                    } else {
                        self.combat_log.push("⚠️ PREGREVANJE! Moraš ohladiti sisteme!".to_string());
                    }
                }

                if ui.button("🛡 Hlađenje & Štit (-30 Heat, +10 HP)").clicked() {
                    self.player_heat = (self.player_heat - 30).max(0);
                    self.player_hp = (self.player_hp + 10).min(100);
                    self.combat_log.push("Sistemi rashlađeni, rekonfigurisani štitovi.".to_string());
                    self.enemy_turn();
                }

                if ui.button("💥 EMP Kvat-Bomba (40 DMG, +50 Heat)").clicked() {
                    if self.player_heat <= 50 {
                        self.enemy_hp -= 40;
                        self.player_heat += 50;
                        self.combat_log.push("ISPALJEN EMP! Težak udarac na AI jezgro!".to_string());
                        self.enemy_turn();
                    } else {
                        self.combat_log.push("⚠️ Rizik od spaljivanja registara! Akcija odbijena.".to_string());
                    }
                }
            });

            ui.add_space(10.0);
            ui.label(egui::RichText::new("Log Borbe:").strong());
            egui::ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                for log in self.combat_log.iter().rev() {
                    ui.label(log);
                }
            });
        });
    }

    fn enemy_turn(&mut self) {
        if self.enemy_hp <= 0 {
            self.combat_log.push("🏆 POBEDA! AI Prototip je uništen!".to_string());
            return;
        }
        // Jednostavna logika neprijatelja
        self.player_hp -= 12;
        self.combat_log.push("🤖 Neprijateljski Mech te je pogodio Kvat-Topom! (-12 HP)".to_string());
    }

    // =========================================================================
    // 2. SILICONRUNNER (Micro Cyberpunk Dungeon Crawler)
    // =========================================================================
    fn render_silicon_runner(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label(egui::RichText::new("🤖 SiliconRunner: VRAM Crawler").strong().size(16.0));
            ui.label(format!("Status: HP [{}] | Sakupio Bajtova: [{} B]", self.runner_hp, self.runner_bytes));
            ui.separator();

            // Prikaz 8x8 VRAM mreže
            egui::Grid::new("runner_grid").spacing([4.0, 4.0]).show(ui, |ui| {
                for y in 0..8 {
                    for x in 0..8 {
                        let symbol = self.grid[y][x];
                        let color = match symbol {
                            'P' => egui::Color32::GREEN,      // Player
                            'E' => egui::Color32::GOLD,       // Exit
                            'X' => egui::Color32::RED,        // Corrupted Sector
                            'B' => egui::Color32::LIGHT_BLUE, // Byte
                            _ => egui::Color32::GRAY,
                        };
                        ui.label(egui::RichText::new(format!("[{}]", symbol)).color(color).monospace());
                    }
                    ui.end_row();
                }
            });

            ui.add_space(10.0);

            // Kontrole kretanja
            ui.horizontal(|ui| {
                if ui.button("⬆ Gore").clicked() { self.move_runner(0, -1); }
                if ui.button("⬇ Dole").clicked() { self.move_runner(0, 1); }
                if ui.button("⬅ Levo").clicked() { self.move_runner(-1, 0); }
                if ui.button("➡ Desno").clicked() { self.move_runner(1, 0); }
            });

            ui.label(format!("Sistemska poruka: {}", self.dungeon_log));
        });
    }

    fn move_runner(&mut self, dx: i32, dy: i32) {
        let new_x = self.runner_pos.0 as i32 + dx;
        let new_y = self.runner_pos.1 as i32 + dy;

        if new_x >= 0 && new_x < 8 && new_y >= 0 && new_y < 8 {
            let (nx, ny) = (new_x as usize, new_y as usize);
            
            match self.grid[ny][nx] {
                'X' => {
                    self.runner_hp -= 15;
                    self.dungeon_log = "⚠️ KORUMPIRAN SEKTOR! Pretrpeo si štetu od Memory Leak-a (-15 HP)".to_string();
                }
                'B' => {
                    self.runner_bytes += 64;
                    self.dungeon_log = "💎 Sakupio si 64 Bajta čiste memorije!".to_string();
                }
                'E' => {
                    self.dungeon_log = "🎉 STIGAO SI DO IZLAZA! Sektor uspešno očaran!".to_string();
                }
                _ => {
                    self.dungeon_log = "Kretanje po registru...".to_string();
                }
            }

            self.grid[self.runner_pos.1][self.runner_pos.0] = '.';
            self.runner_pos = (nx, ny);
            self.grid[ny][nx] = 'P';
        }
    }

    // =========================================================================
    // 3. SPACETRADER256 (Interplanetary Economy Sim)
    // =========================================================================
    fn render_space_trader(&mut self, ui: &mut egui::Ui) {
        let planets = ["Planeta Q0 (Sirov Silicon)", "Planeta Q1 (DSP Čipovi)", "Planeta Q2 (EPROM Stanica)", "Planeta Q3 (Zvezdani Kvat)"];

        ui.group(|ui| {
            ui.label(egui::RichText::new("🚀 SpaceTrader256: Kvatna Ekonomija").strong().size(16.0));
            ui.label(format!("Krediti: {} CR | Teret (Silikon): {} jedinica", self.credits, self.cargo_silicon));
            ui.separator();

            ui.label(format!("Trenutna lokacija: {}", planets[self.current_planet]));
            ui.label(format!("Cena silikona ovde: {} CR / jedinica", self.market_prices[self.current_planet]));

            ui.add_space(10.0);

            // Trgovina
            ui.horizontal(|ui| {
                if ui.button("🛒 Kupi Silikon (1 kom)").clicked() {
                    let price = self.market_prices[self.current_planet];
                    if self.credits >= price {
                        self.credits -= price;
                        self.cargo_silicon += 1;
                        self.trader_log = format!("Kupljen 1 Silikon za {} CR.", price);
                    } else {
                        self.trader_log = "Nemate dovoljno kredita!".to_string();
                    }
                }

                if ui.button("💰 Prodaj Silikon (1 kom)").clicked() {
                    let price = self.market_prices[self.current_planet];
                    if self.cargo_silicon > 0 {
                        self.cargo_silicon -= 1;
                        self.credits += price;
                        self.trader_log = format!("Prodat 1 Silikon za {} CR.", price);
                    } else {
                        self.trader_log = "Nemate silikona u teretnjaku!".to_string();
                    }
                }
            });

            ui.add_space(10.0);
            ui.label(egui::RichText::new("Skoči na drugu planetu (Trosak 5 CR):").strong());

            ui.horizontal(|ui| {
                for (idx, name) in planets.iter().enumerate() {
                    if idx != self.current_planet {
                        if ui.button(format!("🚀 Sleti na Q{}", idx)).clicked() {
                            if self.credits >= 5 {
                                self.credits -= 5;
                                self.current_planet = idx;
                                // Fluktuacija cena pri skoku
                                self.market_prices[idx] = (self.market_prices[idx] as i32 + (idx as i32 * 3 - 2)).clamp(5, 80) as u32;
                                self.trader_log = format!("Uspešan skok na {}!", name);
                            }
                        }
                    }
                }
            });

            ui.add_space(10.0);
            ui.label(format!("Dnevnik broda: {}", self.trader_log));
        });
    }
}