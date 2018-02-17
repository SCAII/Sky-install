extern crate zip;
extern crate curl;
#[cfg(any(target_os="linux", target_os="macos"))]
extern crate git2;
use std::process::Command;
use std::fmt;
use std::error::Error;
use std::env;
//use std::path::{Path,PathBuf};
use std::path::PathBuf;
use std::fs;
#[cfg(any(target_os="linux", target_os="macos"))]
use git2::Repository;
use std::process::Output;


//  install into .scaii/git by default
//  ___enhancement - user can override location by specifing --here , if build commands don't find under scaii git, then look "here" by default
// ___how check for failure of git call on windows?

#[derive(Debug)]
struct InstallError {
    details: String
}

impl InstallError {
    fn new(msg: &str) -> InstallError {
        InstallError{details: msg.to_string()}
    }
}

impl fmt::Display for InstallError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.details)
    }
}

impl Error for InstallError {
    fn description(&self) -> &str {
        &self.details
    }
}
struct Args {
    flag_branch : bool,
    arg_branch_name : String,
}

fn usage() {
    println!("Usage:");
    println!("  sky-install get-core    [branch]   // pull SCAII code");
    println!("  sky-install get-sky-rts [branch]   // pull default RTS");
    println!("  sky-install build-core             // build/configure core");
    println!("  sky-install build-sky-rts          // build/configure default RTS");
    println!("  sky-install clean-core-all         // remove core pull and build artifacts");
    println!("  sky-install clean-core-build       // remove core build artifacts");
    println!("  sky-install clean-sky-rts-all      // remove RTS pull and build artifacts");
    println!("  sky-install clean-sky-rts-build    // remove RTS build artifacts");
    println!("  sky-install full-install [branch]  // pull/build/configure all");
    println!("  sky-install full-clean             // pull/build/configure all");
    println!("");
    println!("via cargo...");
    println!("  cargo build -- <command> [branch]");
}

fn main() {
    let arguments: Vec<String> = env::args().collect();
    let command = arguments[1].clone();
    let args = parse_args(arguments);
    let result = try_command(&command, args);
    match result {
        Ok(()) => {},
        Err(err) => {
            println!("ERROR running command {} : {}", &command, err.description());
            usage();
        },
    }
}

fn parse_args(arguments : Vec<String>) -> Args {
    let mut args = Args {
        flag_branch : false,
        arg_branch_name : "".to_string(),
    };
    if arguments.len() > 2 {
        args.flag_branch = true;
        args.arg_branch_name = arguments[2].clone();
    }
    args
}

fn try_command(command : &String, args : Args) -> Result<(), Box<Error>> {
    let install_dir = get_default_install_dir()?;
    match command.as_ref() {
        "get-core" => {
            try_clean_core_all(&install_dir)?;
            get_core(&install_dir, &args)
        },
        "get-sky-rts" => { 
            try_clean_sky_rts_all(&install_dir)?;
            get_sky_rts(&install_dir, &args)
        },
        "build-core" => { build_core(&install_dir) },
        "build-sky-rts" => { build_sky_rts(&install_dir) },
        "clean-core-all" => { try_clean_core_all(&install_dir) },
        "clean-core-build" => { try_clean_core_build() },
        "clean-sky-rts-all" => { try_clean_sky_rts_all(&install_dir) },
        "clean-sky-rts-build" => { try_clean_sky_rts_build() },
        "full-install" => {
            try_clean_core_all(&install_dir)?;
            try_clean_sky_rts_all(&install_dir)?;
            get_core     (&install_dir, &args)?;
            get_sky_rts  (&install_dir, &args)?;
            build_core   (&install_dir)?;
            build_sky_rts(&install_dir)?;
            Ok(())
        },
        "full-clean" => {
            try_clean_core_all(&install_dir)?;
            try_clean_sky_rts_all(&install_dir)?;
            Ok(())
        },
        _ => {
            println!("unknown install command:  {}", command);
            usage();
            Ok(())
        }
    }
}

// fs::remove_dir_all has issues in windows, so need to shell out
#[cfg(target_os="windows")]
fn remove_tree(dir : &PathBuf) -> Result<(), Box<Error>> {
    //rmdir c:\test /s /q
    println!("...removing {:?}", dir);
    let command : String = "rmdir".to_string();
    let mut args: Vec<String> = Vec::new();
    args.push(dir.as_path().to_str().unwrap().to_string());
    args.push("/s".to_string());
    args.push("/q".to_string());
    let result_string = run_command(&command, args)?;
    if result_string != "" {
        return Err(Box::new(InstallError::new(&format!("ERROR trying to delete files {}", result_string))))
    }
    Ok(())
}

