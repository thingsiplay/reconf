mod parser;

use crate::parser::Config;

use std::error::Error;
use std::path::PathBuf;

// https://docs.rs/gumdrop/latest/gumdrop/
use gumdrop::Options;

// https://crates.io/crates/compact_str/
use compact_str::CompactString;

const APP_VERSION: &str = "0.1";
const APP_NAME: &str = "reconf";

#[derive(Debug, Options)]
#[allow(clippy::struct_excessive_bools)]
struct Arguments {
    #[options(
        free,
        help = "load and write editable RetroArch .cfg config files, multiple
                       files are processed individually, requires --write
                       option for saving to disk"
    )]
    file: Vec<PathBuf>,

    #[options(help = "print help message and exit\n")]
    help: bool,

    #[options(help = "print program name and version and exit\n", no_short)]
    version: bool,

    #[options(help = "print example usage and exit\n", no_short)]
    show_usage: bool,

    #[options(
        help = "load RetroArch .cfg config files for reading purpose only,
                       applies key=value pairs to all editable files and output
                       on the fly (edit)\n",
        meta = "FILE"
    )]
    update: Vec<PathBuf>,

    #[options(
        help = "enable reading text lines from stdin to update key=value pairs
                       from, acts like an anonymous file content (edit)\n",
        short = "i"
    )]
    stdin: bool,

    #[options(
        help = "update VALUE of existing pair or insert a new KEY, option can
                       be used multiple times (edit)\n",
        meta = "KEY VALUE"
    )]
    set: Vec<(CompactString, CompactString)>,

    #[options(
        help = "search the literal text SEARCH in value of existing KEY and
                       change matching text portion with literal text REPLACE,
                       option can be used multiple times (edit)\n",
        meta = "KEY SEARCH REPLACE"
    )]
    replace: Vec<(CompactString, CompactString, CompactString)>,

    #[options(
        help = "remove each key=value pair by searching exact KEY name, option
                       can be used multiple times (edit)\n",
        meta = "KEY"
    )]
    delete: Vec<CompactString>,

    #[options(
        help = "print value without quotes by searching exact KEY name, option
                       can be used multiple times (view)\n",
        meta = "KEY"
    )]
    get: Vec<CompactString>,

    #[options(
        help = "compare KEY names to regex pattern and print key=value pair for
                       each match, this option is only used once, can be
                       combined with '--value' so both has to match (view)\n",
        meta = "KEY"
    )]
    key: Option<CompactString>,

    #[options(
        help = "compare VALUE to regex pattern and print key=value pair for
                       each match, this option is only used once, can be
                       combined with '--key' so both has to match (view)\n",
        meta = "VALUE"
    )]
    value: Option<CompactString>,

    #[options(
        help = "a modifier to '--key' and '--value' options, it will turn them
                       to list mode where '-k' only prints name of keys and
                       '-v' only the value without quotation marks and key
                       name, if this option is used and no '--key' or '--value'
                       is given then it defaults to list all keys (view)\n"
    )]
    list: bool,

    #[options(
        help = "colorize keys and values when print to stdout, accepts a number:
                       '0'=none, '1' up to '9' are some predefined sets of color
                       combinations and style schemes (view)\n",
        meta = "PRESET"
    )]
    color: u8,

    #[options(
        help = "print associated file paths to stderr, includes an empty line
                       on stdout between every file (view)\n"
    )]
    filenames: bool,

    #[options(
        help = "sort keys for output of each file alphabetically (edit)\n",
        short = "S"
    )]
    sort: bool,

    #[options(help = "print config data to stdout, this is done after all
                       modifications and shows how the file would have been
                       saved to disk (view)\n")]
    output: bool,

    #[options(
        help = "force 'LF' line endings instead os default when writing a file,
                       combine it with option '--cr' to produce 'CRLF'\n",
        no_short
    )]
    lf: bool,

    #[options(
        help = "force 'CR' line endings instead os default when writing a file,
                       combine it with option '--lf' to produce 'CRLF'\n",
        no_short
    )]
    cr: bool,

    #[options(
        help = "force 'CRLF' line endings instead os default when writing a file,
                       ignore options '--cr' and '--lf'\n",
        no_short
    )]
    crlf: bool,

    #[options(help = "commit changes to associated files on disk, overwrite
                       existing files or create from scratch, unless --export
                       option is in effect, in which case don't overwrite
                       original files\n")]
    write: bool,

    #[options(
        help = "merge all data and write to a single file, requires --write
                       option for writing to disk (edit)",
        meta = "FILE"
    )]
    export: Option<CompactString>,
}

fn load_files(list_of_files: Vec<PathBuf>) -> Vec<Config> {
    let mut source_configs: Vec<Config> = vec![];

    for file in list_of_files {
        let mut new = Config::new();
        let path: PathBuf = match file.canonicalize() {
            Ok(fullpath) => fullpath,
            Err(_) => file,
        };
        new.load(
            &path
                .into_os_string()
                .into_string()
                .expect("File path must be valid."),
        );
        source_configs.push(new);
    }

    source_configs
}

