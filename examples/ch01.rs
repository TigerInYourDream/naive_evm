use anyhow::Result;
use colored::Colorize;
use naive_evm::op_code::*;
use primitive_types::U256;
use sha3::Digest;
use std::num::NonZeroU32;
use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display, Error, Formatter},
    ops::{Deref, DerefMut},
    str::FromStr,
};

#[derive(Debug)]
struct Block {
    blockhash: U256,
    coinbase: U256,
    timestamp: u64,
    number: u64,
    prevrandao: U256,
    gaslimit: NonZeroU32,
    chainid: u8,
    selfbalance: u64,
    basefee: NonZeroU32,
}

#[allow(dead_code)]
#[derive(Debug)]
struct Account {
    balance: u64,
    nonce: u64,
    storage: HashMap<String, String>,
    code: Vec<u8>,
}

#[allow(dead_code)]
#[derive(Debug)]
struct Transaction {
    nonce: u64,
    gas_price: u64,
    gas_limit: u64,
    to: TransparentU256,
    value: u64,
    data: TransparentU256,
    caller: TransparentU256,
    origin: TransparentU256,
    this_addr: TransparentU256,
    v: u64,
    r: u64,
    s: u64,
}

#[allow(dead_code)]
#[derive(Debug)]
struct EVMLog {
    address: TransparentU256,
    data: TransparentU256,
    topics: Vec<TransparentU256>,
}

pub struct EVM {
    code: Vec<u8>,
    pc: usize,
    // 在堆栈中，每个元素长度为256位 最大深度1024
    stack: Vec<TransparentU256>,
    // memory
    memmory: Vec<u8>,
    storage: HashMap<U256, U256>,
    vaild_jump_dest: HashSet<usize>,
    current_block: Block,
    account_db: HashMap<TransparentU256, Account>,
    transaction: Transaction,
    log: Vec<EVMLog>,
    return_data: Vec<u8>,
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TransparentU256(pub U256);

impl Debug for TransparentU256 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:}", self.0)
    }
}

