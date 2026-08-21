use eframe::egui;
use crate::kernel::{QuatByte, QuatKernel};
use std::collections::VecDeque;

// ============================================================================
// 1. STRUKTURE ZA PROCESE I DIJAGNOSTIKU
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProcessState {
    Running,
    Ready,
    Paused,
    Blocked,
    Terminated,
}

#[derive(Debug, Clone)]
pub struct QuatProcess {
    pub pid: u8,
    pub name: String,
    pub state: ProcessState,
    pub priority: u8,       // 0 do 3 (Kvatni prioritet)
    pub mem_base: u8,       // Početna adresa u RAM-u
    pub mem_limit: u8,      // Veličina zauzeća u RAM-u
    pub cpu_cycles: u64,
    pub quat_core_id: usize,// Na kom od 4 jezgra se izvršava (0-3)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActiveTab {
    Overview,
    Processes,
    CpuGraph,
    RamHeatmap,
    GpuInspector,
    StorageDisk,
}

// ============================================================================
// 2. MAIN TASK MANAGER CLASS
// ============================================================================

pub struct TaskManager {
    pub active_tab: ActiveTab,
    pub processes: Vec<QuatProcess>,
    
    // Istorijski podaci za crtanje grafika (Zadržavamo poslednjih 60 merenja)
    pub cpu_history: [VecDeque<f32>; 4], // Grafici za 4 Quat jezgra
    pub ram_history: VecDeque<f32>,
    pub gpu_fps_history: VecDeque<f32>,
    pub io_read_history: VecDeque<f32>,
    pub io_write_history: VecDeque<f32>,
    
    // Selektovani bajt u RAM toplotnoj mapi za inspekciju
    pub selected_ram_addr: Option<u8>,
    
    // Metrike Storage-a i GPU-a
    pub disk_sectors_used: [bool; 64], // 64 sektora x 4 bajta = 256B Disk
    pub io_read_bytes: u32,
    pub io_write_bytes: u32,
    
    pub auto_refresh: bool,
    pub tick_counter: u64,
}

impl TaskManager {
    pub fn new() -> Self {
        let mut cpu_history = [
            VecDeque::with_capacity(60),
            VecDeque::with_capacity(60),
            VecDeque::with_capacity(60),
            VecDeque::with_capacity(60),
        ];
        
        for ch in 0..4 {
            for _ in 0..60 {
                cpu_history[ch].push_back(0.1);
            }
        }

        let mut ram_hist = VecDeque::with_capacity(60);
        let mut gpu_hist = VecDeque::with_capacity(60);
        let mut io_r_hist = VecDeque::with_capacity(60);
        let mut io_w_hist = VecDeque::with_capacity(60);

        for _ in 0..60 {
            ram_hist.push_back(32.0); // 32 od 256B počiinje zauzeto
            gpu_hist.push_back(60.0);
            io_r_hist.push_back(0.0);
            io_w_hist.push_back(0.0);
        }

        // Dummy sektori na disku (Prvih 8 zauzeto sistemskim fajlovima)
        let mut disk_sectors = [false; 64];
        for i in 0..8 { disk_sectors[i] = true; }

        let demo_processes = vec![
            QuatProcess {
                pid: 0,
                name: "kernel.sys".to_string(),
                state: ProcessState::Running,
                priority: 3,
                mem_base: 0x00,
                mem_limit: 0x1F,
                cpu_cycles: 14205,
                quat_core_id: 0,
            },
            QuatProcess {
                pid: 1,
                name: "quat_dos.sh".to_string(),
                state: ProcessState::Ready,
                priority: 2,
                mem_base: 0x20,
                mem_limit: 0x3F,
                cpu_cycles: 3820,
                quat_core_id: 1,
            },
            QuatProcess {
                pid: 2,
                name: "assembler_ide".to_string(),
                state: ProcessState::Running,
                priority: 1,
                mem_base: 0x40,
                mem_limit: 0x7F,
                cpu_cycles: 8940,
                quat_core_id: 2,
            },
            QuatProcess {
                pid: 3,
                name: "chiptune_dsp".to_string(),
                state: ProcessState::Paused,
                priority: 1,
                mem_base: 0x80,
                mem_limit: 0x9F,
                cpu_cycles: 1200,
                quat_core_id: 3,
            },
        ];

        Self {
            active_tab: ActiveTab::Overview,
            processes: demo_processes,
            cpu_history,
            ram_history: ram_hist,
            gpu_fps_history: gpu_hist,
            io_read_history: io_r_hist,
            io_write_history: io_w_hist,
            selected_ram_addr: None,
            disk_sectors_used: disk_sectors,
            io_read_bytes: 1024,
            io_write_bytes: 256,
            auto_refresh: true,
            tick_counter: 0,
        }
    }

