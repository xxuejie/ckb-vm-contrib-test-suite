use binary::{AstCoreMachine, Write};
use ckb_vm_contrib::ast_interpreter::interpret;
use ckb_vm_contrib::ckb_vm::{
    decoder::build_decoder,
    instructions::execute,
    machine::{DefaultCoreMachine, DefaultMachineBuilder, VERSION1},
    memory::{sparse::SparseMemory, wxorx::WXorXMemory},
    Bytes, CoreMachine, Machine, Memory, Register, SupportMachine, ISA_A, ISA_B, ISA_IMC, ISA_MOP,
};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = std::fs::read(args[0].clone()).unwrap().into();
    let args: Vec<Bytes> = args.into_iter().map(|a| a.into()).collect();

    let core_machine = DefaultCoreMachine::<u64, WXorXMemory<SparseMemory<u64>>>::new(
        ISA_IMC | ISA_A | ISA_B | ISA_MOP,
        VERSION1,
        u64::max_value(),
    );
    let mut machine = DefaultMachineBuilder::new(core_machine).build();
    machine.load_program(&code, &args).expect("load program");

    let mut ast_machine = AstCoreMachine::default();

    let mut decoder = build_decoder::<u64>(machine.isa(), machine.version());
    machine.set_running(true);
    while machine.running() {
        let instruction = {
            let pc = machine.pc().to_u64();
            let memory = machine.memory_mut();
            decoder.decode(memory, pc).expect("decoding")
        };
        ast_machine.reset_registers();
        execute(instruction, &mut ast_machine).expect("execute");

        let new_pc = ast_machine
            .take_pc()
            .map(|pc| interpret(&pc, &mut machine).expect("interpret"));
        let mut memory_writes = vec![];
        let mut register_writes = vec![];
        let mut lr_write = None;
        let mut ecall = false;
        let mut ebreak = false;
        for write in ast_machine.take_writes() {
            match write {
                Write::Lr { value } => {
                    let value = interpret(&value, &mut machine).expect("interpret");
                    lr_write = Some(value);
                }
                Write::Memory {
                    address,
                    size,
                    value,
                } => {
                    let address = interpret(&address, &mut machine).expect("interpret");
                    let value = interpret(&value, &mut machine).expect("interpret");
                    memory_writes.push((address, size, value));
                }
                Write::Register { index, value } => {
                    let value = interpret(&value, &mut machine).expect("interpret");
                    register_writes.push((index, value));
                }
                Write::Ecall => ecall = true,
                Write::Ebreak => ebreak = true,
            }
        }

        // Commit all values
        if let Some(new_pc) = new_pc {
            machine.update_pc(new_pc);
            machine.commit_pc();
        }
        for (address, size, value) in memory_writes {
            match size {
                1 => machine
                    .memory_mut()
                    .store8(&address, &value)
                    .expect("store"),
                2 => machine
                    .memory_mut()
                    .store16(&address, &value)
                    .expect("store"),
                4 => machine
                    .memory_mut()
                    .store32(&address, &value)
                    .expect("store"),
                8 => machine
                    .memory_mut()
                    .store64(&address, &value)
                    .expect("store"),
                _ => panic!("Invalid store size: {}", size),
            }
        }
        if let Some(value) = lr_write {
            machine.memory_mut().set_lr(&value);
        }
        for (index, value) in register_writes {
            machine.set_register(index, value);
        }
        if ecall {
            machine.ecall().expect("ecall");
        }
        if ebreak {
            machine.ebreak().expect("ebreak");
        }
    }

    let result = machine.exit_code();
    if result != 0 {
        println!("Error result: {:?}", result);
        std::process::exit(i32::from(result));
    }
}
