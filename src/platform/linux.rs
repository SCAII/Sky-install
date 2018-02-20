extern crate git2;

use std::error::Error;
use std::path::{Path, Pathbuf};

pub fn copy_built_rts<P: AsRef<Path>>(source_dir: PathBuf, target: P) -> Result<(), Box<Error>> {
    //cp target/release/libscaii_core.so ~/.scaii/bin/
    copy_source_named(source_dir, target, "libbackend.so".to_string())
}

pub fn copy_built_core<P: AsRef<Path>>(source_dir: PathBuf, target: P) -> Result<(), Box<Error>> {
    //cp target/release/libscaii_core.so ~/.scaii/bin/
    copy_source_named(source_dir, target, "libscaii_core.so".to_string())
}

pub fn run_command(command: &str, args: Vec<String>) -> Result<String, Box<Error>> {
    use std::process::{Command, Stdio};
    use error::InstallError;
    use platform::common;

    let mut c = Command::new("sh");
    let c = c.arg("-c");
    let c = c.arg("\"");
    let c = c.arg(command);
    for arg in args.iter() {
        c.arg(arg);
    }
    let c = c.arg("\"");
    println!("...running command {:?}", c);
    let output = c.stdout(Stdio::inherit())
        .output()
        .expect(&String::as_str(format!(
            "failed to launch command {}",
            command
        )));
    common::emit_error_output(&output);
    if output.status.success() {
        let result = String::from_utf8(output.stdout);
        match result {
            Ok(output_string) => Ok(output_string),
            Err(_utf8_convert_error) => Err(Box::new(InstallError::new(
                "problem converting command result from utf8",
            ))),
        }
    } else {
        Err(Box::new(InstallError::new(
            String::from_utf8_lossy(&output.stderr).to_string(),
        )))
    }
}
