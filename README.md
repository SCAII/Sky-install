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
SCAII is known to work on Windows 10 and MacOS. Linux should work but has not been throughly tested.

## Windows Installation
1. Install C++ Dependencies
	1. Install Visual Studio Build Tools
		- Link: https://www.visualstudio.com/downloads/#build-tools-for-visual-studio-2017
	2. Select all options as shown.
		- ![Alt text](https://github.com/Sky-install/tree/master/images/visual_studio_installer_windows.PNG"oh hai there")

2. Install Rust
	- Link: https://www.rust-lang.org/en-US/install.html

3. Install Python 3
	- Link: https://www.python.org/downloads/

4. Clone Sky-Install Repo
	- Command: `git clone https://github.com/SCAII/Sky-install.git`

5. Enter the Sky-Install Repo directory and build the installer
	- Command: `cargo build`

6. Execute Sky-Install and install SCAII
	- Command: `cargo run install dev release`
	- All installed files are placed in user's home directory `~/.scaii/`

## MacOS Installation
WIP