    /// Simulacija ažuriranja metrika sistemskih resursa
    pub fn update_metrics(&mut self, kernel: &QuatKernel) {
        self.tick_counter += 1;

        // Simulacija opterećenja 4 Quat jezgra
        for core in 0..4 {
            let base_load = match core {
                0 => 0.45, // Kernel Core
                1 => 0.20, // DOS Core
                2 => 0.70, // Compiler Core
                _ => 0.10, // DSP Core
            };
            let noise = ((self.tick_counter.wrapping_mul(core as u64 + 1) % 20) as f32 - 10.0) / 100.0;
            let val = (base_load + noise).clamp(0.05, 0.98);
            
            self.cpu_history[core].pop_front();
            self.cpu_history[core].push_back(val);
        }

        // RAM zauzeće iz realnog kernela
        let mut used_ram = 0;
        for b in kernel.ram.iter() {
            if b.0 != 0 { used_ram += 1; }
        }
        self.ram_history.pop_front();
        self.ram_history.push_back(used_ram as f32);

        // Simulated GPU FPS & Storage I/O
        self.gpu_fps_history.pop_front();
        self.gpu_fps_history.push_back(58.0 + (self.tick_counter % 5) as f32);

        self.io_read_history.pop_front();
        self.io_read_history.push_back((self.tick_counter % 4 * 16) as f32);

        self.io_write_history.pop_front();
        self.io_write_history.push_back((self.tick_counter % 3 * 8) as f32);
    }

