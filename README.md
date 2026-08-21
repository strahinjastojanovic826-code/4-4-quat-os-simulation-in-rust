# QuatOS — 4-State Quaternary Operating System

A custom simulated operating system kernel and virtual architecture built in **Rust**, exploring **quaternary (4-state) logic** instead of traditional binary systems.

> **Core Logic:** 4 quats = 4^4 = 256 states = 1 Byte.

---

## 🚀 Overview

**QuatOS** is an experimental operating system simulation that replaces the binary paradigm (0 and 1) with a base-4 quaternary logic system (0, 1, 2, 3). It simulates a complete virtual machine architecture from the ground up — including a custom ALU, graphics rendering pipeline, storage subsystems, and user environment.

---

## 🏛️ System Architecture

The project is structured into modular quat-native layers:

* **Core Kernel & Execution:** Virtual Machine (`quat_vm`), ALU (`quat_alu`), BIOS (`quat_bios`), custom assembler (`assembler`), and 4-state byte handler (`quat_byte`).
* **Display & Graphics:** Custom VGA engine (`quat_vga`), CRT screen emulator (`crt_screen`), GPU dispatcher (`quat_gpu`), and sprite renderer (`quat_sprite`).
* **Storage & File Systems:** Quaternary filesystem (`quat_fs`), virtual disk I/O (`quat_disk`), DOS layer (`quat_dos`), and magnetic tape emulator (`quat_tape`).
* **Audio & Hardware:** Audio engine (`quat_audio`), DSP pipeline (`quat_dsp`), Programmable Interrupt Controller (`quat_pic`), and music tracker (`quat_tracker`).
* **User Space & Shell:** Interactive shell (`terminal`), process manager (`task_manager`), BASIC interpreter (`quat_basic`), benchmark suite (`quat_bench`), and built-in games (`games.rs`).

---

## 🛠️ Quick Start

### Prerequisites
* [Rust & Cargo](https://www.rust-lang.org/) (latest stable)

### Running the OS

```bash
# Clone the repository
git clone [https://github.com/YOUR_USERNAME/quat-os.git](https://github.com/YOUR_USERNAME/quat-os.git)

# Navigate to directory
cd quat-os

# Run the simulation
cargo run
```

<img width="1920" height="1080" alt="Screenshot 2026-08-21 211126" src="https://github.com/user-attachments/assets/cfa71f7e-513e-4d04-b8db-7e48b0975304" />

<img width="1920" height="1080" alt="Screenshot 2026-08-21 211102" src="https://github.com/user-attachments/assets/eab87f3f-40be-4ce8-9f18-ed28e59da457" />

<img width="1920" height="1080" alt="Screenshot 2026-08-21 211040" src="https://github.com/user-attachments/assets/8f3556a5-5f95-41bf-ab8b-482f63056364" />









