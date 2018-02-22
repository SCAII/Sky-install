extern crate git2;

use std::error::Error;
use std::path::{Path, PathBuf};

pub fn copy_built_core(source_dir: PathBuf, target: PathBuf) -> Result<(), Box<Error>> {
    use platform::common;
    //cp target/release/??? ~/.scaii/bin/
    common::copy_source_named(source_dir, target, "libscaii_core.dylib".to_string())
}

pub fn copy_built_rts(source_dir: PathBuf, target: PathBuf) -> Result<(), Box<Error>> {
    use platform::common;
    //cp target/release/??? ~/.scaii/bin/
    common::copy_source_named(source_dir, target, "libsky-rts.dylib".to_string())
}

pub fn run_command(command: &str, args: Vec<String>) -> Result<String, Box<Error>> {
    use std::process::{Command, Stdio};
    use error::InstallError;
    use platform::common;
    use std::env;

    // note - using the sh -c approach on Mac caused the chmod command to fail.  Leaving them out
    // let it succeed, so left it that way assuming all commands would be similar.
    //let mut c = Command::new("sh");
    //let c = c.arg("-c");
    //let c = c.arg(command);
    let mut c = Command::new(command);
    for arg in args.iter() {
        c.arg(arg);
    }
    println!("...in dir...{:?}", env::current_dir());
    println!("...running command {:?}", c);
    let output = c.stdout(Stdio::inherit())
        .output()
        .expect(&String::as_str(&format!(
            "failed to launch command {}",
            command
        )));

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
