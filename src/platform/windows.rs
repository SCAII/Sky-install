use std::path::{Path, PathBuf};
use std::error::Error;
use std::fmt::Debug;
use error::InstallError;
use Args;

use platform::common::*;

// fs::remove_dir_all has issues in windows, so need to shell out
pub fn remove_tree<P: AsRef<Path> + Debug>(dir: P) -> Result<(), Box<Error>> {
    //rmdir c:\test /s /q
    println!("...removing {:?}", dir);
    let command: String = "rmdir".to_string();
    let mut args: Vec<String> = Vec::new();
    args.push(dir.as_ref().to_str().unwrap().to_string());
    args.push("/s".to_string());
    args.push("/q".to_string());
    let result_string = run_command(&protoc_hack(command), args)?;
    if result_string != "" {
        return Err(Box::new(InstallError::new(format!(
            "ERROR trying to delete files {}",
            result_string
        ))));
    }
    Ok(())
}
//shelling out to git on windows due to build error on Jed's windows laptop trying to build git2
// (cmake invocation of cl.exe uses forward slashes for path - likely explanation for dll adjacent to cl.exe
//  not being found)
pub fn get_core(install_dir: PathBuf, command_args: &Args) -> Result<(), Box<Error>> {
    use std::env;
    use platform::common;

    println!("");
    println!("");
    println!("installing core...");
    println!("");
    let orig_dir_pathbuf = env::current_dir()?;
    println!("...cd {:?}", orig_dir_pathbuf);
    env::set_current_dir(&install_dir)?;
    let command: String = "git".to_string();
    let mut args: Vec<String> = Vec::new();
    args.push("clone".to_string());
    args.push("https://github.com/SCAII/SCAII.git".to_string());
    let result_string = run_command(&command, args)?;
    verify_git_clone_success(&result_string)?;
    let mut scaii_dir = install_dir;
    scaii_dir.push("SCAII".to_string());
    if command_args.flag_branch {
        println!("...cd {:?}", scaii_dir);
        env::set_current_dir(scaii_dir.clone())?;
        checkout(command_args.arg_branch_name.clone())?;
    }
    ensure_google_closure_lib_installed(scaii_dir.clone())?;
    common::install_protobuf_javascript_lib(scaii_dir)?;
    env::set_current_dir(orig_dir_pathbuf)?;
    Ok(())
}

pub fn get_sky_rts(install_dir: PathBuf, command_args: &Args) -> Result<(), Box<Error>> {
    use std::env;

    println!("");
    println!("");
    println!("installing Sky-RTS...");
    println!("");
    let orig_dir_pathbuf = env::current_dir()?;
    println!("...cd {:?}", orig_dir_pathbuf);
    env::set_current_dir(install_dir.clone())?;
    let command: String = "git".to_string();
    let mut args: Vec<String> = Vec::new();
    args.push("clone".to_string());
    args.push("https://github.com/SCAII/Sky-RTS.git".to_string());
    let result_string = run_command(&command, args)?;
    verify_git_clone_success(&result_string)?;
    if command_args.flag_branch {
        let mut sky_rts_dir = install_dir.clone();
        sky_rts_dir.push("Sky-RTS".to_string());
        println!("...cd {:?}", sky_rts_dir);
        env::set_current_dir(sky_rts_dir)?;
        checkout(command_args.arg_branch_name.clone())?;
    }
    env::set_current_dir(orig_dir_pathbuf)?;
    Ok(())
}

pub fn copy_built_core(source_dir: PathBuf, target: PathBuf) -> Result<(), Box<Error>> {
    //cp target/release/scaii_core.dll ~/.scaii/bin/
    copy_source_named(source_dir, target, "scaii_core.dll".to_string())
}

pub fn copy_built_rts(source_dir: PathBuf, target: PathBuf) -> Result<(), Box<Error>> {
    //cp target/release/scaii_core.dll ~/.scaii/bin/
    copy_source_named(source_dir, target, "backend.dll".to_string())
}

pub fn copy_recursive<P: AsRef<Path> + Debug>(source: PathBuf, dest: P) -> Result<(), Box<Error>> {
    println!("...copying files from {:?} to {:?}", source, dest);
    let command: String = "xcopy".to_string();
    let mut args: Vec<String> = Vec::new();
    args.push(source.as_path().to_str().unwrap().to_string());
    args.push("/i".to_string());
    args.push("/s".to_string());
    args.push(dest.as_ref().to_str().unwrap().to_string());
    let _result_string = run_command(&protoc_hack(command), args)?;
    Ok(())
}

pub fn run_command(command: &str, args: Vec<String>) -> Result<String, Box<Error>> {
    use std::process::{Command, Stdio};
    use error::InstallError;
    use platform::common;

    let mut c = Command::new("cmd");
    let c = c.arg("/C");
    let c = c.arg(command);
    for arg in args.iter() {
        c.arg(arg);
    }
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