#[cfg(any(target_os="linux", target_os="macos"))]
fn remove_tree(dir : &Path) ->  Result<(), Box<Error>> {
    println!("...removing {:?}", dir);
    let command : String = "rm".to_string();
    let mut args: Vec<String> = Vec::new();
    args.push("-rf".to_string());
    args.push(dir.as_path().to_str().unwrap().to_string());
    let result_string = run_command(&command, args)?;
    if result_string != "" {
        return Err(Box::new(InstallError::new(&format!("ERROR trying to delete files {}", result_string))))
    }
    Ok(())
}

fn try_clean_core_all(install_dir: &PathBuf) -> Result<(), Box<Error>> {
    let mut success : bool = false;
    let mut count = 0;
    while !success {
        let result = clean_core_all(install_dir);
        match result {
            Ok(_) => { success = true;},
            Err(err) => { 
                count = count + 1;
                if count > 2 {
                    return Err(err);
                }
            }
        }
    }
    Ok(())
}

fn try_clean_core_build() -> Result<(), Box<Error>> {
    let mut success : bool = false;
    let mut count = 0;
    while !success {
        let result = clean_core_build();
        match result {
            Ok(_) => { success = true;},
            Err(err) => { 
                count = count + 1;
                if count > 2 {
                    return Err(err);
                }
            }
        }
    }
    Ok(())
}

fn clean_core_build() -> Result<(), Box<Error>> {
    println!("");
    println!("");
    println!("Removing core build artifacts...");
    println!("");
    //rm ~/.scaii/bin/scaii.core
    let mut scaii_core_path = get_dot_scaii_dir()?;
    scaii_core_path.push("bin");
    scaii_core_path.push("scaii.core".to_string());
    if scaii_core_path.as_path().exists() {
        println!("removing core binary {:?}", scaii_core_path);
        fs::remove_file(&scaii_core_path)?;
    }
    
    //rm ~/.scaii/glue
    let mut glue = get_dot_scaii_dir()?;
    glue.push("glue".to_string());
    if glue.as_path().exists() {
        remove_tree(&glue)?;
    }
    Ok(())
}

fn clean_core_all(install_dir: &PathBuf) -> Result<(), Box<Error>> {
    println!("");
    println!("");
    println!("Removing core pull...");
    println!("");
    let mut scaii_dir = install_dir.clone();
    scaii_dir.push("SCAII".to_string());
    if scaii_dir.as_path().exists() {
        remove_tree(&scaii_dir)?;
    }
    clean_core_build()?;
    Ok(())
}

fn try_clean_sky_rts_all(install_dir: &PathBuf) -> Result<(), Box<Error>> {
    let mut success : bool = false;
    let mut count = 0;
    while !success {
        let result = clean_sky_rts_all(install_dir);
        match result {
            Ok(_) => { success = true;},
            Err(err) => { 
                count = count + 1;
                if count > 2 {
                    return Err(err);
                }
            }
        }
    }
    Ok(())
}

fn try_clean_sky_rts_build() -> Result<(), Box<Error>> {
    let mut success : bool = false;
    let mut count = 0;
    while !success {
        let result = clean_sky_rts_build();
        match result {
            Ok(_) => { success = true;},
            Err(err) => { 
                count = count + 1;
                if count > 2 {
                    return Err(err);
                }
            }
        }
    }
    Ok(())
}

fn clean_sky_rts_build() -> Result<(), Box<Error>> {
    println!("");
    println!("");
    println!("removing Sky-RTS build artifacts...");
    println!("");
    // rm ~/.scaii/backends/bin/libsky-rts.so
    let mut sky_binary = get_dot_scaii_dir()?;
    sky_binary.push("backends".to_string());
    sky_binary.push("bin".to_string());
    sky_binary.push("sky-rts.scm".to_string());
    if sky_binary.as_path().exists() {
        println!("...removing sky-rts binary {:?}",sky_binary);
        fs::remove_file(&sky_binary)?;
    }

    // ~/.scaii/backends/sky-rts
    let mut dir = get_dot_scaii_dir()?;
    dir.push("backends".to_string());
    dir.push("sky-rts".to_string());
    if dir.as_path().exists() {
        remove_tree(&dir)?;
    }
    Ok(())
}

fn clean_sky_rts_all(install_dir: &PathBuf) -> Result<(), Box<Error>> {
    println!("");
    println!("");
    println!("removing Sky-RTS pull...");
    println!("");
    let mut rts_dir = install_dir.clone();
    rts_dir.push("Sky-RTS".to_string());
    if rts_dir.as_path().exists() {
        remove_tree(&rts_dir)?;
    }
    clean_sky_rts_build()?;
    Ok(())
}

