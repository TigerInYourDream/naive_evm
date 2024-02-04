use anyhow::Result;
use naive_evm::op_code::*;
use primitive_types::U256;
use std::{
    fmt::{Debug, Display, Error, Formatter},
    ops::{Deref, DerefMut},
};

pub struct EVM {
    code: Vec<u8>,
    pc: usize,
    // 在堆栈中，每个元素长度为256位 最大深度1024
    stack: Vec<TransparentU256>,
    // memory
    memmory: Vec<u8>,
}
#[derive(Clone, PartialEq, Eq)]
pub struct TransparentU256(pub U256);

impl Debug for TransparentU256 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:}", self.0)
    }
}

impl Deref for TransparentU256 {
    type Target = U256;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TransparentU256 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<u64> for TransparentU256 {
    fn from(value: u64) -> Self {
        TransparentU256(U256::from(value))
    }
}

impl From<U256> for TransparentU256 {
    fn from(value: U256) -> Self {
        TransparentU256(value)
    }
}

impl Display for EVM {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Evm stack: {:?} memmory: {:?}", self.stack, self.memmory)
    }
}

impl EVM {
    pub fn init(code: &[u8]) -> Self {
        Self {
            code: code.to_vec(),
            pc: 0,
            stack: Vec::with_capacity(256),
            memmory: Vec::new(),
        }
    }

    pub fn next_instruction(&mut self) -> u8 {
        let instruction = self.code[self.pc as usize];
        self.pc += 1;
        instruction
    }

    pub fn push(&mut self, size: usize) {
        let data = &self.code[self.pc..self.pc + size];
        let value = U256::from(data);
        self.stack.push(value.into());
        self.pc += size;
    }

    pub fn pop(&mut self) -> TransparentU256 {
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
        let res = a.checked_add(*b).expect("add overflow");
        self.stack.push(res.into());
    }

    pub fn mul(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        let res = a.checked_mul(*b).expect("mul overflow");
        self.stack.push(res.into());
    }

    pub fn sub(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        let res = b.checked_sub(*a).expect("sub overflow");
        self.stack.push(res.into());
    }

    pub fn div(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        let res = b.checked_div(*a).expect("div overflow");
        self.stack.push(res.into());
    }

    pub fn sdiv(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        let res = b.checked_div(*a).expect("sdiv overflow");
        self.stack.push(res.into());
    }

    pub fn r#mod(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        let res = b.checked_rem(*a).expect("mod overflow");
        self.stack.push(res.into());
    }

    pub fn exp(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        let res = b.checked_pow(*a).expect("exp overflow");
        self.stack.push(res.into());
    }

    pub fn lt(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        let res = if *b < *a { 1 } else { 0 };
        self.stack.push(res.into());
    }

    pub fn eq(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        let res = if *b == *a { 1 } else { 0 };
        self.stack.push(res.into());
    }

    pub fn gt(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        let res = if *b > *a { 1 } else { 0 };
        self.stack.push(res.into());
    }

    pub fn iszero(&mut self) {
        if self.stack.is_empty() {
            panic!("stack underflow");
        }
        let a = self.pop();
        let res = if a.is_zero() { 1 } else { 0 };
        self.stack.push(res.into());
    }

    pub fn and_op(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        self.stack.push(((*a) & (*b)).into());
    }

    pub fn or(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        self.stack.push((*a | *b).into());
    }

    pub fn xor(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        self.stack.push((*a ^ *b).into());
    }

    pub fn not(&mut self) {
        if self.stack.is_empty() {
            panic!("stack underflow");
        }
        let a = self.pop();
        self.stack.push((!(*a)).into());
    }

    pub fn shl(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        self.stack.push((*b << *a).into());
    }

    pub fn shr(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        self.stack.push((*b >> *a).into());
    }

    pub fn byte(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let a = self.pop();
        let b = self.pop();
        self.stack
            .push(((*b >> (*a * 8)) & U256::from(0xff)).into());
    }

    pub fn mstore(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let offset = self.pop().as_u64() as usize;
        let value = self.pop();
        // 填充 offsite + 32
        while self.memmory.len() < offset + 32 {
            self.memmory.push(0.into());
        }
        // 补充[u8;32]
        let mut res: [u8; 32] = [0; 32];
        value.to_big_endian(&mut res);
        self.memmory[offset..offset + 32].copy_from_slice(&res);
    }

    pub fn mstore8(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let offset = self.pop().as_u64() as usize;
        // only need low 8 bits
        let value = self.pop();
        while self.memmory.len() < offset + 32 {
            self.memmory.push(0.into());
        }
        let mut res: [u8; 32] = [0; 32];
        value.to_big_endian(&mut res);
        self.memmory[offset..offset + 32].copy_from_slice(&res[24..32]);
    }

    pub fn mload(&mut self) {
        if self.stack.is_empty() {
            panic!("stack underflow");
        }
        let offset = self.pop().as_u32() as usize;
        while self.memmory.len() < 32 + offset {
            self.memmory.push(0.into());
        }
        let value = &self.memmory[offset..offset + 32];
        self.stack.push(U256::from(value).into());
    }

    pub fn msize(&mut self) {
        let size = self.memmory.len() as u64;
        self.stack.push(size.into());
    }

    pub fn run(&mut self) {
        while self.pc < self.code.len() {
            let op = self.next_instruction();
            match op {
                i if PUSH1 <= i && i <= PUSH32 => {
                    let size = op - PUSH1 + 1;
                    self.push(size as usize);
                }
                PUSH0 => self.stack.push(0.into()),
                // pop()
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
                MSTORE => {
                    self.mstore();
                }
                MSTORE8 => {
                    self.mstore8();
                }
                MLOAD => {
                    self.mload();
                }
                MSIZE => {
                    self.msize();
                }
                _ => unimplemented!(),
            }
        }
    }
}

pub fn main() {
    let code = b"\x60\x02\x60\x20\x52\x60\x20\x51";
    let mut evm = EVM::init(code);
    evm.run();
    println!("[memory] --> {:?}", &evm.memmory[0x20..0x40]);
    println!("[stack]  --> {:?}", &evm.stack);
}
