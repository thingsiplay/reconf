# reconf for RetroArch

Edit or view data from **Re**troArch **conf**ig files

- **Author**: Tuncay D.
- **License**: [MIT License](LICENSE)
- **Source**: [Github](https://github.com/thingsiplay/reconf)
- **Download**: [Github](https://github.com/thingsiplay/reconf/releases)

## Introduction

**reconf** is a non interactive CLI application for scripting or directly in
the terminal.

The configuration of RetroArch is saved in a simple file format with file
extension ".cfg" and ".opt". **reconf** can read and write to those files with
simple commands in the terminal or a script. While this can be achieved with
general purpose tools like `sed` and `awk` in example, there are still reasons
why I wrote this program.  RetroArch priotizes the first encounter of a key if
it is found multiple times in the file (important when reading or writing to
the file). Working with quotation marks (as the values require it to handle) on
the commandline adds some complexity to it too. These little annoyances and a
few other features are the reason why **reconf** exist, not because it is
necessary.

Use the options `reconf --help` to list all options and `reconf --show-usage`
for a tutorial with examples.

### Example

```bash
$ reconf file1.cfg -l
input_player1_joypad_index
aspect_ratio_index
video_threaded
video_max_swapchain_images

$ reconf file1.cfg -g aspect_ratio_index
0

$ reconf file1.cfg -u update_ppsspp.cfg -e combined.cfg -o -w
input_player2_joypad_index = "nul"
input_player3_joypad_index = "nul"
screenshot_directory = "~/.config/retroarch/screenshots/PPSSPP"
input_player1_joypad_index = "2"
aspect_ratio_index = "22"
video_threaded = "true"
video_max_swapchain_images = "2"
```

### Features

- view with simple commands
- search with regex by key name, value or both
- update content from stdin or other files
- adds new keys to top of the file for priority reasons
- output each file to stdout
- merge files
- sort content alphabetically
- highlight keys and values with color

### Quick Start

1. Download **reconf** from
   [Releases](https://github.com/thingsiplay/reconf/releases) and unpack it.
2. Optionally, install it in a directory within your systems `$PATH`.
3. Show the help `reconf --help` and a simple tutorial `reconf --show-usage`.
4. Optionally, create an alias to always output file content and filename with
   coloring activated: `alias cfg='reconf -ofc1'`

## Requirements

The program itself does not require anything special. The program is written in
Rust and there are two different Linux binaries precompiled, plus an
experimental Windows version.

## Installation

At the moment there is no installer or any package. Just download the
standalone binary, put it in a folder within your systems `$PATH` and run it in
the commandline (or in a script). You can also download the source code and
build it with the Rust tools.

### Compile from source

If you download the source code, then you can simply build it with the Rust
tools. If you want build one of the target, then you need to install

```
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-unknown-linux-musl
rustup target add x86_64-pc-windows-gnu
```

The Makefile can be executed with `make`, but it is written from perspective of
a Linux system (`bash`, `grep`, `tar`... ). It also makes use of `cargo clippy`,
`cargo fmt`, `pandoc` and `upx`. Build a release version with `make release` or
everything with `make all`. You can also easily ignore the Makefile and just do
`cargo build --release`, in which case you don't need all the above targets.

In anyway, building with the Makefile will create the binaries in a subfolder
"reconf". If you use `cargo build --release` directly, then the binary is found
in the folder "target/release".

## RetroArch config format

As this program is for viewing and editing RetroArch config files, let's
discuss what format this is. These files have a file ending ".cfg" or ".opt".
But not all ".cfg" files are compatible. Supported are in example the
"retroarch.cfg" in your main RetroArch folder and the various ".cfg" and ".opt"
files for each core found in the subfolders of "config".

Each line represents a setting. These are usually saved and loaded by RetroArch.

- key has no space
- value is always enclosed in quotation marks: `""`
- key and value are separated by equal sign: `=`
- first encounter of key has priority over any following with same name
- comments start with a hash symbol: `#`
- escape quotation marks on values and do not allow equation sign on key names
   when writing

## Known Bugs, Limitations and Quirks

- Comments are not supported and will be removed when saving file. Not a big
  deal, because usually the config files do not have comments. But if you do,
  then beware of this issue.
- Like RetroArch, if a key is found multiple times in a file, then the first
  one is taken for priority reason. But unlike RetroArch, the other keys with
  same name are basically deleted.
- Any quotation marks inside values are currently not handled, so don't use
  them, program will add surrounding quotation marks anyway.
