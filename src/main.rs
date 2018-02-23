extern crate curl;
extern crate zip;

use std::error::Error;
use std::env;
use std::path::PathBuf;
use std::fs;

pub(crate) mod error;

pub(crate) mod platform;
use platform::*;

//  install into .scaii/git by default
//  ___enhancement - user can override location by specifing --here , if build commands don't find under scaii git, then look "here" by default
// ___how check for failure of git call on windows?

pub struct Args {
    flag_branch: bool,
    arg_branch_name: String,
}

fn usage() {
    println!("\nUsage:");
    println!("\tsky-install get-core    [branch]   \t pull SCAII code");
    println!("\tsky-install get-sky-rts [branch]   \t pull default RTS");
    println!("\tsky-install build-core             \t build/configure core");
    println!("\tsky-install build-sky-rts          \t build/configure default RTS");
    println!("\tsky-install clean-core-all         \t remove core pull and build artifacts");
    println!("\tsky-install clean-core-build       \t remove core build artifacts");
    println!("\tsky-install clean-sky-rts-all      \t remove RTS pull and build artifacts");
    println!("\tsky-install clean-sky-rts-build    \t remove RTS build artifacts");
    println!("\tsky-install full-install [branch]  \t pull/build/configure all");
    println!("\tsky-install full-clean             \t clean all");
    println!("\tsky-install re-install   [branch]  \t shallow clean/build/configure all (faster than full-install)");
    println!("");
    println!("via cargo...");
    println!("  cargo build -- <command> [branch]");
}

fn main() {
    let arguments: Vec<String> = env::args().collect();
    let args = parse_args(&arguments);
    let command = arguments[1].clone();
    let result = try_command(&command, args);
    match result {
        Ok(()) => {}
        Err(err) => {
            println!("ERROR running command {} : {}", &command, err.description());
            usage();
        }
    }
}

fn parse_args(arguments: &Vec<String>) -> Args {
    let mut args = Args {
        flag_branch: false,
        arg_branch_name: "".to_string(),
    };
    if arguments.len() < 2 {
        usage();
        std::process::exit(0);
    } else if arguments.len() > 2 {
        args.flag_branch = true;
        args.arg_branch_name = arguments[2].clone();
    } else if arguments.len() == 2 {
        args.flag_branch = true;
        args.arg_branch_name = "dev".to_string();
        println!("No branch specified, defaulting to 'dev'");
    }
    args
}