impl Default for TransparentU256 {
    fn default() -> Self {
        TransparentU256(U256::zero())
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
        // Hardcode block
        let current_block = Block {
            blockhash: U256::from_str(
                "0x7527123fc877fe753b3122dc592671b4902ebf2b325dd2c7224a43c0cbeee3ca",
            )
            .unwrap(),
            coinbase: U256::from_str("0x388C818CA8B9251b393131C08a736A67ccB19297").unwrap(),
            timestamp: 1625900000,
            number: 17871709,
            prevrandao: U256::from_str(
                "0xce124dee50136f3f93f19667fb4198c6b94eecbacfa300469e5280012757be94",
            )
            .unwrap(),
            gaslimit: NonZeroU32::new(30).unwrap(),
            chainid: 1,
            selfbalance: 100,
            basefee: NonZeroU32::new(30).unwrap(),
        };

        // HARD CODE ACCOUNT
        let mut account_db: HashMap<TransparentU256, Account> = HashMap::new();
        account_db.insert(
            // change it to TransparentU256
            U256::from("0x9bbfed6889322e016e0a02ee459d306fc19545d8").into(),
            Account {
                balance: 100,
                nonce: 1,
                storage: HashMap::new(),
                code: vec![0x60, 0x00, 0x60, 0x00],
            },
        );

        let transaction = Transaction {
            nonce: 0,
            gas_price: 1,
            gas_limit: 21000,
            to: U256::from("").into(),
            value: 0,
            data: U256::from("").into(),
            caller: U256::from("0x00").into(),
            origin: U256::from("0x00").into(),
            this_addr: U256::from("0x00").into(),
            v: 0,
            r: 0,
            s: 0,
        };

        Self {
            code: code.to_vec(),
            pc: 0,
            stack: Vec::with_capacity(256),
            memmory: Vec::new(),
            storage: HashMap::new(),
            vaild_jump_dest: HashSet::new(),
            current_block,
            account_db,
            transaction,
            log: Vec::new(),
            return_data: Vec::new(),
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
        // pop None
        self.stack.pop().unwrap_or(U256::zero().into())
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

    pub fn sstore(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let key = self.pop();
        let value = self.pop();
        self.storage.insert(*key, *value);
    }

    pub fn sload(&mut self) {
        if self.stack.is_empty() {
            panic!("stack underflow");
        }
        let key = self.pop();
        let default = &TransparentU256::default();
        let value = self.storage.get(&key).unwrap_or(default);
        self.stack.push((*value).into());
    }

    pub fn stop(&mut self) {
        let text = "stop evm".red().bold();
        println!("[evm]     --> {}", text)
    }

    pub fn find_valid_jump_destinations(&mut self) {
        let mut pc = 0;
        while pc < self.code.len() {
            let op = self.code[pc];
            if op == JUMPDEST {
                self.vaild_jump_dest.insert(pc);
            } else if op >= PUSH1 && op <= PUSH32 {
                // skip the immediate
                pc += (op - PUSH1 + 1) as usize;
            }
            pc += 1;
        }
    }

    // empty func
    pub fn jump_dest(&self) {}

    // JUMP指令用于无条件跳转到一个新的程序计数器位置。它从堆栈中弹出一个元素，将这个元素设定为新的程序计数器（pc）的值。操作码是0x56，gas消耗为8
    pub fn jump(&mut self) {
        if self.stack.is_empty() {
            panic!("stack underflow");
        }
        let dest = self.pop().as_usize();
        if dest >= self.code.len() {
            panic!("invalid jump destination");
        }
        println!("valid jump dest: {:?}", self.vaild_jump_dest);
        if !self.vaild_jump_dest.contains(&dest) {
            panic!("invalid jump destination");
        }
        self.pc = dest;
    }

    pub fn jumpi(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }

        let dest = self.pop().as_usize();
        let op = self.pop();
        if op.as_usize() != 0 {
            if !self.vaild_jump_dest.contains(&dest) {
                panic!("invalid jump destination");
            }
            self.pc = dest;
        }
    }

    pub fn pc(&mut self) {
        self.stack.push(U256::from(self.pc).into());
    }

    pub fn blockhash(&mut self) {
        if self.stack.is_empty() {
            panic!("stack underflow");
        }
        let block_number = self.pop().as_u64();
        if block_number == self.current_block.number {
            let block_hash = self.current_block.blockhash;
            self.stack.push(block_hash.into());
        } else {
            self.stack.push(0.into())
        }
    }

    pub fn coinbase(&mut self) {
        self.stack.push(self.current_block.coinbase.into());
    }

    pub fn timestamp(&mut self) {
        self.stack.push(self.current_block.timestamp.into());
    }

    pub fn number(&mut self) {
        self.stack.push(self.current_block.number.into());
    }

    pub fn prevrandao(&mut self) {
        self.stack.push(self.current_block.prevrandao.into());
    }

    pub fn gaslimit(&mut self) {
        self.stack
            .push(TransparentU256(self.current_block.gaslimit.get().into()));
    }

    pub fn chainid(&mut self) {
        self.stack
            .push(TransparentU256(self.current_block.chainid.into()));
    }

    pub fn selfbalance(&mut self) {
        self.stack.push(self.current_block.selfbalance.into());
    }

    pub fn basefee(&mut self) {
        self.stack
            .push(TransparentU256(self.current_block.basefee.get().into()));
    }

    pub fn dup(&mut self, postion: usize) {
        if let Some(value) = self.stack.get(self.stack.len() - postion) {
            self.stack.push(value.clone());
        } else {
            panic!("stack underflow");
        }
    }

    pub fn swap(&mut self, postion: usize) {
        if self.stack.len() < postion + 1 {
            panic!("stack underflow");
        }
        let idx1 = self.stack.len() - 1;
        let idx2 = self.stack.len() - 1 - postion;
        self.stack.swap(idx1, idx2);
    }

    pub fn sha3(&mut self) {
        if self.stack.is_empty() {
            panic!("stack underflow");
        }
        let offset = self.pop().as_u64() as usize;
        let size = self.pop().as_u64() as usize;
        let data = &self.memmory[offset..offset + size];
        let mut hasher = sha3::Keccak256::new();
        hasher.update(&data);
        let result = hasher.finalize();
        self.stack.push(U256::from(&result[..]).into());
    }

    pub fn balance(&mut self) {
        if self.stack.is_empty() {
            panic!("stack underflow");
        }
        let address = self.pop();
        let account = self.account_db.get(&address).unwrap();
        self.stack.push(account.balance.into());
    }

    pub fn extcodesize(&mut self) {
        if self.stack.is_empty() {
            panic!("stack underflow");
        }
        let address = self.pop();
        let account = self.account_db.get(&address).unwrap();
        self.stack.push((account.code.len() as u64).into());
    }

    pub fn extcodecopy(&mut self) {
        if self.stack.len() < 4 {
            panic!("stack underflow");
        }
        let addr = self.pop();
        let mem_offset = self.pop().as_u64() as usize;
        let code_offset = self.pop().as_u64() as usize;
        let length = self.pop().as_u64() as usize;

        let code =
            &self.account_db.get(&addr).unwrap().code.clone()[code_offset..code_offset + length];
        while self.memmory.len() < mem_offset + length {
            self.memmory.push(0.into());
        }
        self.memmory[mem_offset..mem_offset + length].copy_from_slice(&code[..]);
    }

    pub fn extcodehash(&mut self) {
        if self.stack.is_empty() {
            panic!("stack underflow");
        }
        let address = self.pop();
        let account = self.account_db.get(&address).unwrap();
        let mut hasher = sha3::Keccak256::new();
        hasher.update(&account.code);
        let result = hasher.finalize();
        self.stack.push(U256::from(&result[..]).into());
    }

    pub fn address(&mut self) {
        self.stack.push(self.transaction.this_addr.clone().into());
    }

    pub fn origin(&mut self) {
        self.stack.push(self.transaction.origin.clone().into());
    }

    pub fn caller(&mut self) {
        self.stack.push(self.transaction.caller.clone().into());
    }

    pub fn callvalue(&mut self) {
        self.stack.push(self.transaction.value.into());
    }

    pub fn log(&mut self, num_topics: usize) {
        if self.stack.len() < 2 + num_topics {
            panic!("stack underflow");
        }
        let mem_offset = self.pop().as_u32() as usize;
        let length = self.pop().as_u32() as usize;
        let num_topics = self.pop().as_u64() as usize;
        let mut topics = Vec::with_capacity(num_topics);
        for _ in 0..num_topics {
            topics.push(self.pop());
        }
        let data = &self.memmory[mem_offset..mem_offset + length];
        self.log.push(EVMLog {
            address: self.transaction.this_addr.clone(),
            data: U256::from(data).into(),
            topics,
        });
    }

    pub fn return_op(&mut self) {
        if self.stack.len() < 2 {
            panic!("stack underflow");
        }
        let mem_offset = self.pop().as_u32() as usize;
        let length = self.pop().as_u32() as usize;
        if self.memmory.len() < mem_offset + length {
            self.memmory.resize(mem_offset + length, 0.into());
        }
        self.return_data = self.memmory[mem_offset..mem_offset + length].to_vec();
    }

    pub fn return_data_size(&mut self) {
        self.stack.push((self.return_data.len() as u64).into());
    }

    pub fn return_data_copy(&mut self) {
        if self.stack.len() < 3 {
            panic!("stack underflow");
        }
        let mem_offset = self.pop().as_u32() as usize;
        let length = self.pop().as_u32() as usize;
        let data = &self.return_data;
        if self.memmory.len() < mem_offset + length {
            self.memmory.resize(mem_offset + length, 0.into());
        }
        self.memmory[mem_offset..mem_offset + length].copy_from_slice(&data[..]);
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
                SSTORE => {
                    self.sstore();
                }
                SLOAD => {
                    self.sload();
                }
                STOP => {
                    self.stop();
                    break;
                }
                JUMP => {
                    self.jump();
                }
                JUMPDEST => {
                    self.jump_dest();
                }
                JUMPI => {
                    self.jumpi();
                }
                BLOCKHASH => {
                    self.blockhash();
                }
                COINBASE => {
                    self.coinbase();
                }
                TIMESTAMP => {
                    self.timestamp();
                }
                NUMBER => self.number(),
                PREVRANDAO => {
                    self.prevrandao();
                }
                GASLIMIT => {
                    self.gaslimit();
                }
                CHAINID => {
                    self.chainid();
                }
                SELFBALANCE => {
                    self.selfbalance();
                }
                BASEFEE => {
                    self.basefee();
                }
                i if DUP1 <= i && i <= DUP16 => {
                    let position = i - DUP1 + 1;
                    self.dup(position as usize);
                }
                i if SWAP1 <= i && i <= SWAP16 => {
                    let position = op - SWAP1 + 1;
                    self.swap(position as usize)
                }
                SHA3 => {
                    self.sha3();
                }
                BALANCE => {
                    self.balance();
                }
                EXTCODESIZE => {
                    self.extcodesize();
                }
                EXTCODECOPY => {
                    self.extcodecopy();
                }
                EXTCODEHASH => {
                    self.extcodehash();
                }
                ADDRESS => {
                    self.address();
                }
                ORIGIN => {
                    self.origin();
                }
                CALLER => {
                    self.caller();
                }
                CALLVALUE => {
                    self.callvalue();
                }
                LOG0 => {
                    self.log(0);
                }
                LOG1 => {
                    self.log(1);
                }
                LOG3 => {
                    self.log(2);
                }
                LOG4 => {
                    self.log(3);
                }
                RETURN => {
                    self.return_op();
                }
                RETURNDATASIZE => {
                    self.return_data_size();
                }
                _ => unimplemented!(),
            }
        }
    }
}