fn get_dot_scaii_dir() -> Result<PathBuf, Box<Error>> {
    let mut home_dir_pathbuf = get_home_dir()?;
    home_dir_pathbuf.push(".scaii".to_string());
    ensure_dir_exists(&home_dir_pathbuf)?;
    Ok(home_dir_pathbuf)
}

fn get_default_install_dir() -> Result<PathBuf, Box<Error>> {
    let mut install_dir_pathbuf = get_dot_scaii_dir()?;
    install_dir_pathbuf.push("git".to_string());
    ensure_dir_exists(&install_dir_pathbuf)?;
    Ok(install_dir_pathbuf)
}

//shelling out to git on windows due to build error on Jed's windows laptop trying to build git2
// (cmake invocation of cl.exe uses forward slashes for path - likely explanation for dll adjacent to cl.exe 
//  not being found)
#[cfg(target_os="windows")]
fn get_core(install_dir : &PathBuf, command_args: &Args) -> Result<(), Box<Error>> {
    println!("");
    println!("");
    println!("installing core...");
    println!("");
    let orig_dir_pathbuf = env::current_dir()?;
    println!("...cd {:?}", orig_dir_pathbuf);
    env::set_current_dir(install_dir.as_path())?;
    let command : String = "git".to_string();
    let mut args: Vec<String> = Vec::new();
    args.push("clone".to_string());
    args.push("https://github.com/SCAII/SCAII.git".to_string());
    let result_string = run_command(&command, args)?;
    verify_git_clone_success(&result_string)?;
    let mut scaii_dir = install_dir.clone();
    scaii_dir.push("SCAII".to_string());
    if command_args.flag_branch {
        println!("...cd {:?}", scaii_dir);
        env::set_current_dir(scaii_dir.as_path())?;
        checkout(&command_args.arg_branch_name)?;
    }
    ensure_google_closure_lib_installed(&scaii_dir)?;
    install_protobuf_javascript_lib(&scaii_dir)?;
    env::set_current_dir(orig_dir_pathbuf.as_path())?;
    Ok(())
}

fn verify_git_clone_success(result_string : &String) -> Result<(), Box<Error>>  {
    if result_string.starts_with("error") || result_string.starts_with("fatal" ){
        return Err(Box::new(InstallError::new(&format!("ERROR - git pull failed : {}", result_string))));
    }
    Ok(())
}

#[cfg(any(target_os="linux", target_os="macos"))]
fn get_core(mut install_dir : &PathBuf, command_args: &Args) -> Result<(), Box<Error>>{
    println!("");
    println!("");
    println!("installing core...");
    println!("");
    let orig_dir_pathbuf = env::current_dir()?;
    println!("...cd {:?}", orig_dir_pathbuf);
    env::set_current_dir(install_dir.as_path())?;
    let url = "https://github.com/SCAII/SCAII.git";
    let repo = match Repository::clone(url, install_dir.to_str()) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to clone: {}", e),
    };
    let scaii_dir = install_dir.clone();
    scaii_dir.push("SCAII".to_string());
    if command_args.flag_branch {
        println!("...cd {:?}", scaii_dir);
        env::set_current_dir(scaii_dir.as_path())?;
        checkout(&command_args.arg_branch_name)?;
    }
    ensure_google_closure_lib_installed(&scaii_dir)?;
    install_protobuf_javascript_lib(&scaii_dir)?;
    env::set_current_dir(orig_dir_pathbuf.as_path())?;
    Ok(())
}

#[cfg(target_os="windows")]
fn get_sky_rts(install_dir : &PathBuf, command_args: &Args) -> Result<(), Box<Error>>{
    println!("");
    println!("");
    println!("installing Sky-RTS...");
    println!("");
    let orig_dir_pathbuf = env::current_dir()?;
    println!("...cd {:?}", orig_dir_pathbuf);
    env::set_current_dir(install_dir.as_path())?;
    let command : String = "git".to_string();
    let mut args: Vec<String> = Vec::new();
    args.push("clone".to_string());
    args.push("https://github.com/SCAII/Sky-RTS.git".to_string());
    let result_string = run_command(&command, args)?;
    verify_git_clone_success(&result_string)?;
    if command_args.flag_branch {
        let mut sky_rts_dir = install_dir.clone();
        sky_rts_dir.push("Sky-RTS".to_string());
        println!("...cd {:?}", sky_rts_dir);
        env::set_current_dir(sky_rts_dir.as_path())?;
        checkout(&command_args.arg_branch_name)?;
    }
    env::set_current_dir(orig_dir_pathbuf.as_path())?;
    Ok(())
}

