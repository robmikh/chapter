use std::error::Error;
use std::fs;
use std::path::PathBuf;

use prost::Message;

use chapter::*;

mod test_plan;

fn compile(yarn_path: &PathBuf) {
    let yarn_path_str = yarn_path.to_str().unwrap();
    let output_path = yarn_path.parent().unwrap();
    let output_path_str = output_path.as_os_str().to_str().unwrap();

    let status = std::process::Command::new("ysc.exe")
                                .args([
                                    "compile",
                                    "--output-directory",
                                    output_path_str,
                                    yarn_path_str,
                                ])
                                .status()
                                .unwrap();
                            assert!(status.success());

}

fn ensure_yarnc(yarnc_path_str: &str) {
    assert!(yarnc_path_str.ends_with(".yarnc"));
    let yarnc_path = PathBuf::from(yarnc_path_str);
    if !yarnc_path.exists() {
        let yarn_path = {
            let mut path = yarnc_path.clone();
            path.set_extension("yarn");
            path
        };
        compile(&yarn_path);
    }
}

fn test_script_with_setup<F: FnOnce(&mut test_plan::PlanRunner)>(yarnc_path: &str, setup: F) -> Result<(), Box<dyn Error>> {
    ensure_yarnc(yarnc_path);
    let mut runner = test_plan::PlanRunner::new(yarnc_path);
    setup(&mut runner);
    runner.run()
}

fn test_script(yarnc_path: &str) -> Result<(), Box<dyn Error>> {
    test_script_with_setup(yarnc_path, |_|{})
}

fn set_up_vm(yarnc_path: &str) -> VirtualMachine {
    let _ = pretty_env_logger::try_init();
    ensure_yarnc(yarnc_path);

    let proto_path = PathBuf::from(yarnc_path);

    // Read the file's bytes and load a Program.
    let proto_data = fs::read(&proto_path).unwrap();
    let program = Program::decode(&*proto_data).unwrap();

    // Load LineInfos from a csv file.
    let mut csv_path = proto_path;
    csv_path.set_file_name(format!(
        "{}-Lines.csv",
        csv_path.file_stem().unwrap().to_str().unwrap()
    ));

    let mut csv_reader = csv::Reader::from_path(csv_path).unwrap();
    let _string_table: Vec<LineInfo> = csv_reader
        .deserialize()
        .map(|result| result.unwrap())
        .collect();

    let mut vm = VirtualMachine::new(program);
    vm.library.insert(
        "assert".to_string(),
        FunctionInfo::new(1, &|_vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            if !parameters[0].as_bool() {
                assert!(false, "Assertion failed");
            }
        }),
    );
    vm.library.insert(
        "add_three_operands".to_string(),
        FunctionInfo::new_returning(3, &|_vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            let res = parameters[0].add(&parameters[1]).unwrap();
            res.add(&parameters[2]).unwrap()
        }),
    );
    vm.library.insert(
        "last_value".to_string(),
        FunctionInfo::new_returning(-1, &|_vm: &mut VirtualMachine, parameters: &[YarnValue]| {
            parameters.last().unwrap().clone()
        }),
    );

    vm
}

#[test]
fn test_commands() -> Result<(), Box<dyn Error>> {
    test_script("test_files/Commands.yarnc")
}

#[test]
fn test_expressions() -> Result<(), Box<dyn Error>> {
    let mut vm = set_up_vm("test_files/Expressions.yarnc");

    vm.set_node("Start")?;
    while vm.execution_state != ExecutionState::Stopped {
        vm.continue_dialogue()?;
    }

    Ok(())
}

#[test]
fn test_format_functions() -> Result<(), Box<dyn Error>> {
    test_script("test_files/FormatFunctions.yarnc")
}

#[test]
fn test_number_functions() -> Result<(), Box<dyn Error>> {
    test_script("test_files/NumberFunctions.yarnc")
}

#[test]
fn test_functions() -> Result<(), Box<dyn Error>> {
    let mut vm = set_up_vm("test_files/Functions.yarnc");

    vm.set_node("Start")?;
    while vm.execution_state != ExecutionState::Stopped {
        vm.continue_dialogue()?;
    }

    Ok(())
}

#[test]
fn test_if_statements() -> Result<(), Box<dyn Error>> {
    test_script("test_files/IfStatements.yarnc")
}

#[test]
fn test_inline_expressions() -> Result<(), Box<dyn Error>> {
    test_script("test_files/InlineExpressions.yarnc")
}

#[test]
fn test_random_functions() -> Result<(), Box<dyn Error>> {
    test_script_with_setup("test_files/RandomFunctions.yarnc", |runner| {
        runner.get_vm().set_random_seed(12345);
    })
}

#[test]
fn test_shortcut_options() -> Result<(), Box<dyn Error>> {
    test_script("test_files/ShortcutOptions.yarnc")
}

#[test]
fn test_smileys() -> Result<(), Box<dyn Error>> {
    test_script("test_files/Smileys.yarnc")
}

#[test]
fn test_tags() -> Result<(), Box<dyn Error>> {
    test_script("test_files/Tags.yarnc")
}

#[test]
fn test_types() -> Result<(), Box<dyn Error>> {
    let mut vm = set_up_vm("test_files/Types.yarnc");

    vm.set_node("Start")?;
    while vm.execution_state != ExecutionState::Stopped {
        vm.continue_dialogue()?;
    }

    Ok(())
}

#[test]
fn test_variable_storage() -> Result<(), Box<dyn Error>> {
    let mut vm = set_up_vm("test_files/VariableStorage.yarnc");

    vm.set_node("Start")?;
    while vm.execution_state != ExecutionState::Stopped {
        vm.continue_dialogue()?;
    }

    Ok(())
}

#[test]
fn test_visited_counter() -> Result<(), Box<dyn Error>> {
    test_script("test_files/VisitedCounter.yarnc")
}
