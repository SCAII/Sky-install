# Sky-install
Installer for SCAII

```
Usage:
    sky-install install <branch name> [debug|release]
    sky-install reinstall [debug|release]
    sky-install uninstall

Options:
    install         Performs a clean installation of
                    Sky-RTS and compiles release version.
                    Defaults to dev branch if not
                    specified.
    reinstall       Quickly recompiles and reinstalls
                    Sky-RTS without fetching the latest
                    version from Github.
    uninstall       Uninstalls Sky-Rts.
```

# Installation Instructions
SCAII RTS works on Windows 10 and MacOS. Linux should work (not tested).

## Windows Installation
1. Install C++ Dependencies
	1. Install Visual Studio Build Tools
		- Link: https://www.visualstudio.com/downloads/#build-tools-for-visual-studio-2017
	2. Select all options as shown.
		![alt text](https://raw.githubusercontent.com/SCAII/Sky-install/master/images/visual_studio_installer_windows.PNG "oh hai there")

2. Install Rust
	- Link: https://www.rust-lang.org/en-US/install.html
	- Command: `rustup default 1.26.2`

3. Install Python 3
	- Link: https://www.python.org/downloads/

4. Install Protobuf via PIP3
	- Command: `pip3 install protobuf`

5. Clone Sky-Install Repo
	- Command: `git clone https://github.com/SCAII/Sky-install.git`

6. Enter the Sky-Install Repo directory and build the installer
	- Command: `cargo build`

7. Execute Sky-Install and install SCAII
	- Command: `cargo run install dev release`
	- All installed files are placed in user's home directory `~/.scaii/`

8. Set environment PATH variables
	1. Create a new path variable called `PYTHONPATH` with value `C:\Users\<your windows profile name>\.scaii\glue\python` 
	2. Add a new environment variable to the existing Windows `Path` with value `C:\Users\<your windows profile name>\.scaii\bin`
	- Note: Spaces in profile names have messed stuff up
	![alt text](https://raw.githubusercontent.com/SCAII/Sky-install/master/images/windows_path.PNG "confusing picture")

## MacOS Installation
WIP