#[cfg(any(target_os="linux", target_os="macos"))]
fn get_sky_rts(install_dir : &PathBuf, command_args: &Args) -> Result<(), Box<Error>>{
    println!("");
    println!("");
    println!("installing Sky-RTS...");
    println!("");
    let orig_dir_pathbuf = env::current_dir()?;
    println!("...cd {:?}", orig_dir_pathbuf);
    env::set_current_dir(install_dir.as_path())?;
    let url = "https://github.com/SCAII/Sky-RTS.git";
    let repo = match Repository::clone(url, install_dir.to_str()) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to clone: {}", e),
    };
    if command_args.flag_branch {
        install_dir.push("Sky-RTS".to_string());
        println!("...cd {:?}", install_dir);
        env::set_current_dir(install_dir.as_path())?;
        checkout(&command_args.arg_branch_name)?;
    }
    env::set_current_dir(orig_dir_pathbuf.as_path())?;
    Ok(())
}


fn checkout(branch : &String) -> Result<(), Box<Error>> {
    let command : String = "git".to_string();
    let mut args: Vec<String> = Vec::new();
    args.push("checkout".to_string());
    args.push(branch.clone());
    let result_string = run_command(&command, args)?;
    if !result_string.starts_with("error") {
        return Ok(());
    }
    else {
        return Err(Box::new(InstallError::new(&format!("ERROR - problem checking out branch {} : {}", branch, result_string))));
    }
}

fn build_core(install_dir : &PathBuf) -> Result<(), Box<Error>> {
    println!("");
    println!("");
    println!("building core...");
    println!("");
    let orig_dir_pathbuf = env::current_dir()?;
    //cd SCAII/
    let mut scaii_install_dir = install_dir.clone();
    scaii_install_dir.push("SCAII".to_string());
    if !scaii_install_dir.as_path().exists(){
        return Err(Box::new(InstallError::new("scaii core has not been installed - run 'get-core' command first.")));
    }
    println!("...cd {:?}", scaii_install_dir);
    env::set_current_dir(scaii_install_dir.as_path())?;
    
    //cargo build --release
    let command : String = "cargo".to_string();
    let mut args: Vec<String> = Vec::new();
    args.push("build".to_string());
    args.push("--release".to_string());
    let build_output = run_command(&command, args)?;
    if build_output.contains("error") {
        return Err(Box::new(InstallError::new(&format!("ERROR - cargo build failed {:?}", build_output))));
    }

    //mkdir ~/.scaii
    //mkdir ~/.scaii/bin
    let mut bindir = get_dot_scaii_dir()?;
    bindir.push("bin");
    ensure_dir_exists(&bindir)?;

    //cp target/release/libscaii_core.so ~/.scaii/bin/
    assert!(scaii_install_dir.ends_with("SCAII"));
    let mut source = scaii_install_dir.clone();
    source.push("target".to_string());
    source.push("release".to_string());
    let mut target = bindir.clone();
    target.push("scaii.core".to_string());
    copy_built_core(&source, &target)?;

    //cp -r glue ~/.scaii/
    let mut source = scaii_install_dir.clone();
    source.push("glue".to_string());
    let mut dest = get_dot_scaii_dir()?;
    dest.push("glue".to_string());
    copy_recursive(&source, &dest)?;
    env::set_current_dir(orig_dir_pathbuf.as_path())?;
    Ok(())
}

fn copy_source_named(source_dir: &PathBuf, target: &PathBuf, source_filename : String) -> Result<(), Box<Error>>{
    let mut source: PathBuf = source_dir.clone();
    source.push(source_filename);
    copy_file(&source, &target)?;
    Ok(())
}

#[cfg(target_os="windows")]
fn copy_built_core(source_dir : &PathBuf, target : &PathBuf) -> Result<(), Box<Error>>{
    //cp target/release/scaii_core.dll ~/.scaii/bin/
    copy_source_named(source_dir, target, "scaii_core.dll".to_string())
}

#[cfg(target_os="macos")]
fn copy_built_core(source_dir : &PathBuf, target : &PathBuf) -> Result<(), Box<Error>>{
    //cp target/release/??? ~/.scaii/bin/
    copy_source_named(source_dir, target, "libscaii_core.dylib".to_string())
}

#[cfg(target_os="linux")]
fn copy_built_core(source_dir: &PathBuf, target : &PathBuf) -> Result<(), Box<Error>> {
    //cp target/release/libscaii_core.so ~/.scaii/bin/
    copy_source_named(source_dir, target, "libscaii_core.so".to_string())
}

