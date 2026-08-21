use crate::QuatVM;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuatByte(pub u8);

impl QuatByte {
    pub const ZERO: QuatByte = QuatByte(0);

    /// Konstruktor sa 1 argumentom (VRAĆEN radi asamblera i starog koda)
    pub fn new(val: u8) -> Self {
        QuatByte(val)
    }

    /// Konstruktor sa 4 pojedinačna kvata (q3, q2, q1, q0)
    pub fn from_quats(q3: u8, q2: u8, q1: u8, q0: u8) -> Self {
        QuatByte(((q3 & 3) << 6) | ((q2 & 3) << 4) | ((q1 & 3) << 2) | (q0 & 3))
    }

    pub fn from_u8(val: u8) -> Self {
        QuatByte(val)
    }

    pub fn to_u8(&self) -> u8 {
        self.0
    }

    /// Pretvara u 4 cifre u bazi 4 [Q3, Q2, Q1, Q0]
    pub fn to_quats(&self) -> [u8; 4] {
        [
            (self.0 >> 6) & 3,
            (self.0 >> 4) & 3,
            (self.0 >> 2) & 3,
            self.0 & 3,
        ]
    }

    /// IZVORNO ČETVERNO SABIRANJE U BAZI 4 (Quaternary ALU)
    pub fn add(&self, rhs: &QuatByte) -> (QuatByte, bool) {
        let a = self.to_quats();
        let b = rhs.to_quats();
        let mut res = [0u8; 4];
        let mut carry = 0u8;

        for i in (0..4).rev() {
            let sum = a[i] + b[i] + carry;
            res[i] = sum % 4;
            carry = sum / 4;
        }

        (QuatByte::from_quats(res[0], res[1], res[2], res[3]), carry > 0)
    }

    /// IZVORNO ČETVERNO ODUZIMANJE U BAZI 4 (Quaternary Borrow ALU)
    pub fn sub(&self, rhs: &QuatByte) -> (QuatByte, bool) {
        let a = self.to_quats();
        let b = rhs.to_quats();
        let mut res = [0u8; 4];
        let mut borrow = 0i16;

        for i in (0..4).rev() {
            let mut diff = a[i] as i16 - b[i] as i16 - borrow;
            if diff < 0 {
                diff += 4;
                borrow = 1;
            } else {
                borrow = 0;
            }
            res[i] = diff as u8;
        }

        (QuatByte::from_quats(res[0], res[1], res[2], res[3]), borrow > 0)
    }

    pub fn to_quat_str(&self) -> String {
        let q = self.to_quats();
        format!("Q{}{}{}{}", q[0], q[1], q[2], q[3])
    }
}

//jebo me dan kada sam pomislio da sam zaokruzio 4 stanja (0 1 2 3) sa 4 kvatnim sistemom
// i pitas ai jer te mrzi da pises sam i on te psihicki ujebe

// DRIVER & KERNEL SIMULATOR
pub struct QuatKernel {
    pub ram: [QuatByte; 256],
    pub vram: [QuatByte; 256],
    pub reg_a: QuatByte,
    pub reg_b: QuatByte,
    pub pc: u8,
    pub is_running: bool,
    pub logs: Vec<String>,
}

//Ne pitaj nista
//detaljno proveri da li su 4 stanja u 4 kvatnom sistemu
//ako nije srecno

impl QuatKernel {
    pub fn new() -> Self {
        let mut kernel = Self {
            ram: [QuatByte::ZERO; 256],
            vram: [QuatByte::ZERO; 256],
            reg_a: QuatByte::ZERO,
            reg_b: QuatByte::ZERO,
            pc: 0,
            is_running: false,
            logs: vec!["QuatKernel 4^4 Inicijalizovan u Nativnom Četvernom Modu.".to_string()],
        };

        kernel.load_sample_program();
        kernel
    }

    pub fn load_sample_program(&mut self) {
        self.ram[0] = QuatByte::from_quats(0, 0, 2, 2); // Q0022
        self.ram[1] = QuatByte::from_quats(0, 0, 1, 1); // Q0011
        self.log("Sample program uspešno učitan u RAM.");
    }

    pub fn step(&mut self) {
        if self.pc >= 250 {
            self.is_running = false;
            return;
        }
//Ni sam ne znam da li pisem 4 ili 2 stanja

        let data = self.ram[self.pc as usize];

        // IZVRŠAVANJE: Izvorni Quaternary ALU (A = A + Data)
        let (new_reg_a, carry) = self.reg_a.add(&data);
        self.reg_a = new_reg_a;

        let q = self.reg_a.to_quats();
        self.log(format!(
            "PC: {:02X} | RegA = {} (Quats: [{}, {}, {}, {}]) | Carry: {}",
            self.pc,
            self.reg_a.to_quat_str(),
            q[0], q[1], q[2], q[3],
            carry
        ));

        self.pc += 1;
    }

    pub fn log(&mut self, msg: impl Into<String>) {
        self.logs.push(msg.into());
        if self.logs.len() > 50 {
            self.logs.remove(0);
        }
    }
}