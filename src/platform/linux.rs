extern crate git2;

use std::error::Error;
use std::path::{Path, PathBuf};

pub fn copy_built_rts(source_dir: PathBuf, target: PathBuf) -> Result<(), Box<Error>> {
    use platform::common;
    use std::fs;
    // rename libbackend.so to libsky-rts.source
    common::copy_source_named(
        source_dir,
        target,
        "libbackend.so".to_string(),
        "libsky-rts.so".to_string(),
    )
}

pub fn copy_built_core(source_dir: PathBuf, target: PathBuf) -> Result<(), Box<Error>> {
    use platform::common;
    //cp target/release/libscaii_core.so ~/.scaii/bin/
    common::copy_source_named(
        source_dir,
        target,
        "libscaii_core.so".to_string(),
        "libscaii_core.so".to_string(),
    );
}

pub fn run_command(command: &str, args: Vec<String>) -> Result<String, Box<Error>> {
    use std::process::{Command, Stdio};
    use error::InstallError;
    use platform::common;

    let mut c = Command::new(command);
    for arg in args {
        c.arg(arg);
    }
    println!("...running command {:?}", c);
    let output = c.stdout(Stdio::inherit())
        .output()
        .expect(&format!("failed to launch command {}", command));
    common::emit_error_output(&output);
    if output.status.success() {
        let result = String::from_utf8(output.stdout);
        match result {
            Ok(output_string) => Ok(output_string),
            Err(_utf8_convert_error) => Err(Box::new(InstallError::new(
                "problem converting command result from utf8".to_string(),
            ))),
        }
    } else {
        Err(Box::new(InstallError::new(
            String::from_utf8_lossy(&output.stderr).to_string(),
        )))
    }
}