#[cfg(target_os="windows")]
fn copy_built_rts(source_dir : &PathBuf, target : &PathBuf) -> Result<(), Box<Error>>{
    //cp target/release/scaii_core.dll ~/.scaii/bin/
    copy_source_named(source_dir, target, "backend.dll".to_string())
}

#[cfg(target_os="macos")]
fn copy_built_rts(source_dir : &PathBuf, target : &PathBuf) -> Result<(), Box<Error>>{
    //cp target/release/??? ~/.scaii/bin/
    copy_source_named(source_dir, target, "libbackend.dylib".to_string())
}

#[cfg(target_os="linux")]
fn copy_built_rts(source_dir: &PathBuf, target : &PathBuf) -> Result<(), Box<Error>> {
    //cp target/release/libscaii_core.so ~/.scaii/bin/
    copy_source_named(source_dir, target, "libbackend.so".to_string())
}


fn copy_file(source : &PathBuf , dest : &PathBuf) -> Result<(), Box<Error>> {
    let src = source.as_path().to_str().unwrap();
    let dst = dest.as_path().to_str().unwrap();
    println!("copying {} to {}", src, dst);
    let copy_result = fs::copy(src, dst); 
    match copy_result {
        Ok(_) => Ok(()),
        Err(err) => {
            Err(Box::new(InstallError::new(&format!("ERROR - could not copy core binary: {:?}", err.description() ))))
        }
    }
}

fn get_home_dir() -> Result<PathBuf, Box<Error>> {
    let result : Option<PathBuf> = env::home_dir();
    match result {
        Some(pathbuf) => { Ok(pathbuf) },
        None => {
            Err(Box::new(InstallError::new("could not determine user's home directory")))
        }
    }
}

fn build_sky_rts(_install_dir : &PathBuf) -> Result<(), Box<Error>> {
    println!("");
    println!("");
    println!("building sky-rts...");
    println!("");
    let mut sky_rts_dir = get_default_install_dir()?;
    sky_rts_dir.push("Sky-RTS");
    if !sky_rts_dir.as_path().exists(){
        return Err(Box::new(InstallError::new("Sky-RTS has not been installed - run 'get-sky-rts' command first.")));
    }

    let orig_dir_pathbuf = env::current_dir()?;
    //mkdir ~/.scaii/backends
    let mut dir = get_dot_scaii_dir()?;
    ensure_dir_exists(&dir)?;
    dir.push("backends".to_string());
    ensure_dir_exists(&dir)?;

    //mkdir ~/.scaii/backends/bin
    dir.push("bin".to_string());
    ensure_dir_exists(&dir)?;

    //mkdir ~/.scaii/backends/sky-rts
    dir.pop();
    dir.push("sky-rts".to_string());
    ensure_dir_exists(&dir)?;

    //mkdir ~/.scaii/backends/sky-rts/maps
    dir.push("maps".to_string());
    ensure_dir_exists(&dir)?;

    //mkdir ~/.scaii/glue/python/scaii/env/sky_rts
    dir.pop();
    dir.pop();
    dir.push("glue".to_string());
    dir.push("python".to_string());
    dir.push("scaii".to_string());
    dir.push("env".to_string());
    dir.push("sky_rts".to_string());
    ensure_dir_exists(&dir)?;

    // # Part B, Build  Sky-RTS 
    // cd ../Sky-RTS/
    // cd backend/
    let mut backend = sky_rts_dir.clone();
    backend.push("backend".to_string());
    println!("...cd {:?}", backend);
    env::set_current_dir(backend.as_path())?;
    
    //cargo build --release
    let command : String = "cargo".to_string();
    let mut args: Vec<String> = Vec::new();
    args.push("build".to_string());
    args.push("--release".to_string());
    let build_output = run_command(&command, args)?;
    if build_output.contains("error") {
        return Err(Box::new(InstallError::new(&format!("ERROR - cargo build failed {:?}", build_output))));
    }
    
    // cp target/release/libbackend.so ~/.scaii/backends/bin/libsky-rts.so
    let mut source = backend.clone();
    source.push("target".to_string());
    source.push("release".to_string());

    let mut dest = get_dot_scaii_dir()?;
    dest.push("backends".to_string());
    dest.push("bin".to_string());
    dest.push("sky-rts.scm".to_string());
    copy_built_rts(&source, &dest)?;

    // cp -r game_wrapper/python/* ~/.scaii/glue/python/scaii/env/sky_rts/
    let mut source = sky_rts_dir.clone();
    source.push("game_wrapper".to_string());
    source.push("python".to_string());
    source.push("*".to_string());
    let mut dest = get_dot_scaii_dir()?;
    dest.push("glue");
    dest.push("python");
    dest.push("scaii");
    dest.push("env");
    dest.push("sky_rts");
    copy_recursive(&source, &dest)?;

    // cp backend/lua/tower_example.lua ~/.scaii/backends/sky-rts/maps
    let mut source = backend.clone();
    source.push("lua".to_string());
    source.push("tower_example.lua".to_string());
    let mut dest = get_dot_scaii_dir()?;
    dest.push("backends".to_string());
    dest.push("sky-rts".to_string());
    dest.push("maps".to_string());
    dest.push("tower_example.lua".to_string());
    copy_file(&source, &dest)?;
    // export PYTHONPATH=$PYTHONPATH:/home/lamki/.scaii/bin:/home/lamki/.scaii/glue/python/ 
    // export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/home/lamki/.scaii/bin/
    env::set_current_dir(orig_dir_pathbuf.as_path())?;
    Ok(())
}