fn read_stdin_config() -> Config {
    use std::io::prelude::*;
    let stdin = std::io::stdin();
    let lines: Vec<String> = stdin
        .lock()
        .lines()
        .map(std::result::Result::unwrap)
        .collect();
    let mut stdin_config = Config::new();

    for line in lines {
        stdin_config.insert_from_string(&line);
    }

    stdin_config
}

fn usage_message() -> CompactString {
    compact_str::format_compact!(
        "\
{APP_NAME} - Edit or view data from RetroArch config files

Usage: {APP_NAME} FILE... [OPTIONS]
       {APP_NAME} file.cfg -o
       {APP_NAME} file.cfg -g key
       {APP_NAME} *.cfg -fl

RetroArch uses a very simple configuration format with file ending \".cfg\" and
sometimes \".opt\". An example is your \"retroarch.cfg\". These store settings
as \"key\" on the left and \"value\" on the right of an equal sign. Value is
always enclosed between quotation marks, like so:

    input_1_joypad_index = \"1\"

This program can view parts of the file in different ways or help updating it's
values without complicated commands or escaping quotation marks. When multiple
options are present, then following priority is processed in order:

    1. Read editable files. In example: '{APP_NAME} file1.cfg'
    2. Load update files and apply them to parts of editable files.
    3. Read in stdin and apply content to parts of editable files.
    4. Apply content editing commands such as '--set' or '--replace'.
    5. Print name of file (but to stderr).
    6. Print matches from viewing commands such as '--get' or '--key'.
    7. Output entire file data with modifications in place.
    8. Write or overwrite editable files with any applied modifications.

    x. Special case where editable files are not written, if '--export'
       option specifies an output file. In this case all data are merged
       into a single place.

Examples:

Keep in mind, no file will be written or overwritten until the option '--write'
or '-w' is in effect. The examples below have a comment starting with '#'
symbol explaining what the command is doing. Shell commands are indicated with
a '$' in the beginning of a line and must be omitted when typing the command in
the terminal. The following lines are output.

    # Read a file and output the content to stdout after default processing.
    $ {APP_NAME} file1.cfg -o
    input_player1_joypad_index = \"1\"
    aspect_ratio_index = \"0\"
    video_threaded = \"true\"
    video_max_swapchain_images = \"2\"

Surrounding quotation marks on values are handled automatically when reading or
writing. If a key is found multiple times with different values in a file, then
it priotizes the top most, just like RetroArch does. But this program will
remove the other instances of the key when saving. Comments are also removed.

    # Read a file and rewrite it to disk after default processing.
    $ {APP_NAME} file1.cfg -w

    # Output value of key 'aspect_ratio_index' without quotes. Note the file
    # stays unchanged on disk.
    $ {APP_NAME} file1.cfg -g aspect_ratio_index
    0

    # Set value of key 'aspect_ratio_index' to '22'. Non existing keys will be
    # added automatically. Use option '-w' for saving the changes back to file.
    $ {APP_NAME} file1.cfg -s aspect_ratio_index 22 -w

    # Let's check value again.
    $ {APP_NAME} file1.cfg -g aspect_ratio_index
    22

    # We can get entire key and value with surrounding quotation marks too.
    $ {APP_NAME} file1.cfg -k aspect_ratio_index
    aspect_ratio_index = \"22\"

    # The search term for '-k' is actually a regular expression. All results
    # from pattern matching the key names will be printed.
    $ {APP_NAME} file1.cfg -k '^video_'
    video_threaded = \"true\"
    video_max_swapchain_images = \"2\"

    # Read data from stdin, which acts like content of an anonymous file.
    # Default process it and output to stdout again. Missing quotation marks
    # around the value will be added automatically.
    $ echo 'video_threaded=false\\nhello = \"world\"' | {APP_NAME} -io
    hello = \"world\"
    video_threaded = \"false\"

You can combine single letter options to type less dashes and spaces. In
example the options '-i -o' becomes '-io'. If an option expects an argument,
then you can type it as '-oe new.cfg' instead of '-o -e new.cfg' in example.

    # Let's load a file, update some keys from stdin stream and output to
    # stdout without writing a file. And sort the output alphabetically.
    $ echo 'video_threaded=false\\nhello = \"world\"' | {APP_NAME} file1.cfg -ioS
    aspect_ratio_index = \"0\"
    hello = \"world\"
    input_player1_joypad_index = \"1\"
    video_max_swapchain_images = \"2\"
    video_threaded = \"false\"

The files specified from option \"--update FILE\" works similar to the
\"--input\" option reading from stdin. These commands will update values and
keys on the fly and apply it to the editable files or output stream.

    # List all keys with 'video' in name and print it together with filenames.
    # Filenames and paths can be output with '-f' to stderr. This is useful for
    # inspecting multiple files. Missing filenames are represented by a single
    # double colon ':'. Also notice the empty line between files in stdout.
    # Option '-c1' adds basic coloring (not visible in this usage tutorial).
    $ {APP_NAME} ~/.config/retroarch/config/*/*.cfg -f -k video -c1

    /home/tuncay/.config/retroarch/config/scummvm/scummvm.cfg:
    video_vsync = \"false\"

    /home/tuncay/.config/retroarch/config/SMS Plus GX/gamegear.cfg:

    /home/tuncay/.config/retroarch/config/Snes9x/Snes9x.cfg:
    video_scale_integer_overscale = \"false\"
    video_scale_integer = \"false\"

    # Use '--list' or '-l' option to list keys without values. This will change
    # the behaviour of the other options '-k' and '-v' to list key names or
    # values on it's own. Let's try same command as before, but with the list
    # option added.
    $ {APP_NAME} ~/.config/retroarch/config/*/*.cfg -f -k video -c1 -l

    /home/tuncay/.config/retroarch/config/scummvm/scummvm.cfg:
    video_vsync

    /home/tuncay/.config/retroarch/config/SMS Plus GX/gamegear.cfg:

    /home/tuncay/.config/retroarch/config/Snes9x/Snes9x.cfg:
    video_scale_integer_overscale
    video_scale_integer

When inserting new keys or reading them, it will always insert to or operate on
the top. RetroArch priotizes first encounter of key too. Have in mind this
program do not preserve comments in the config file.

https://github.com/thingsiplay/{APP_NAME}/"
    )
}