fn try_command(command: &String, args: Args) -> Result<(), Box<Error>> {
    use platform::*;

    let install_dir = get_default_install_dir()?;
    match command.as_ref() {
        "get-core" => {
            try_clean_core_all(install_dir.clone())?;
            get_core(install_dir, &args)
        }
        "get-sky-rts" => {
            try_clean_sky_rts_all(install_dir.clone())?;
            get_sky_rts(install_dir, &args)
        }
        "build-core" => build_core(&install_dir),
        "build-sky-rts" => build_sky_rts(install_dir),
        "clean-core-all" => try_clean_core_all(install_dir),
        "clean-core-build" => try_clean_core_build(),
        "clean-sky-rts-all" => try_clean_sky_rts_all(install_dir),
        "clean-sky-rts-build" => try_clean_sky_rts_build(),
        "full-install" => {
            try_clean_core_all(install_dir.clone())?;
            try_clean_sky_rts_all(install_dir.clone())?;
            get_core(install_dir.clone(), &args)?;
            get_sky_rts(install_dir.clone(), &args)?;
            build_core(&install_dir)?;
            build_sky_rts(install_dir.clone())?;
            Ok(())
        }
        "re-install" => {         
            if !(install_dir.exists()){
                println!("Previous installation not found. Please run with argument 
                    \n\t> full-install [branch]");
                std::process::exit(0);
            }else{
                println!("Previous compile exists. Skipping clean re-compile.
                     \nIf you want to perform a fresh install please run with argumet 
                     \n\t> full-install [branch]");
                shallow_clean();
            }
            build_core(&install_dir)?;
            build_sky_rts(install_dir.clone())?;
            Ok(())
        }
        "full-clean" => {
            try_clean_core_all(install_dir.clone())?;
            try_clean_sky_rts_all(install_dir)?;
            Ok(())
        }
        _ => {
            println!("unknown install command:  {}", command);
            usage();
            Ok(())
        }
    }
}

fn try_clean_core_all(install_dir: PathBuf) -> Result<(), Box<Error>> {
    let mut success: bool = false;
    let mut count = 0;
    while !success {
        let result = clean_core_all(install_dir.clone());
        match result {
            Ok(_) => {
                success = true;
            }
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
    let mut success: bool = false;
    let mut count = 0;
    while !success {
        let result = clean_core_build();
        match result {
            Ok(_) => {
                success = true;
            }
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

fn clean_core_all(install_dir: PathBuf) -> Result<(), Box<Error>> {
    println!("");
    println!("");
    println!("Removing core pull...");
    println!("");
    let mut scaii_dir = install_dir;
    scaii_dir.push("SCAII".to_string());
    if scaii_dir.as_path().exists() {
        remove_tree(&scaii_dir)?;
    }
    clean_core_build()?;
    Ok(())
}

fn try_clean_sky_rts_all(install_dir: PathBuf) -> Result<(), Box<Error>> {
    let mut success: bool = false;
    let mut count = 0;
    while !success {
        let result = clean_sky_rts_all(install_dir.clone());
        match result {
            Ok(_) => {
                success = true;
            }
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
    let mut success: bool = false;
    let mut count = 0;
    while !success {
        let result = clean_sky_rts_build();
        match result {
            Ok(_) => {
                success = true;
            }
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
        println!("...removing sky-rts binary {:?}", sky_binary);
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

fn clean_sky_rts_all(install_dir: PathBuf) -> Result<(), Box<Error>> {
    println!("");
    println!("");
    println!("removing Sky-RTS pull...");
    println!("");
    let mut rts_dir = install_dir;
    rts_dir.push("Sky-RTS".to_string());
    if rts_dir.as_path().exists() {
        remove_tree(&rts_dir)?;
    }
    clean_sky_rts_build()?;
    Ok(())
}

fn get_dot_scaii_dir() -> Result<PathBuf, Box<Error>> {
    use platform::common;
    let mut home_dir_pathbuf = get_home_dir()?;
    home_dir_pathbuf.push(".scaii".to_string());
    common::ensure_dir_exists(&home_dir_pathbuf)?;
    Ok(home_dir_pathbuf)
}

fn get_default_install_dir() -> Result<PathBuf, Box<Error>> {
    use platform::common;

    let mut install_dir_pathbuf = get_dot_scaii_dir()?;
    install_dir_pathbuf.push("git".to_string());
    common::ensure_dir_exists(&install_dir_pathbuf)?;
    Ok(install_dir_pathbuf)
}

fn build_core(install_dir: &PathBuf) -> Result<(), Box<Error>> {
    use error::InstallError;
    use common;

    println!("");
    println!("");
    println!("building core...");
    println!("");
    let orig_dir_pathbuf = env::current_dir()?;
    //cd SCAII/
    let mut scaii_install_dir = install_dir.clone();
    scaii_install_dir.push("SCAII".to_string());
    if !scaii_install_dir.as_path().exists() {
        return Err(Box::new(InstallError::new(
            "scaii core has not been installed - run 'get-core' command first.".to_string(),
        )));
    }
    println!("...cd {:?}", scaii_install_dir);
    env::set_current_dir(scaii_install_dir.as_path())?;

    //cargo build --release
    let command: String = "cargo".to_string();
    let mut args: Vec<String> = Vec::new();
    args.push("build".to_string());
    args.push("--release".to_string());
    let build_output = run_command(&command, args)?;
    if build_output.contains("error") {
        return Err(Box::new(InstallError::new(format!(
            "ERROR - cargo build failed {:?}",
            build_output
        ))));
    }

    //mkdir ~/.scaii
    //mkdir ~/.scaii/bin
    let mut bindir = get_dot_scaii_dir()?;
    bindir.push("bin");
    common::ensure_dir_exists(&bindir)?;

    //cp target/release/libscaii_core.so ~/.scaii/bin/
    assert!(scaii_install_dir.ends_with("SCAII"));
    let mut source = scaii_install_dir.clone();
    source.push("target".to_string());
    source.push("release".to_string());
    let target = bindir.clone();
    copy_built_core(source, target)?;

    //cp -r glue ~/.scaii/
    let mut source = scaii_install_dir.clone();
    source.push("glue".to_string());
    let mut dest = get_dot_scaii_dir()?;
    dest.push("glue".to_string());
    copy_recursive(source, &dest)?;
    env::set_current_dir(orig_dir_pathbuf.as_path())?;
    Ok(())
}

fn shallow_clean() -> Result<(), Box<Error>> {
    let mut dir = get_dot_scaii_dir()?;
    
    dir.push("backends".to_string());
    if dir.as_path().exists() {
        remove_tree(&dir)?;
        println!("..Shallow clean :: removed {:?}", dir);
    }
    dir.pop();

    dir.push("bin".to_string());
    if dir.as_path().exists() {
        remove_tree(&dir)?;
        println!("..Shallow clean :: removed {:?}", dir);
    }
    dir.pop();

    dir.push("glue".to_string());
    if dir.as_path().exists() {
        remove_tree(&dir)?;
        println!("..Shallow clean :: removed {:?}", dir);
        
    }
    dir.pop();
    Ok(())
}  

fn get_home_dir() -> Result<PathBuf, Box<Error>> {
    use error::InstallError;

    let result: Option<PathBuf> = env::home_dir();
    match result {
        Some(pathbuf) => Ok(pathbuf),
        None => Err(Box::new(InstallError::new(
            "could not determine user's home directory".to_string(),
        ))),
    }
}

fn build_sky_rts(install_dir: PathBuf) -> Result<(), Box<Error>> {
    use error::InstallError;
    use platform::common;

    println!("");
    println!("");
    println!("building sky-rts...");
    println!("");
    let mut sky_rts_dir = install_dir;
    sky_rts_dir.push("Sky-RTS");
    if !sky_rts_dir.as_path().exists() {
        return Err(Box::new(InstallError::new(
            "Sky-RTS has not been installed - run 'get-sky-rts' command first.".to_string(),
        )));
    }

    let orig_dir_pathbuf = env::current_dir()?;
    //mkdir ~/.scaii/backends
    let mut dir = get_dot_scaii_dir()?;
    common::ensure_dir_exists(&dir)?;
    dir.push("backends".to_string());
    common::ensure_dir_exists(&dir)?;

    //mkdir ~/.scaii/backends/bin
    dir.push("bin".to_string());
    common::ensure_dir_exists(&dir)?;

    //mkdir ~/.scaii/backends/sky-rts
    dir.pop();
    dir.push("sky-rts".to_string());
    common::ensure_dir_exists(&dir)?;

    //mkdir ~/.scaii/backends/sky-rts/maps
    dir.push("maps".to_string());
    common::ensure_dir_exists(&dir)?;

    //mkdir ~/.scaii/glue/python/scaii/env/sky_rts
    dir.pop();
    dir.pop();
    dir.pop();
    dir.push("glue".to_string());
    dir.push("python".to_string());
    dir.push("scaii".to_string());
    dir.push("env".to_string());
    dir.push("sky_rts".to_string());
    common::ensure_dir_exists(&dir)?;

    // # Part B, Build  Sky-RTS
    // cd ../Sky-RTS/
    // cd backend/
    let mut backend = sky_rts_dir.clone();
    backend.push("backend".to_string());
    println!("...cd {:?}", backend);
    env::set_current_dir(backend.as_path())?;

    //cargo build --release
    let command: String = "cargo".to_string();
    let mut args: Vec<String> = Vec::new();
    args.push("build".to_string());
    args.push("--release".to_string());
    let build_output = run_command(&command, args)?;
    if build_output.contains("error") {
        return Err(Box::new(InstallError::new(format!(
            "ERROR - cargo build failed {:?}",
            build_output
        ))));
    }

    // cp target/release/libbackend.so ~/.scaii/backends/bin/libsky-rts.so
    let mut source = backend.clone();
    source.push("target".to_string());
    source.push("release".to_string());

    let mut dest = get_dot_scaii_dir()?;
    dest.push("backends".to_string());
    dest.push("bin".to_string());
    copy_built_rts(source, dest)?;

    // cp -r game_wrapper/python/* ~/.scaii/glue/python/scaii/env/sky_rts/
    let mut source = sky_rts_dir.clone();
    source.push("game_wrapper".to_string());
    source.push("python".to_string());

    if cfg!(target_os = "windows") {
        source.push("*".to_string()); 
    }else {
        source.push(".".to_string());
    }
    let mut dest = get_dot_scaii_dir()?;
    dest.push("glue");
    dest.push("python");
    dest.push("scaii");
    dest.push("env");
    dest.push("sky_rts");
    copy_recursive(source, &dest)?;

    // cp backend/lua/* ~/.scaii/backends/sky-rts/maps
    let mut source = backend.clone();
    source.push("lua".to_string());
    
    if cfg!(target_os = "windows") {
        source.push("*".to_string()); 
    }else {
        source.push(".".to_string());
    }

    let mut dest = get_dot_scaii_dir()?;
    dest.push("backends".to_string());
    dest.push("sky-rts".to_string());
    dest.push("maps".to_string());
    //dest.push("tower_example.lua".to_string());

    copy_recursive(source, &dest)?;

    // export PYTHONPATH=$PYTHONPATH:/home/lamki/.scaii/bin:/home/lamki/.scaii/glue/python/
    // export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/home/lamki/.scaii/bin/
    env::set_current_dir(orig_dir_pathbuf.as_path())?;
    Ok(())
}
