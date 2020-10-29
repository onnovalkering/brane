use brane_exec::{ExecuteInfo, hpc};
use futures::executor::block_on;

#[test]
fn it_adds_two() {
    let image = String::from("ubuntu:20.04");
    let command = Some(vec![String::from("echo"), String::from("'hello'")]);
    let exec = ExecuteInfo::new(image, None, None, command);

    let result = block_on(hpc::run(exec));

    assert!(result.is_ok());
}
