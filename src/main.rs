mod kernel;
mod task_manager;
mod terminal;
mod assembler;
mod crt_screen;
mod quat_audio;
mod quat_basic;
mod quat_bios;
mod quat_disk;
mod quat_dos;
mod quat_dsp;
mod quat_fs;
mod quat_gpu;
mod quat_logic;
mod quat_net;
mod quat_pic;
mod quat_vga;
mod quat_vm;
mod quat_sprite;
mod quat_tracker;
mod quat_alu;
mod quat_bench;
mod quat_byte;
mod quat_tape;
mod games; 

use eframe::egui;
use kernel::QuatKernel;
use task_manager::TaskManager;
use terminal::QuatTerminal;
use assembler::Assembler;
use crt_screen::CrtScreen;
use quat_audio::QuatAudio;
use quat_basic::QuatBASIC;
use quat_bios::QuatBIOS;
use quat_disk::QuatDisk;
use quat_dos::QuatDOS;
use quat_dsp::QuatDSP;
use quat_fs::QuatFS;
use quat_gpu::QuatGPU;
use quat_logic::QuatLogic;
use quat_net::QuatNET;
use quat_pic::QuatPIC;
use quat_vga::QuatVGA;
use quat_vm::QuatVM;
use quat_tape::QuatTAPE;
use quat_sprite::QuatSpriteStudio;
use quat_tracker::QuatTracker;
use quat_alu::QuatALU;
use quat_bench::QuatBENCH;
use quat_byte::QuatByteAudition;
use games::GamesManager;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([950.0, 650.0])
            .with_title("QuatOS 4^4 Kernel & Virtual Environment"),
        ..Default::default()
    };

    eframe::run_native(
        "QuatOS 4^4 Engine",
        options,
        Box::new(|_| Box::new(QuatSimulatorApp::default())),
    )
}

struct QuatSimulatorApp {
    kernel: QuatKernel,
    task_manager: TaskManager,
    terminal: QuatTerminal,
    assembler: Assembler,
    crt_screen: CrtScreen,
    audio: QuatAudio,
    basic: QuatBASIC,
    bios: QuatBIOS,
    disk: QuatDisk,
    dos: QuatDOS,
    dsp: QuatDSP,
    fs: QuatFS,
    gpu: QuatGPU,
    logic: QuatLogic,
    net: QuatNET,
    pic: QuatPIC,
    vga: QuatVGA,
    vm: QuatVM,
    tape: QuatTAPE,
    spritestudio: QuatSpriteStudio,
    tracker: QuatTracker,
    alu: QuatALU,
    bench: QuatBENCH,
    byteaudition: QuatByteAudition,
    games: GamesManager,

    auto_step: bool,
    selected_app: String,
}

impl Default for QuatSimulatorApp {
    fn default() -> Self {
        Self {
            kernel: QuatKernel::new(),
            task_manager: TaskManager::new(),
            terminal: QuatTerminal::new(),
            assembler: Assembler::new(),
            crt_screen: CrtScreen::new(),
            audio: QuatAudio::new(),
            basic: QuatBASIC::new(),
            bios: QuatBIOS::new(),
            disk: QuatDisk::new(),
            dos: QuatDOS::new(),
            dsp: QuatDSP::new(),
            fs: QuatFS::new(),
            gpu: QuatGPU::new(),
            logic: QuatLogic::new(),
            net: QuatNET::new(),
            pic: QuatPIC::new(),
            vga: QuatVGA::new(),
            vm: QuatVM::new(),
            tape: QuatTAPE::new(),
            spritestudio: QuatSpriteStudio::new(),
            tracker: QuatTracker::new(),
            alu: QuatALU::new(),
            bench: QuatBENCH::new(),
            byteaudition: QuatByteAudition::new(),
            games: GamesManager::new(),

            auto_step: false,
            selected_app: "Sistemski Monitor".to_string(),
        }
    }
}