    /// Renders custom line chart using egui Painter
    fn draw_performance_chart(&self, ui: &mut egui::Ui, data: &VecDeque<f32>, max_val: f32, line_color: egui::Color32, height: f32) {
        let (response, painter) = ui.allocate_painter(egui::vec2(ui.available_width(), height), egui::Sense::hover());
        let rect = response.rect;

        // Pozadina i mreža
        painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(18, 22, 28));
        painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0, egui::Color32::from_gray(50)));

        // Crtanje grid linija
        for i in 1..4 {
            let y = rect.top() + (rect.height() / 4.0) * i as f32;
            painter.line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                egui::Stroke::new(0.5, egui::Color32::from_gray(35)),
            );
        }

        if data.len() < 2 { return; }

        let step_x = rect.width() / (data.len() - 1) as f32;
        let points: Vec<egui::Pos2> = data.iter().enumerate().map(|(idx, &val)| {
            let x = rect.left() + idx as f32 * step_x;
            let norm_val = (val / max_val).clamp(0.0, 1.0);
            let y = rect.bottom() - norm_val * rect.height();
            egui::pos2(x, y)
        }).collect();

        // Crtanje linije grafikona
        for window in points.windows(2) {
            painter.line_segment([window[0], window[1]], egui::Stroke::new(2.0, line_color));
        }
    }

    /// Haupt UI metoda Task Managera
    pub fn ui(&mut self, ui: &mut egui::Ui, kernel: &mut QuatKernel) {
        if self.auto_refresh {
            self.update_metrics(kernel);
        }

        ui.heading("📊 QuatOS Task Manager & System Diagnostic Monitor");
        ui.separator();

        // TAB MENTU
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.active_tab, ActiveTab::Overview, "🖥 Pregled");
            ui.selectable_value(&mut self.active_tab, ActiveTab::Processes, "⚙ Procesi");
            ui.selectable_value(&mut self.active_tab, ActiveTab::CpuGraph, "⚡ CPU (4 Cores)");
            ui.selectable_value(&mut self.active_tab, ActiveTab::RamHeatmap, "🧠 RAM Heatmap (256B)");
            ui.selectable_value(&mut self.active_tab, ActiveTab::GpuInspector, "🎨 GPU & VRAM");
            ui.selectable_value(&mut self.active_tab, ActiveTab::StorageDisk, "💾 Storage Disk");

            ui.add_space(20.0);
            ui.checkbox(&mut self.auto_refresh, "Auto-refresh");
        });

        ui.separator();

        match self.active_tab {
            ActiveTab::Overview => self.render_overview(ui, kernel),
            ActiveTab::Processes => self.render_processes(ui),
            ActiveTab::CpuGraph => self.render_cpu_tab(ui),
            ActiveTab::RamHeatmap => self.render_ram_heatmap(ui, kernel),
            ActiveTab::GpuInspector => self.render_gpu_tab(ui),
            ActiveTab::StorageDisk => self.render_storage_tab(ui),
        }
    }

    // ========================================================================
    // TAB RENDERS
    // ========================================================================

    fn render_overview(&mut self, ui: &mut egui::Ui, kernel: &QuatKernel) {
        ui.columns(2, |cols| {
            cols[0].vertical(|ui| {
                ui.group(|ui| {
                    ui.label(egui::RichText::new("💻 Sistemske Informacije").strong());
                    ui.label(format!("Arhitektura: 4-Quat / 8-Bit ($4^4$)"));
                    ui.label(format!("Ukupna RAM Memorija: 256 Bajtova (0x00 - 0xFF)"));
                    ui.label(format!("Radni Takt Procesora: 1.00 MHz"));
                    ui.label(format!("Virtuelna Jezgra: 4 Quat-Core jedinice"));
                });

                ui.add_space(10.0);
                ui.label("⚡ Prosek CPU Zauzeća:");
                let avg_cpu: f32 = self.cpu_history.iter().map(|h| *h.back().unwrap_or(&0.0)).sum::<f32>() / 4.0;
                self.draw_performance_chart(ui, &self.cpu_history[0], 1.0, egui::Color32::GREEN, 80.0);
                ui.label(format!("Trenutno: {:.1}%", avg_cpu * 100.0));
            });

            cols[1].vertical(|ui| {
                ui.group(|ui| {
                    ui.label(egui::RichText::new("📈 Status Resursa").strong());
                    let used_ram = *self.ram_history.back().unwrap_or(&0.0);
                    ui.label(format!("RAM Popunjenost: {:.0} / 256 B ({:.1}%)", used_ram, (used_ram / 256.0) * 100.0));
                    ui.label(format!("Aktivni Procesi: {}", self.processes.len()));
                    ui.label(format!("GPU Render Speed: {:.0} FPS", *self.gpu_fps_history.back().unwrap_or(&60.0)));
                    ui.label(format!("Disk Čitanje/Pisanje: {} / {} B", self.io_read_bytes, self.io_write_bytes));
                });

                ui.add_space(10.0);
                ui.label("🧠 Zauzeće RAM Memorije:");
                self.draw_performance_chart(ui, &self.ram_history, 256.0, egui::Color32::LIGHT_BLUE, 80.0);
            });
        });
    }

    fn render_processes(&mut self, ui: &mut egui::Ui) {
        ui.label("📋 Lista Aktivnih Procesa Operativnog Sistema:");
        ui.add_space(5.0);

        egui::Grid::new("process_grid").striped(true).min_col_width(70.0).show(ui, |ui| {
            ui.label("PID");
            ui.label("Ime Procesu");
            ui.label("Stanje");
            ui.label("Prioritet");
            ui.label("RAM Opseg");
            ui.label("CPU Core");
            ui.label("Ciklusi");
            ui.label("Akcije");
            ui.end_row();

            let mut process_to_kill: Option<u8> = None;

            //Zlatni Task manager
            //Uvek bude ogroman da ga prosto
            //Ne budis iz zimskog sna

            for proc in self.processes.iter_mut() {
                ui.label(format!("{:02X}", proc.pid));
                ui.label(&proc.name);

                let (state_str, color) = match proc.state {
                    ProcessState::Running => ("RUNNING", egui::Color32::GREEN),
                    ProcessState::Ready => ("READY", egui::Color32::LIGHT_BLUE),
                    ProcessState::Paused => ("PAUSED", egui::Color32::YELLOW),
                    ProcessState::Blocked => ("BLOCKED", egui::Color32::RED),
                    ProcessState::Terminated => ("DEAD", egui::Color32::GRAY),
                };
                ui.label(egui::RichText::new(state_str).color(color).strong());

                ui.label(format!("Q-{}", proc.priority));
                ui.label(format!("0x{:02X}-0x{:02X}", proc.mem_base, proc.mem_limit));
                ui.label(format!("Core {}", proc.quat_core_id));
                ui.label(format!("{}", proc.cpu_cycles));

                ui.horizontal(|ui| {
                    if proc.state == ProcessState::Running {
                        if ui.button("⏸ Pause").clicked() {
                            proc.state = ProcessState::Paused;
                        }
                    } else if proc.state == ProcessState::Paused {
                        if ui.button("▶ Resume").clicked() {
                            proc.state = ProcessState::Running;
                        }
                    }
                    if ui.button("❌ Kill").clicked() {
                        process_to_kill = Some(proc.pid);
                    }
                });

                ui.end_row();
            }

            if let Some(pid) = process_to_kill {
                self.processes.retain(|p| p.pid != pid);
            }
        });
    }

    fn render_cpu_tab(&mut self, ui: &mut egui::Ui) {
        ui.label("⚡ Analiza Opterećenja 4 Virtuelna Quat-Jezgra:");
        ui.add_space(5.0);

        ui.columns(2, |cols| {
            for i in 0..4 {
                let col_idx = i % 2;
                cols[col_idx].group(|ui| {
                    let cur_load = *self.cpu_history[i].back().unwrap_or(&0.0) * 100.0;
                    ui.label(egui::RichText::new(format!("Quat-Core #{} Load: {:.1}%", i, cur_load)).strong());
                    self.draw_performance_chart(ui, &self.cpu_history[i], 1.0, egui::Color32::LIGHT_RED, 100.0);
                });
                if i == 1 { cols[0].add_space(10.0); cols[1].add_space(10.0); }
            }
        });
    }

    fn render_ram_heatmap(&mut self, ui: &mut egui::Ui, kernel: &mut QuatKernel) {
        ui.label("🧠 Toplotna Mapa RAM Memorije (256 Bajtova / $4^4$ Grid):");
        ui.label("Legenda: [Zeleno = Kôd] [Plavo = VRAM/Grafika] [Sivo = Prazno] [Crveno = Stek]");
        ui.separator();

        ui.columns(2, |cols| {
            // MATRICA 16x16
            cols[0].vertical(|ui| {
                egui::Grid::new("ram_matrix").spacing([3.0, 3.0]).show(ui, |ui| {
                    for row in 0..16 {
                        ui.label(format!("{:01X}x", row));
                        for col in 0..16 {
                            let addr = (row * 16 + col) as u8;
                            let val = kernel.ram[addr as usize].0;

                            let bg_color = match addr {
                                0x00..=0x3F => egui::Color32::from_rgb(20, 80, 40), // Code region
                                0x40..=0x9F => egui::Color32::from_rgb(30, 50, 90), // Data/VRAM region
                                0xE0..=0xFF => egui::Color32::from_rgb(100, 30, 30), // Stack region
                                _ => if val != 0 { egui::Color32::from_rgb(60, 60, 60) } else { egui::Color32::from_rgb(25, 25, 25) },
                            };

                            let text_color = if self.selected_ram_addr == Some(addr) {
                                egui::Color32::YELLOW
                            } else {
                                egui::Color32::WHITE
                            };

                            let btn = egui::Button::new(egui::RichText::new(format!("{:02X}", val)).monospace().color(text_color).size(11.0))
                                .fill(bg_color);

                            if ui.add(btn).clicked() {
                                self.selected_ram_addr = Some(addr);
                            }
                        }
                        ui.end_row();
                    }
                });
            });

            // EDICIJA I INSPEKCIJA SELEKTOVANOG BAJTA
            cols[1].vertical(|ui| {
                ui.group(|ui| {
                    ui.label(egui::RichText::new("🔍 Inspekcija Selektovanog Bajta").strong());
                    if let Some(addr) = self.selected_ram_addr {
                        let byte_ref = &mut kernel.ram[addr as usize];
                        ui.label(format!("Adresa: 0x{:02X} (Dekadno: {})", addr, addr));
                        ui.label(format!("Vrednost (Hex): 0x{:02X}", byte_ref.0));
                        ui.label(format!("Vrednost (Dec): {}", byte_ref.0));
                        
                        let q = byte_ref.to_quats();
                        ui.label(format!("4-Quat Notacija: [Q3:{} Q2:{} Q1:{} Q0:{}]", q[3], q[2], q[1], q[0]));

                        ui.separator();
                        ui.label("✏ Izmeni vrednost bajta:");
                        let mut val_u8 = byte_ref.0;
                        if ui.add(egui::DragValue::new(&mut val_u8).clamp_range(0..=255)).changed() {
                            byte_ref.0 = val_u8;
                        }
                    } else {
                        ui.label("Klikni na bilo koja vrata/bajt u matrici levo radi inspekcije.");
                    }
                });
            });
        });
    }

    fn render_gpu_tab(&mut self, ui: &mut egui::Ui) {
        ui.label("🎨 Grafički Subsistem (QuatVGA Engine & CRT Pipeline):");
        ui.separator();

        ui.columns(2, |cols| {
            cols[0].vertical(|ui| {
                ui.label("📈 FPS Grafik:");
                self.draw_performance_chart(ui, &self.gpu_fps_history, 75.0, egui::Color32::GOLD, 100.0);
                
                ui.add_space(10.0);
                ui.group(|ui| {
                    ui.label(egui::RichText::new("📊 VRAM & Display Status").strong());
                    ui.label("Rezolucija: 160 x 120 Piksela (4:3)");
                    ui.label("Dubina Boja: 4-Bit Kvatska Paleta (16 Boja)");
                    ui.label("VRAM Zauzeće: 9,600 Bajtova (Mapirano u ERAM)");
                    ui.label("Aktivan CRT Filter: Trinitron ShadowMask");
                });
            });

            cols[1].vertical(|ui| {
                ui.group(|ui| {
                    ui.label(egui::RichText::new("🕹 Hardware Sprite Engine Status").strong());
                    ui.label("Maksimalno Sprajtova: 16 Hardware Sprites");
                    ui.label("Aktivni Sprajtovi: 4");
                    ui.label("Kolizije Detektovane: 0");
                    ui.label("Scanline Rendering Rate: 15.75 kHz");
                });
            });
        });
    }

    fn render_storage_tab(&mut self, ui: &mut egui::Ui) {
        ui.label("💾 QuatDisk Storage & Sector Map (256 B Floppy Simulator):");
        ui.separator();

        ui.columns(2, |cols| {
            cols[0].vertical(|ui| {
                ui.label("🔲 Sektorska Mapa Diska (64 Sektora x 4 Bajta):");
                ui.label("[Plavo = Zauzet Sektor] [Crno = Slobodan Sektor]");
                ui.add_space(5.0);

                egui::Grid::new("sector_grid").spacing([4.0, 4.0]).show(ui, |ui| {
                    for row in 0..8 {
                        for col in 0..8 {
                            let sec_idx = row * 8 + col;
                            let used = self.disk_sectors_used[sec_idx];
                            let color = if used { egui::Color32::LIGHT_BLUE } else { egui::Color32::BLACK };

                            let (resp, painter) = ui.allocate_painter(egui::vec2(22.0, 22.0), egui::Sense::hover());
                            painter.rect_filled(resp.rect, 2.0, color);
                            painter.rect_stroke(resp.rect, 2.0, egui::Stroke::new(1.0, egui::Color32::GRAY));
                        }
                        ui.end_row();
                    }
                });
            });

            cols[1].vertical(|ui| {
                ui.label("📊 Storage I/O Protoci (Throughput):");
                ui.label("Čitanje sa diska (Read B/s):");
                self.draw_performance_chart(ui, &self.io_read_history, 64.0, egui::Color32::GREEN, 60.0);
                
                ui.label("Pisanje na disk (Write B/s):");
                self.draw_performance_chart(ui, &self.io_write_history, 64.0, egui::Color32::LIGHT_RED, 60.0);
            });
        });
    }
}