#[allow(clippy::too_many_lines)]
fn main() -> Result<(), Box<dyn Error>> {
    let args: Arguments = Options::parse_args_default_or_exit();

    if args.version {
        println!("{APP_NAME} v{APP_VERSION}");
    }
    if args.show_usage {
        println!("{}", usage_message());
    }

    let mut force_newline: Option<&'static str> = None;
    if args.crlf || (args.lf && args.cr) {
        force_newline = Some("\r\n");
    } else if args.lf {
        force_newline = Some("\n");
    } else if args.cr {
        force_newline = Some("\r");
    }

    // Load all source and update files and stdin data from specified
    // commandline options. Create a dummy file and the export if necessary.
    // All data from these files is then collected for interpretation as
    // RetroArch .cfg config data.
    let mut source_configs: Vec<Config> = load_files(args.file);
    if source_configs.is_empty() {
        source_configs.push(Config::new());
    }
    let mut update_configs: Vec<Config> = load_files(args.update);
    let mut export_config: Config = Config::new();
    if let Some(ref path) = args.export {
        export_config.set_path(path);
    }
    if args.stdin {
        update_configs.push(read_stdin_config());
    }

    // Process all input files, update commands and print if requested.
    for config in &mut source_configs {
        config.style = args.color;

        // Updating commands
        for update in &update_configs {
            config.insert_from_config(update);
        }

        // Editing commands
        if args.sort && args.export.is_none() {
            config.sort();
        }
        for (key, value) in &args.set {
            config.set(key, value);
        }
        for (key, search, replace) in &args.replace {
            config.replace(key, search, replace);
        }
        for key in &args.delete {
            config.remove(key);
        }

        // Viewing commands
        if args.filenames {
            println!();
            eprintln!("{}:", config.path_to_string());
        }
        for key in &args.get {
            if let Some(value) = config.get(key) {
                config.print_value(&value);
            };
        }

        // The following segment has 2 purposes: If key and value are given at
        // the same time, then they are combined to act as a single search
        // where both have to match at the same time. The other purpose is if
        // a list option is given, then all output are only list key without
        // value or value without key. That's why it is a bit convoluted here.
        //
        // Mode: Key and Value +- List
        // if args.key.is_some() && args.value.is_some() {
        if let (Some(k), Some(v)) = (&args.key, &args.value) {
            for (key, value) in config.find(k, v) {
                if args.list {
                    config.print_key(&key);
                    config.print_value(&value);
                } else {
                    config.print_pair(&key, &value);
                }
            }
        // Mode: List only
        } else if args.list && args.key.is_none() && args.value.is_none() {
            config.print_keys_list();
        // Mode: Key or Value +- List
        } else {
            if let Some(k_pattern) = &args.key {
                for (key, value) in config.find_by_key(k_pattern) {
                    if args.list {
                        config.print_key(&key);
                    } else {
                        config.print_pair(&key, &value);
                    }
                }
            }
            if let Some(v_pattern) = &args.value {
                for (key, value) in config.find_by_value(v_pattern) {
                    if args.list {
                        config.print_value(&value);
                    } else {
                        config.print_pair(&key, &value);
                    }
                }
            }
        }

        // Writing commands
        // Overwrite files with '--write' only if no '--export' option is set.
        if args.export.is_some() {
            export_config.insert_from_config(config);
        } else {
            if args.output {
                println!("{}", config);
            }
            if args.write {
                if let Some(newline) = force_newline {
                    config.lineending = newline;
                }
                config.write()?;
            }
        }
    }

    // The actual viewing and writing commands for export file is done after
    // the main processing loop above. Because it must be done after all files
    // are processed.
    if args.export.is_some() {
        if args.filenames {
            println!();
            eprintln!("{}:", export_config.path_to_string());
        }
        if args.sort {
            export_config.sort();
        }
        if args.output {
            println!("{}", export_config);
        }
        if args.write {
            if let Some(newline) = force_newline {
                export_config.lineending = newline;
            }
            export_config.write()?;
        }
    }

    Ok(())
}
