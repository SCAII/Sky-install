use std::error::Error;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::process::Output;

pub fn verify_git_clone_success(result_string: &str) -> Result<(), Box<Error>> {
    use error::InstallError;

    if result_string.starts_with("error") || result_string.starts_with("fatal") {
        return Err(Box::new(InstallError::new(format!(
            "ERROR - git pull failed : {}",
            result_string
        ))));
    }
    Ok(())
}

#[inline]
pub fn protoc_hack(command: String) -> String {
    if command == "\"protoc\"" {
        String::from("protoc")
    } else {
        command
    }
}

pub fn checkout(branch: String) -> Result<(), Box<Error>> {
    use error::InstallError;
    use platform;

    let command: String = "git".to_string();
    let mut args: Vec<String> = Vec::new();
    args.push("checkout".to_string());
    args.push(branch.clone());
    let result_string = platform::run_command(&command, args)?;
    if !result_string.starts_with("error") {
        return Ok(());
    } else {
        return Err(Box::new(InstallError::new(format!(
            "ERROR - problem checking out branch {} : {}",
            branch, result_string
        ))));
    }
}

pub fn ensure_google_closure_lib_installed(scaii_root: PathBuf) -> Result<(), Box<Error>> {
    use error::InstallError;

    //\SCAII\viz\js\closure-library\closure\bin
    let mut closure_dir = scaii_root.clone();
    closure_dir.push("viz");
    closure_dir.push("js");
    closure_dir.push("closure-library");
    if closure_dir.as_path().exists() {
        println!("closure library already installed at {:?}.", closure_dir);
        Ok(())
    } else {
        println!("...installing google closure library");
        let mut closure_install_dir = scaii_root;
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
        let install_result = install_google_closure_library(
            closure_install_dir,
            url,
            filename,
            String::from("closure-library-20171112"),
        );
        match install_result {
            Ok(_) => Ok(()),
            Err(error) => Err(Box::new(InstallError::new(format!(
                "google closure library download appears to have failed: {:?}",
                error.description()
            )))),
        }
    }
}

pub fn install_google_closure_library(
    mut closure_install_dir: PathBuf,
    url: String,
    filename: String,
    orig_unzipped_dir_name: String,
) -> Result<PathBuf, Box<Error>> {
    use error::InstallError;
    use std::env;
    use std::fs;

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
                Err(Box::new(InstallError::new(format!(
                    "google closure library download appears to have failed \
                     - file not present {:?}",
                    closure_zip_path
                ))))
            } else {
                println!("...unzipping");
                let f = fs::File::open(&closure_zip_path)?;
                unzip_file(&closure_install_dir, f)?;
                let mut closure_temp_dir_name = closure_install_dir.clone();
                closure_temp_dir_name.push(&orig_unzipped_dir_name);

                closure_install_dir.push("closure-library");
                let rename_result = fs::rename(&orig_unzipped_dir_name, &closure_install_dir);
                let result = match rename_result {
                    Ok(_) => {
                        if closure_install_dir.exists() {
                            Ok(closure_install_dir)
                        } else {
                            Err(Box::new(InstallError::new(format!(
                                "{:?} does not exist after unzipping closure bundle.",
                                closure_install_dir
                            ))))
                        }
                    }
                    Err(error) => {
                        println!("{}", error.description());
                        Err(Box::new(InstallError::new(format!(
                            "could not rename {:?} to {:?}.",
                            closure_temp_dir_name, closure_install_dir
                        ))))
                    }
                }?;
                fs::remove_file(closure_zip_path)?;
                Ok(result)
            }
        }
        Err(error) => Err(Box::new(InstallError::new(format!(
            "tried using curl library to download protoc from {} , but hit error: {}",
            url,
            error.description()
        )))),
    }
}

pub fn copy_source_named(
    source_dir: PathBuf,
    target_dir: PathBuf,
    source_filename: String,
    dest_filename: String,
) -> Result<(), Box<Error>> {
    let mut source: PathBuf = source_dir;
    source.push(source_filename);
    let mut target: PathBuf = target_dir;
    target.push(dest_filename);
    copy_file(&source, &target)?;
    Ok(())
}

