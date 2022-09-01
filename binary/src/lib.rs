use ckb_vm::{
    instructions::ast::Value, machine::VERSION1, Bytes, CoreMachine, Error, Machine, Memory, ISA_B,
    ISA_IMC, ISA_MOP,
};
use ckb_vm_contrib::ast_interpreter::PC_INDEX;
use std::mem;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Write {
    Memory {
        address: Value,
        size: u8,
        value: Value,
    },
    Register {
        index: usize,
        value: Value,
    },
    Ecall,
    Ebreak,
}

pub struct AstCoreMachine {
    registers: [Value; 32],
    pc: Value,
    next_pc: Value,

    writes: Vec<Write>,
    version: u32,
    isa: u8,
}

impl Default for AstCoreMachine {
    fn default() -> Self {
        Self {
            registers: init_registers(),
            pc: Value::Register(PC_INDEX),
            next_pc: Value::Register(PC_INDEX),
            writes: vec![],
            version: VERSION1,
            isa: ISA_IMC | ISA_MOP | ISA_B,
        }
    }
}

impl AstCoreMachine {
    pub fn new(version: u32, isa: u8) -> Self {
        let mut m = AstCoreMachine::default();
        m.version = version;
        m.isa = isa;
        m
    }

    pub fn reset_registers(&mut self) {
        self.registers = init_registers();
        self.pc = Value::Register(PC_INDEX);
        self.next_pc = Value::Register(PC_INDEX);
    }

    pub fn take_writes(&mut self) -> Vec<Write> {
        mem::replace(&mut self.writes, Vec::new())
    }

    pub fn take_pc(&mut self) -> Option<Value> {
        let pc = mem::replace(&mut self.pc, Value::Register(PC_INDEX));
        match &pc {
            Value::Register(i) if *i == PC_INDEX => None,
            _ => Some(pc),
        }
    }
}

impl Machine for AstCoreMachine {
    fn ecall(&mut self) -> Result<(), Error> {
        self.writes.push(Write::Ecall);
        Ok(())
    }

    fn ebreak(&mut self) -> Result<(), Error> {
        self.writes.push(Write::Ebreak);
        Ok(())
    }
}

impl CoreMachine for AstCoreMachine {
    type REG = Value;
    type MEM = Self;

    fn pc(&self) -> &Value {
        &self.pc
    }

    fn update_pc(&mut self, pc: Self::REG) {
        self.next_pc = pc;
    }

    fn commit_pc(&mut self) {
        self.pc = self.next_pc.clone();
    }

    fn memory(&self) -> &Self {
        &self
    }

    fn memory_mut(&mut self) -> &mut Self {
        self
    }

    fn registers(&self) -> &[Value] {
        &self.registers
    }

    fn set_register(&mut self, index: usize, value: Value) {
        self.registers[index] = value.clone();
        self.writes.push(Write::Register { index, value });
    }

    fn version(&self) -> u32 {
        self.version
    }

    fn isa(&self) -> u8 {
        self.isa
    }
}

impl Memory for AstCoreMachine {
    type REG = Value;

    fn init_pages(
        &mut self,
        _addr: u64,
        _size: u64,
        _flags: u8,
        _source: Option<Bytes>,
        _offset_from_addr: u64,
    ) -> Result<(), Error> {
        unreachable!()
    }

    fn fetch_flag(&mut self, _page: u64) -> Result<u8, Error> {
        unreachable!()
    }

    fn set_flag(&mut self, _page: u64, _flag: u8) -> Result<(), Error> {
        unreachable!()
    }

    fn clear_flag(&mut self, _page: u64, _flag: u8) -> Result<(), Error> {
        unreachable!()
    }

    fn store_byte(&mut self, _addr: u64, _size: u64, _value: u8) -> Result<(), Error> {
        unreachable!()
    }

    fn store_bytes(&mut self, _addr: u64, _value: &[u8]) -> Result<(), Error> {
        unreachable!()
    }

    fn execute_load16(&mut self, _addr: u64) -> Result<u16, Error> {
        unreachable!()
    }

    fn execute_load32(&mut self, _addr: u64) -> Result<u32, Error> {
        unreachable!()
    }

    fn load8(&mut self, addr: &Value) -> Result<Value, Error> {
        Ok(Value::Load(Rc::new(addr.clone()), 1))
    }

    fn load16(&mut self, addr: &Value) -> Result<Value, Error> {
        Ok(Value::Load(Rc::new(addr.clone()), 2))
    }

    fn load32(&mut self, addr: &Value) -> Result<Value, Error> {
        Ok(Value::Load(Rc::new(addr.clone()), 4))
    }

    fn load64(&mut self, addr: &Value) -> Result<Value, Error> {
        Ok(Value::Load(Rc::new(addr.clone()), 8))
    }

    fn store8(&mut self, addr: &Value, value: &Value) -> Result<(), Error> {
        self.writes.push(Write::Memory {
            address: addr.clone(),
            size: 1,
            value: value.clone(),
        });
        Ok(())
    }

    fn store16(&mut self, addr: &Value, value: &Value) -> Result<(), Error> {
        self.writes.push(Write::Memory {
            address: addr.clone(),
            size: 2,
            value: value.clone(),
        });
        Ok(())
    }

    fn store32(&mut self, addr: &Value, value: &Value) -> Result<(), Error> {
        self.writes.push(Write::Memory {
            address: addr.clone(),
            size: 4,
            value: value.clone(),
        });
        Ok(())
    }

    fn store64(&mut self, addr: &Value, value: &Value) -> Result<(), Error> {
        self.writes.push(Write::Memory {
            address: addr.clone(),
            size: 8,
            value: value.clone(),
        });
        Ok(())
    }
}

pub fn init_registers() -> [Value; 32] {
    [
        Value::Imm(0),
        Value::Register(1),
        Value::Register(2),
        Value::Register(3),
        Value::Register(4),
        Value::Register(5),
        Value::Register(6),
        Value::Register(7),
        Value::Register(8),
        Value::Register(9),
        Value::Register(10),
        Value::Register(11),
        Value::Register(12),
        Value::Register(13),
        Value::Register(14),
        Value::Register(15),
        Value::Register(16),
        Value::Register(17),
        Value::Register(18),
        Value::Register(19),
        Value::Register(20),
        Value::Register(21),
        Value::Register(22),
        Value::Register(23),
        Value::Register(24),
        Value::Register(25),
        Value::Register(26),
        Value::Register(27),
        Value::Register(28),
        Value::Register(29),
        Value::Register(30),
        Value::Register(31),
    ]
}