impl eframe::App for QuatSimulatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.auto_step && self.kernel.is_running {
            self.kernel.step();
            ctx.request_repaint();
        }

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("💻 QuatOS 4^4 System");
                ui.separator();
                if ui.button("▶ Start Engine").clicked() { self.kernel.is_running = true; self.auto_step = true; }
                if ui.button("⏸ Pauza").clicked() { self.auto_step = false; }
                if ui.button("⏭ 1 Takt").clicked() { self.kernel.step(); }
                if ui.button("🔄 Reset").clicked() { self.kernel = QuatKernel::new(); }
            });
        });

        // Meni sa leve strane - odabir alata
        egui::SidePanel::left("apps_panel").resizable(false).default_width(200.0).show(ctx, |ui| {
            ui.heading("📱 Alati & OS Moduli");
            ui.separator();
            ui.selectable_value(&mut self.selected_app, "Sistemski Monitor".to_string(), "📊 Monitor Registara");
            ui.selectable_value(&mut self.selected_app, "Task Manager".to_string(), "📈 Task Manager");
            ui.selectable_value(&mut self.selected_app, "Quat Terminal".to_string(), "🖥️ Terminal Shell");
            ui.selectable_value(&mut self.selected_app, "RAM Inspector".to_string(), "💾 256 RAM Matrica");

            ui.separator();

            ui.selectable_value(&mut self.selected_app, "Assembler".to_string(), "⚙️ Asembler");
            ui.selectable_value(&mut self.selected_app, "QuatDOS".to_string(), "📟 QuatDOS Shell");
            ui.selectable_value(&mut self.selected_app, "QuatBASIC".to_string(), "🔤 BASIC Interpreter");
            ui.selectable_value(&mut self.selected_app, "QuatVGA".to_string(), "📺 QuatVGA (Mode 13h)");
            ui.selectable_value(&mut self.selected_app, "QuatDisk".to_string(), "💾 QuatDisk (3.5\" FDD)");
            ui.selectable_value(&mut self.selected_app, "QuatAudio".to_string(), "🔊 QuatAudio");
            ui.selectable_value(&mut self.selected_app, "QuatGPU".to_string(), "🎮 QuatGPU");
            ui.selectable_value(&mut self.selected_app, "QuatNET".to_string(), "🌐 QuatNET");
            ui.selectable_value(&mut self.selected_app, "QuatPIC".to_string(), "🕹️ QuatPIC / Pad");
            ui.selectable_value(&mut self.selected_app, "QuatBIOS".to_string(), "⚙️ QuatBIOS");
            ui.selectable_value(&mut self.selected_app, "QuatFS".to_string(), "📁 QuatFS");
            ui.selectable_value(&mut self.selected_app, "QuatDSP".to_string(), "📻 QuatDSP");
            ui.selectable_value(&mut self.selected_app, "QuatLogic".to_string(), "🔲 QuatLogic");
            ui.selectable_value(&mut self.selected_app, "QuatVM".to_string(), "📦 QuatVM");
            ui.selectable_value(&mut self.selected_app, "CRT Screen".to_string(), "📺 CRT Ekran");
            ui.selectable_value(&mut self.selected_app, "QuatTape".to_string(), "🧠 QuatTape");
            ui.selectable_value(&mut self.selected_app, "QuatSriteStudio".to_string(), "🖼️ QuatSriteStudio");
            ui.selectable_value(&mut self.selected_app, "QuatTracker".to_string(), "🎥 QuatTracker");
            ui.selectable_value(&mut self.selected_app, "QuatALU".to_string(), "🧮 QuatALU");
            ui.selectable_value(&mut self.selected_app, "QuatBENCH".to_string(), "⚙️ QuatBENCH");
            ui.selectable_value(&mut self.selected_app, "QuatByteAudition".to_string(), "🎤 QuatByteAudition");
            ui.selectable_value(&mut self.selected_app, "GamesManager".to_string(), "🕹️ GamesManager");
        });

        // Centralni prikaz izabrane aplikacije
         egui::CentralPanel::default().show(ctx, |ui| {
    match self.selected_app.as_str() {
        "Task Manager" => self.task_manager.ui(ui,&mut self.kernel ),
        "Quat Terminal" => self.terminal.ui(ui, &mut self.kernel),
        "Assembler" => self.assembler.ui(ui, &mut self.kernel),
        "QuatDOS" => self.dos.ui(ui, &mut self.kernel, &mut self.net, &mut self.pic),
        "QuatBASIC" => self.basic.ui(ui, &mut self.audio, &mut self.vga),
        "QuatVGA" => self.vga.ui(ui, ctx),
        "QuatDisk" => self.disk.ui(ui, &mut self.audio),
        "QuatAudio" => self.audio.ui(ui, &mut self.kernel),
        "QuatGPU" => self.gpu.ui(ui, &mut self.kernel),
        "QuatNET" => self.net.ui(ui, &mut self.kernel),
        "QuatPIC" => self.pic.ui(ui, &mut self.kernel),
        "QuatBIOS" => self.bios.ui(ui, &mut self.kernel),
        "QuatFS" => self.fs.ui(ui, &mut self.kernel),
        "QuatDSP" => self.dsp.ui(ui, &mut self.kernel),
        "QuatLogic" => self.logic.ui(ui,&mut self.kernel ),
        "QuatVM" => self.vm.ui(ui, &mut self.kernel ),
        "CRT Screen" => self.crt_screen.ui(ui, &mut self.kernel),
        "QuatTape" => self.tape.ui(ui, &mut self.kernel),
        "QuatSpriteStudio" => self.spritestudio.ui(ui, &mut self.kernel),
        "QuatTracker" => self.tracker.ui(ui, &mut self.kernel),
        "QuatALU" => self.alu.ui(ui, &mut self.kernel),
        "QuatBENCH" => self.bench.ui(ui, &mut self.kernel),
        "QuatByteAudition" => self.byteaudition.ui(ui, &mut self.kernel),
        "GamesManager" => self.games.ui(ui),
        "RAM Inspector" => {
            ui.heading("RAM Memorija (256 lokacija 4^4 Stanja)");
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("ram_grid").striped(true).show(ui, |ui| {
                   for (i, byte) in self.kernel.ram.iter().take(256).enumerate() {
                      let q = byte.to_quats();
                      ui.label(format!("[{:02X}]:", i));
                      ui.label(format!("{},{},{},{}", q[0], q[1], q[2], q[3]));
                       if (i + 1) % 8 == 0 {
                        ui.end_row();
                       }
                    }
                });
            });
        }
                _ => { // Sistemski Monitor
                    ui.heading("Status Procesora ($4^4$ Arhitektura)");
                    ui.add_space(10.0);
                    let q_a = self.kernel.reg_a.to_quats();
                    let q_b = self.kernel.reg_b.to_quats();

                    ui.group(|ui| {
                        ui.label(format!("Registar A: {}", self.kernel.reg_a.0));
                        ui.label(format!("Kvatarni zapis: [{}] [{}] [{}] [{}]", q_a[0], q_a[1], q_a[2], q_a[3]));
                    });
                    ui.add_space(5.0);
                    ui.group(|ui| {
                        ui.label(format!("Registar B: {}", self.kernel.reg_b.0));
                        ui.label(format!("Kvatarni zapis: [{}] [{}] [{}] [{}]", q_b[0], q_b[1], q_b[2], q_b[3]));
                    });
                    ui.add_space(5.0);
                    ui.label(format!("Program Counter (PC): {:02X}", self.kernel.pc));

                    // Podrazumevani Monitor Registara
                    ui.add_space(5.0);
            ui.heading("🖥️ Monitor Registara i Stanja Procesora");
            ui.separator();
            ui.label(format!("Akumulator (A): {:?}", self.kernel.reg_a));
            ui.label(format!("Program Counter (PC): {:02X}", self.kernel.pc));
            ui.label(format!("Status: Standard Mode"));
                }
            }
        });
    }
}