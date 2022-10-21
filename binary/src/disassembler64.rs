use ckb_vm_contrib::ckb_vm::{
    decoder::build_decoder,
    instructions::{execute, instruction_length, set_instruction_length_n},
    machine::{DefaultCoreMachine, DefaultMachineBuilder, VERSION1},
    memory::{sparse::SparseMemory, wxorx::WXorXMemory},
    Bytes, CoreMachine, Register, SupportMachine, ISA_B, ISA_IMC,
};
use ckb_vm_contrib::{assembler::parse, printer::InstructionPrinter};
use std::convert::TryInto;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = std::fs::read(args[0].clone()).unwrap().into();
    let args: Vec<Bytes> = args.into_iter().map(|a| a.into()).collect();

    let mut machine1 = {
        let core_machine = DefaultCoreMachine::<u64, WXorXMemory<SparseMemory<u64>>>::new(
            ISA_IMC | ISA_B,
            VERSION1,
            u64::max_value(),
        );
        let mut machine = DefaultMachineBuilder::new(core_machine).build();
        machine.load_program(&code, &args).expect("load program");
        machine
    };
    let mut machine2 = {
        let core_machine = DefaultCoreMachine::<u64, WXorXMemory<SparseMemory<u64>>>::new(
            ISA_IMC | ISA_B,
            VERSION1,
            u64::max_value(),
        );
        let mut machine = DefaultMachineBuilder::new(core_machine).build();
        machine.load_program(&code, &args).expect("load program");
        machine
    };

    let mut decoder = build_decoder::<u64>(machine1.isa(), machine1.version());
    machine1.set_running(true);
    while machine1.running() {
        let pc = machine1.pc().to_u64();
        let instruction = {
            let memory = machine1.memory_mut();
            decoder.decode(memory, pc).expect("decoding")
        };
        let text = format!(
            "{}",
            InstructionPrinter::new(
                instruction
                    .try_into()
                    .expect("convert to tagged instruction"),
            )
        );
        let parsed_insts = parse::<u64>(&text).expect("parsing");

        assert_eq!(parsed_insts.len(), 1);
        let mut parsed_inst: u64 = parsed_insts[0].clone().into();
        parsed_inst = set_instruction_length_n(parsed_inst, instruction_length(instruction));
        let parsed_inst_text = format!(
            "{}",
            InstructionPrinter::new(
                parsed_inst
                    .try_into()
                    .expect("convert to tagged instruction"),
            )
        );
        assert_eq!(
            parsed_inst, instruction,
            "Current pc: 0x{:x}, original inst text: {}, parsed inst text: {}",
            pc, text, parsed_inst_text
        );

        execute(parsed_inst, &mut machine1).expect("execute on machine1");
        execute(instruction, &mut machine2).expect("execute on machine2");

        assert_eq!(machine1.pc(), machine2.pc(), "previous pc: {:x}", pc);
        assert_eq!(machine1.registers(), machine2.registers(), "registers");
        assert_eq!(machine1.exit_code(), machine2.exit_code(), "exit_code");
    }

    let result = machine1.exit_code();
    if result != 0 {
        println!("Error result: {:?}", result);
        std::process::exit(i32::from(result));
    }
}