#[cfg(target_os="windows")]
fn copy_recursive(source : &PathBuf, dest: &PathBuf) -> Result<(), Box<Error>> {
    println!("...copying files from {:?} to {:?}", source, dest);
    let command : String = "xcopy".to_string();
    let mut args: Vec<String> = Vec::new();
    args.push(source.as_path().to_str().unwrap().to_string());
    args.push("/i".to_string());
    args.push("/s".to_string());
    args.push(dest.as_path().to_str().unwrap().to_string());
    let _result_string = run_command(&command, args)?;
    Ok(())
}

#[cfg(any(target_os="linux", target_os="macos"))]
fn copy_recursive(source : &PathBuf, dest: &PathBuf) -> Result<(), Box<Error>> {
    println!("...copying files from {:?} to {:?}", source, dest);
    let command : String = "cp".to_string();
    let mut args: Vec<String> = Vec::new();
    args.push("-r".to_string());
    args.push(source.as_str());
    args.push(dest.as_str());
    let result_string = run_command(&command, args)?;
    if !(result_string = "".to_string()){
        return Err(Box::new(InstallError::new(&format!("ERROR - problem copying files {:?}", result_string))))
    }
    Ok(())
}

fn ensure_google_closure_lib_installed(scaii_root : &PathBuf) ->  Result<(), Box<Error>> {
    //\SCAII\viz\js\closure-library\closure\bin
    let mut closure_dir = scaii_root.clone();
    closure_dir.push("viz");
    closure_dir.push("js");
    closure_dir.push("closure-library");
    if closure_dir.as_path().exists() {
        println!("closure library already installed at {:?}.", closure_dir);
        Ok(())
    }
    else {
        println!("...installing google closure library");
        let mut closure_install_dir = scaii_root.clone();
        closure_install_dir.push("viz");
        closure_install_dir.push("js");
        
        //
        //  WEIRD - I was trying to download version v20170910 that I have been using all along 
        // (and thus testing against).  It turns out the version number in the package.json file are
        // a release behind, so I had to download v20171112.zip to get this desired version:
        // "version": "20170910.0.0",
        //
        //let filename = String::from("v20170910.zip");
        //let url = String::from("https://github.com/google/closure-library/archive/v20170910.zip");
        let filename = String::from("v20171112.zip");
        let url = String::from("https://github.com/google/closure-library/archive/v20171112.zip");
        let install_result = install_google_closure_library(closure_install_dir, url, filename, String::from("closure-library-20171112"));
        match install_result {
            Ok(_) => Ok(()),
            Err(error) =>Err(Box::new(InstallError::new(&format!("google closure library download appears to have failed: {:?}", error.description() ))))
        }
    }
}

