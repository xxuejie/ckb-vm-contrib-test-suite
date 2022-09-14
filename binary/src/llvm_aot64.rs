use ckb_vm_contrib::{
    ckb_vm::Bytes,
    llvm_aot::{DlSymbols, LlvmAotMachine, LlvmCompilingMachine},
};
use std::process::Command;
use tempfile::Builder;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let filename = args[0].clone();
    let code = std::fs::read(&filename).unwrap().into();
    let args: Vec<Bytes> = args.into_iter().map(|a| a.into()).collect();

    LlvmCompilingMachine::initialize().expect("initialize");
    let compiling_machine =
        LlvmCompilingMachine::load(&filename, &code, "test_suite_aot", &(|_| 0))
            .expect("loading binary to compile");
    let object_file = Builder::new().suffix(".o").tempfile().expect("tempfile");
    let object_path = object_file.path().to_str().expect("tempfile");
    let object = compiling_machine.aot(true).expect("aot");
    std::fs::write(object_path, &object).expect("write");

    let library_file = Builder::new().suffix(".so").tempfile().expect("tempfile");
    let library_path = library_file.path().to_str().expect("tempfile");
    let mut cmd = Command::new("gcc");
    cmd.arg("-shared")
        .arg("-o")
        .arg(library_path)
        .arg(object_path);
    let output = cmd.output().expect("cmd");
    assert!(output.status.success(), "cmd error");

    let dl_symbols = DlSymbols::new(library_path, "test_suite_aot").expect("dl symbols");
    let aot_symbols = &dl_symbols.aot_symbols;

    let mut machine =
        LlvmAotMachine::new(4 * 1024 * 1024, &aot_symbols).expect("create aot machine");
    machine.load_program(&code, &args).expect("load to run");
    // TODO: we should aim at finishing all the tests in fast mode
    let result = machine.run(false).expect("run");

    if result != 0 {
        println!("Error result: {:?}", result);
        std::process::exit(i32::from(result));
    }
}