pub fn main() {
    let appname = r#"

    ███╗   ██╗ █████╗ ██╗██╗   ██╗███████╗    ███████╗██╗   ██╗███╗   ███╗
    ████╗  ██║██╔══██╗██║██║   ██║██╔════╝    ██╔════╝██║   ██║████╗ ████║
    ██╔██╗ ██║███████║██║██║   ██║█████╗      █████╗  ██║   ██║██╔████╔██║
    ██║╚██╗██║██╔══██║██║╚██╗ ██╔╝██╔══╝      ██╔══╝  ╚██╗ ██╔╝██║╚██╔╝██║
    ██║ ╚████║██║  ██║██║ ╚████╔╝ ███████╗    ███████╗ ╚████╔╝ ██║ ╚═╝ ██║
    ╚═╝  ╚═══╝╚═╝  ╚═╝╚═╝  ╚═══╝  ╚══════╝    ╚══════╝  ╚═══╝  ╚═╝     ╚═╝
                                                                          
    
    "#;
    println!("{}", appname.green().bold());

    let code = b"\x3D";
    let mut evm = EVM::init(code);
    // check valid jumo dest
    evm.find_valid_jump_destinations();
    evm.return_data.append(&mut vec![0xaa, 0xaa]);
    evm.run();
    println!("[memoryhex]--> {:?}", hex::encode(&evm.memmory));
    println!("[memory]   --> {:?}", &evm.memmory[..]);
    println!("[stack]    --> {:?}", &evm.stack);
    println!("[storage]  --> {:?}", &evm.storage);
    println!("[log]      --> {:?}", &evm.log);
    println!("[return]   --> {:?}", hex::encode(&evm.return_data));
}