pub fn copy_file<P1: AsRef<Path>, P2: AsRef<Path>>(source: P1, dest: P2) -> Result<(), Box<Error>> {
    use error::InstallError;
    use std::fs;

    let src = source.as_ref().to_str().unwrap();
    let dst = dest.as_ref().to_str().unwrap();
    println!("copying {} to {}", src, dst);
    let copy_result = fs::copy(src, dst);
    match copy_result {
        Ok(_) => Ok(()),
        Err(err) => Err(Box::new(InstallError::new(format!(
            "ERROR - could not copy core binary: {:?}",
            err.description()
        )))),
    }
}

fn download_using_curl(url: &String, target_path: &PathBuf) -> Result<(), Box<Error>> {
    use curl::easy::{Easy2, Handler, WriteError};
    use std::io::Write;
    use std::fs;

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

fn append_relative_path(mut path_buf: PathBuf, subdir: &str) -> PathBuf {
    let parts_iter = subdir.split("/");
    for part in parts_iter {
        match part {
            "" => {
                // do nothing
            }
            ref x => {
                path_buf.push(x);
            }
        }
    }
    path_buf
}

fn unzip_file(parent: &PathBuf, zip_file: File) -> Result<(), Box<Error>> {
    use std::io::{Read, Write};
    use zip;
    use std::fs;

    let mut zip = try!(zip::ZipArchive::new(&zip_file));
    println!(
        "unzipping {:?}... zip file count is {}",
        zip_file,
        zip.len()
    );
    for i in 0..zip.len() {
        let mut zip_file = zip.by_index(i).unwrap();
        let file_size = zip_file.size();
        match file_size {
            0 => {
                ensure_subdir_exists(parent.clone(), zip_file.name())?;
            }
            _ => {
                let path = append_relative_path(parent.clone(), zip_file.name());
                let mut buf: Vec<u8> = Vec::new();
                let _read_result_usize = zip_file.read_to_end(&mut buf)?;
                let mut output_file = fs::File::create(path)?;
                output_file.write_all(buf.as_slice())?;
            }
        }
    }
    Ok(())
}

pub fn emit_error_output(output: &Output) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr != "" {
        println!("   ...stderr: {}", stderr);
    }
}

pub fn ensure_dir_exists(path_buf: &PathBuf) -> Result<(), Box<Error>> {
    use std::fs;
    if !path_buf.as_path().exists() {
        fs::create_dir_all(path_buf.as_path())?;
    }
    Ok(())
}

fn ensure_subdir_exists(mut path_buf: PathBuf, subdir: &str) -> Result<(), Box<Error>> {
    let parts_iter = subdir.split("/");
    for part in parts_iter {
        match part {
            "" => {
                // do nothing
            }
            ref x => {
                path_buf.push(x);
                ensure_dir_exists(&path_buf)?;
            }
        }
    }
    Ok(())
}

pub fn install_protobuf_javascript_lib(install_dir: PathBuf) -> Result<(), Box<Error>> {
    use std::env;
    use platform;

    println!("...installing google protobuf javascript library...");
    let orig_dir_pathbuf = env::current_dir()?;
    let mut js_dir = install_dir.clone();
    js_dir.push("viz".to_string());
    js_dir.push("js".to_string());
    println!("...cd {:?}", js_dir);
    env::set_current_dir(js_dir.as_path())?;
    let command: String = "git".to_string();
    let mut args: Vec<String> = Vec::new();
    args.push("clone".to_string());
    args.push("https://github.com/google/protobuf".to_string());
    println!("...cloning repo");
    let result_string = platform::run_command(&protoc_hack(command.clone()), args)?;
    verify_git_clone_success(&result_string)?;

    let mut protobuf_slash_js_dir = js_dir.clone();
    protobuf_slash_js_dir.push("protobuf".to_string());
    protobuf_slash_js_dir.push("js".to_string());

    let mut protobuf_js_dir = js_dir.clone();
    protobuf_js_dir.push("protobuf_js".to_string());
    println!("...copying javascript portion");
    platform::copy_recursive(protobuf_slash_js_dir, &protobuf_js_dir)?;

    let mut protobuf_dir = js_dir.clone();
    protobuf_dir.push("protobuf".to_string());
    platform::remove_tree(&protobuf_dir)?;
    env::set_current_dir(orig_dir_pathbuf.as_path())?;
    Ok(())
}