fn install_google_closure_library(mut closure_install_dir : PathBuf, url : String, filename : String, orig_unzipped_dir_name: String) -> Result<PathBuf, Box<Error>> {
    println!("...cd {:?}", closure_install_dir);
    env::set_current_dir(&closure_install_dir)?;
    let mut closure_zip_path: PathBuf = closure_install_dir.clone();
    closure_zip_path.push(filename);
    println!("...downloading closure zip");
    let curl_result = download_using_curl(&url, &closure_zip_path);
    match curl_result {
        Ok(_) => {
            // verify expected file exists
            if !closure_zip_path.as_path().exists() {
                println!("ERROR - google closure library install failed.");
                Err(Box::new(InstallError::new(&format!("google closure library download appears to have failed - file not present {:?}", closure_zip_path ))))
            }
            else {
                println!("...unzipping");
                let f = fs::File::open(closure_zip_path)?;
                unzip_file(&closure_install_dir,f)?;
                let mut closure_temp_dir_name = closure_install_dir.clone();
                closure_temp_dir_name.push(&orig_unzipped_dir_name);
                
                closure_install_dir.push("closure-library");
                let rename_result = fs::rename(&orig_unzipped_dir_name, &closure_install_dir);
                match rename_result {
                    Ok(_) => {
                        if closure_install_dir.exists() {
                            Ok(closure_install_dir)
                        }
                        else {
                            Err(Box::new(InstallError::new(&format!("{:?} does not exist after unzipping closure bundle.",closure_install_dir))))
                        }
                    },
                    Err(error) => {
                        println!("{}", error.description());
                        Err(Box::new(InstallError::new(&format!("could not rename {:?} to {:?}.",closure_temp_dir_name,closure_install_dir))))
                    }
                }
                
                
            }
        }
        Err(error) => {
            Err(Box::new(InstallError::new(&format!("tried using curl library to download protoc from {} , but hit error: {}", url, error.description() ))))
        }
    }
}

fn install_protobuf_javascript_lib(install_dir :&PathBuf)  ->  Result<(), Box<Error>> {
    println!("...installing google protobuf javascript library...");
    let orig_dir_pathbuf = env::current_dir()?;
    let mut js_dir = install_dir.clone();
    js_dir.push("viz".to_string());
    js_dir.push("js".to_string());
    println!("...cd {:?}", js_dir);
    env::set_current_dir(js_dir.as_path())?;
    let command : String = "git".to_string();
    let mut args: Vec<String> = Vec::new();
    args.push("clone".to_string());
    args.push("https://github.com/google/protobuf".to_string());
    println!("...cloning repo");
    let result_string = run_command(&command, args)?;
    verify_git_clone_success(&result_string)?;
    
    let mut protobuf_slash_js_dir= js_dir.clone();
    protobuf_slash_js_dir.push("protobuf".to_string());
    protobuf_slash_js_dir.push("js".to_string());

    let mut protobuf_js_dir= js_dir.clone();
    protobuf_js_dir.push("protobuf_js".to_string());
    println!("...copying javascript portion");
    copy_recursive(&protobuf_slash_js_dir, &protobuf_js_dir)?;
    
    let mut protobuf_dir = js_dir.clone();
    protobuf_dir.push("protobuf".to_string());
    remove_tree(&protobuf_dir)?;
    env::set_current_dir(orig_dir_pathbuf.as_path())?;
    Ok(())
}

#[allow(dead_code)]
fn download_using_curl(url : &String, target_path: &PathBuf) ->  Result<(), Box<Error>> {
    use curl::easy::{Easy2, Handler, WriteError};
    use std::io::Write;
    struct Collector(Vec<u8>);

    impl Handler for Collector {
        fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
            self.0.extend_from_slice(data);
            Ok(data.len())
        }
    }
    
    let mut easy = Easy2::new(Collector(Vec::new()));
    easy.get(true).unwrap();
    easy.follow_location(true).unwrap();
    easy.url(url).unwrap();
    easy.perform().unwrap();

    assert_eq!(easy.response_code().unwrap(), 200);
    let contents = easy.get_ref();
    let mut output_file = fs::File::create(target_path)?;
    output_file.write_all(&contents.0)?;
    Ok(())
}

#[allow(dead_code)]
fn set_execute_permission(path_buf : &PathBuf) -> Result<(),Box<Error>>{
    if cfg!(target_os = "windows") {
        Ok(())
    } 
    else {
        let command = String::from("chmod");
        let mut args : Vec<String> = Vec::new();

        args.push(String::from("744"));
        args.push(String::from(path_buf.as_path().to_str().unwrap()));
        let result_string = run_command(&command, args)?;
        match result_string.as_str() {
            "" => Ok(()),
            ref x => Err(Box::new(InstallError::new(&format!("couldnot set execute permission on {:?} : {}", path_buf, x ))))
        }
    }
}

fn unzip_file(parent : &PathBuf, zip_file : fs::File) -> Result<(), Box<Error>> {
    use std::io::Read;
    use std::io::Write;
    let mut zip = try!(zip::ZipArchive::new(&zip_file));
    println!("unzipping {:?}... zip file count is {}", zip_file,zip.len());
    for i in 0..zip.len()
    {
        let mut zip_file = zip.by_index(i).unwrap();
        let file_size = zip_file.size();
        match file_size {
            0 => {
                ensure_subdir_exists(parent.clone(), zip_file.name())?;
            },
            _ => {
                let path = append_relative_path(parent.clone(), zip_file.name());
                let mut buf : Vec<u8> = Vec::new();
                let _read_result_usize = zip_file.read_to_end(&mut buf)?;
                let mut output_file = fs::File::create(path)?;
                output_file.write_all(buf.as_slice())?;
            }
        }
    }
    Ok(())
}


