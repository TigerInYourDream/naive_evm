use anyhow::Result;
use std::fmt::{Display, Error, Formatter};

const PUSH0: u8 = 0x5F;
const PUSH1: u8 = 0x60;
const PUSH32: u8 = 0x7F;
const POP: u8 = 0x50;
const ADD: u8 = 0x01;
const MUL: u8 = 0x02;
const SUB: u8 = 0x03;
const DIV: u8 = 0x04;
const SDIV: u8 = 0x05;
const MOD: u8 = 0x06;
const EXP: u8 = 0x0A;
const LT: u8 = 0x10;
const GT: u8 = 0x11;
const EQ: u8 = 0x14;
const ISZERO: u8 = 0x15;
const AND: u8 = 0x16;
const OR: u8 = 0x17;
const XOR: u8 = 0x18;
const NOT: u8 = 0x19;
const SHL: u8 = 0x1B;
const SHR: u8 = 0x1C;
const BYTE: u8 = 0x1A;

pub struct EVM {
    code: Vec<u8>,
    pc: usize,
    // 在堆栈中，每个元素长度为256位 最大深度1024
    stack: Vec<i32>,
}

impl Display for EVM {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "evm stack {:?}", self.stack)
    }
}

impl EVM {
    pub fn init(code: &[u8]) -> Self {
        Self {
            code: code.to_vec(),
            pc: 0,
            stack: Vec::with_capacity(256),
        }
    }

    pub fn next_instruction(&mut self) -> u8 {
        let instruction = self.code[self.pc as usize];
        self.pc += 1;
        instruction
    }

    pub fn push(&mut self, size: usize) {
        let data = &self.code[self.pc..self.pc + size];
        // 简单填充[u8] 为u32
        let mut buffer = [0u8; 4];
        for (i, byte) in data.iter().enumerate() {
            buffer[4 - size + i] = *byte;
        }
        let value = i32::from_be_bytes(buffer);
        self.stack.push(value);
        self.pc += size;
    }

    pub fn pop(&mut self) -> i32 {
        if self.stack.is_empty() {
            panic!("stack underflow");
        }
        self.stack.pop().unwrap()
    }

    pub fn add(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        // 防止溢出 256位
        let res = a.checked_add(b).expect("add overflow");
        self.stack.push(res);
    }

    pub fn mul(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        // 防止溢出 256位
        let res = a.checked_mul(b).expect("mul overflow");
        self.stack.push(res);
    }

    pub fn sub(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        // 防止溢出 256位
        let res = b.checked_sub(a).expect("sub overflow");
        self.stack.push(res);
    }

    pub fn div(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        let res = b.checked_div(a).expect("div overflow");
        self.stack.push(res.abs());
    }

    pub fn sdiv(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        let res = b.checked_div(a).expect("sdiv overflow");
        self.stack.push(res);
    }

    pub fn r#mod(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        let res = b.checked_rem(a).expect("mod overflow");
        self.stack.push(res);
    }

    pub fn exp(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        let res = b.checked_pow(a as u32).expect("exp overflow");
        self.stack.push(res);
    }

    pub fn lt(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        let res = if b < a { 1 } else { 0 };
        self.stack.push(res);
    }

    pub fn eq(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        let res = if b == a { 1 } else { 0 };
        self.stack.push(res);
    }

    pub fn gt(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        let res = if b > a { 1 } else { 0 };
        self.stack.push(res);
    }

    pub fn iszero(&mut self) {
        if self.stack.is_empty() {
            panic!("stack underflow");
        }
        let a = self.pop();
        let res = if a == 0 { 1 } else { 0 };
        self.stack.push(res);
    }

    pub fn and_op(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        self.stack.push(a & b);
    }

    pub fn or(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        self.stack.push(a | b);
    }

    pub fn xor(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        self.stack.push(a ^ b);
    }

    pub fn not(&mut self) {
        if self.stack.is_empty() {
            panic!("stack underflow");
        }
        let a = self.pop();
        self.stack.push(!a);
    }

    pub fn shl(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        self.stack.push(b << a);
    }

    pub fn shr(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        self.stack.push(b >> a);
    }

    pub fn byte(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        self.stack.push((b >> (a * 8)) & 0xff);
    }

    pub fn run(&mut self) {
        while self.pc < self.code.len() {
            let op = self.next_instruction();
            match op {
                // from PUSH1 to PUSH32
                i if PUSH1 <= i && i <= PUSH32 => {
                    println!("op : {:0x?}", i);
                    let size = op - PUSH1 + 1;
                    self.push(size as usize);
                }
                // just for PUSH0 for save gas
                PUSH0 => self.stack.push(0),
                // pop
                POP => {
                    self.pop();
                }
                ADD => {
                    self.add();
                }
                MUL => {
                    self.mul();
                }
                SUB => {
                    self.sub();
                }
                DIV => {
                    self.div();
                }
                SDIV => {
                    self.sdiv();
                }
                MOD => {
                    self.r#mod();
                }
                EXP => {
                    self.exp();
                }
                LT => {
                    self.lt();
                }
                GT => {
                    self.gt();
                }
                EQ => {
                    self.eq();
                }
                ISZERO => {
                    self.iszero();
                }
                AND => {
                    self.and_op();
                }
                OR => {
                    self.or();
                }
                XOR => {
                    self.xor();
                }
                NOT => {
                    self.not();
                }
                SHL => {
                    self.shl();
                }
                SHR => {
                    self.shr();
                }
                BYTE => {
                    self.byte();
                }
                _ => unimplemented!(),
            }
        }
    }
}

pub fn main() {
    let code = b"\x60\x10\x60\x03\x1C";
    let mut evm = EVM::init(code);
    evm.run();
    println!("{:}", evm);
}
