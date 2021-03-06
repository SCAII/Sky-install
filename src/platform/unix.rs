extern crate git2;

use error::InstallError;
use std::error::Error;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use Args;

use platform::common::*;

#[path = "macos.rs"]
#[cfg(target_os = "macos")]
pub mod os_specific;

#[path = "linux.rs"]
#[cfg(target_os = "linux")]
pub mod os_specific;

pub use os_specific::*;
use os_specific::{copy_built_core, run_command};

pub fn remove_tree<P: AsRef<Path> + Debug>(dir: P) -> Result<(), Box<Error>> {
    println!("removing {:?}", dir);
    let command: String = "rm".to_string();
    let mut args: Vec<String> = Vec::new();
    args.push("-rf".to_string());
    args.push(dir.as_ref().to_str().unwrap().to_string());
    let result_string = run_command(&command, args)?;
    if result_string != "" {
        return Err(Box::new(InstallError::new(format!(
            "ERROR trying to delete files {}",
            result_string
        ))));
    }
    Ok(())
}

pub fn get_core(install_dir: PathBuf, command_args: &Args) -> Result<(), Box<Error>> {
    use self::git2::Repository;
    use std::env;

    println!("installing core...");
    let orig_dir_pathbuf = env::current_dir()?;
    println!("{:?}", orig_dir_pathbuf);
    env::set_current_dir(install_dir.clone())?;

    let command: String = "git".to_string();
    let mut args: Vec<String> = Vec::new();
    args.push("clone".to_string());
    args.push("https://github.com/SCAII/SCAII.git".to_string());
    let result_string = run_command(&command, args)?;
    verify_git_clone_success(&result_string)?;

    // let url = "https://github.com/SCAII/SCAII.git";
    // let _repo = match Repository::clone(url, install_dir.clone().to_str().unwrap()) {
    //     Ok(repo) => repo,
    //     Err(e) => panic!("failed to clone: {}", e),
    // };
    let mut scaii_dir = install_dir.clone();
    scaii_dir.push("SCAII".to_string());
    if command_args.flag_branch {
        println!("{:?}", scaii_dir);
        env::set_current_dir(scaii_dir.clone())?;
        checkout(command_args.arg_branch_name.clone())?;
    }
    ensure_google_closure_lib_installed(scaii_dir.clone())?;
    install_protobuf_javascript_lib(scaii_dir)?;
    env::set_current_dir(orig_dir_pathbuf)?;
    Ok(())
}

pub fn get_sky_rts(install_dir: PathBuf, command_args: &Args) -> Result<(), Box<Error>> {
    use self::git2::Repository;
    use std::env;

    println!("installing Sky-RTS...");
    let orig_dir_pathbuf = env::current_dir()?;
    println!("{:?}", orig_dir_pathbuf);
    env::set_current_dir(install_dir.clone())?;

    let command: String = "git".to_string();
    let mut args: Vec<String> = Vec::new();
    args.push("clone".to_string());
    args.push("https://github.com/SCAII/Sky-RTS.git".to_string());
    let result_string = run_command(&command, args)?;
    verify_git_clone_success(&result_string)?;

    // let url = "https://github.com/SCAII/Sky-RTS.git";
    // let _repo = match Repository::clone(url, install_dir.clone().to_str().unwrap()) {
    //     Ok(repo) => repo,
    //     Err(e) => panic!("failed to clone: {}", e),
    // };
    if command_args.flag_branch {
        let mut sky_rts_dir = install_dir.clone();
        sky_rts_dir.push("Sky-RTS".to_string());
        println!("{:?}", sky_rts_dir);
        env::set_current_dir(sky_rts_dir.clone())?;
        checkout(command_args.arg_branch_name.clone())?;
    }
    env::set_current_dir(orig_dir_pathbuf)?;
    Ok(())
}

pub fn copy_recursive<P: AsRef<Path> + Debug>(source: PathBuf, dest: P) -> Result<(), Box<Error>> {
    println!("copying {:?} to {:?}", source, dest);
    let command: String = "cp".to_string();
    let mut args: Vec<String> = Vec::new();
    args.push("-a".to_string());
    args.push(source.to_str().unwrap().to_string());
    args.push(dest.as_ref().to_str().unwrap().to_string());
    let result_string = run_command(&command, args)?;
    if !(result_string == "".to_string()) {
        return Err(Box::new(InstallError::new(format!(
            "ERROR - problem copying files {:?}",
            result_string
        ))));
    }
    Ok(())
}