fn emit_output(output : &Output) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stdout != "" {
        println!("   ...stdout: {}", stdout);
    }
    if stderr != "" {
        println!("   ...stderr: {}", stderr);
    }
}

#[cfg(target_os="windows")]
fn run_command_platform(command: &String, args: Vec<String>) -> Result<String, Box<Error>> {
    let mut c = Command::new("cmd");
    let c = c.arg("/C");
    let c = c.arg(command);
    for arg in args.iter() {
        c.arg(arg);
    }
    println!("...running command {:?}", c);
    let output = c.output().expect(&String::as_str(
        &format!("failed to launch command {}", command),
    ));
    emit_output(&output);
    if output.status.success() {
        let result = String::from_utf8(output.stdout);
        match result{
            Ok(output_string) => Ok(output_string),
            Err(_utf8_convert_error) => Err(Box::new(InstallError::new("problem converting command result from utf8"))),
        }
    }
    else {
        Err(Box::new(InstallError::new(&String::from_utf8_lossy(&output.stderr))))
    }
}

#[cfg(target_os="linux")]
fn run_command_platform(command: &String, args: Vec<String>) -> Result<String, Box<Error>> {
    let mut c = Command::new("sh");
    let c = c.arg("-c");
    let c = c.arg(command);
    for arg in args.iter() {
        c.arg(arg);
    }
    println!("...running command {:?}", c);
    let output = c.output().expect(&String::as_str(
        &format!("failed to launch command {}", command),
    ));
    emit_output(&output);
    if output.status.success() {
        let result = String::from_utf8(output.stdout);
        match result{
            Ok(output_string) => Ok(output_string),
            Err(_utf8_convert_error) => Err(Box::new(InstallError::new("problem converting command result from utf8"))),
        }
    }
    else {
        Err(Box::new(InstallError::new(&String::from_utf8_lossy(&output.stderr))))
    }
}

#[cfg(target_os="macos")]
fn run_command_platform(command: &String, args: Vec<String>) -> Result<String, Box<Error>> {
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
    let output = c.output().expect(&String::as_str(
        &format!("failed to launch command {}", command),
    ));

    emit_output(&output);
    if output.status.success() {
        let result = String::from_utf8(output.stdout);
        match result{
            Ok(output_string) => Ok(output_string),
            Err(_utf8_convert_error) => Err(Box::new(InstallError::new("problem converting command result from utf8"))),
        }
    }
    else {
        Err(Box::new(InstallError::new(&String::from_utf8_lossy(&output.stderr))))
    }
}

fn run_command(command: &String, args: Vec<String>) -> Result<String, Box<Error>> {
    let hard_coded_protoc_command = String::from("protoc");
    let mut final_command: &String = &command.clone();
    if command == "\"protoc\"" { // has extra quotes that I can't figure out how to prevent (from PathBug)
        final_command = &hard_coded_protoc_command;
    }

    run_command_platform(final_command,args)
}


fn ensure_dir_exists(path_buf: &PathBuf) -> Result<(), Box<Error>> {
    if !path_buf.as_path().exists() {
        fs::create_dir_all(path_buf.as_path())?;
    }
    Ok(())
}

fn ensure_subdir_exists(mut path_buf: PathBuf, subdir : &str) -> Result<(), Box<Error>> {
    let parts_iter = subdir.split("/");
    for part in parts_iter {
        match part {
            "" => {
                // do nothing
            },
            ref x => {
                path_buf.push(x);
                ensure_dir_exists(&path_buf)?;
            }
        }
    }
    Ok(())
}

fn append_relative_path(mut path_buf: PathBuf, subdir : &str)-> PathBuf {
    let parts_iter = subdir.split("/");
    for part in parts_iter {
        match part {
            "" => {
                // do nothing
            },
            ref x => {
                path_buf.push(x);
            }
        }
    }
    path_buf
}

#[allow(dead_code)]
fn get_shared_parent_dir() -> PathBuf {
    let pathbuf_result = env::current_dir();
    match pathbuf_result {
        Ok(pathbuf) => {
            let parent_option = pathbuf.parent();
            match parent_option {
                Some(path_ref) => {
                    return path_ref.to_path_buf();
                }
                None => {
                    panic!("could not determine parent dir of this executable!");
                }
            }
        },
        Err(_) => {
            panic!("could not determine parent dir of this executable!");
        }
    }
}
