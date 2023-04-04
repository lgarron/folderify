use clap::{CommandFactory, Parser, ValueEnum};
use clap_complete::generator::generate;
use clap_complete::{Generator, Shell};
use std::io::stdout;
use std::process::exit;
use std::{env::var, fmt::Display, path::PathBuf, process::Command};

/// Generate a native-style macOS folder icon from a mask file.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[clap(name = "value_hints_derive")]
struct FolderifyArgs {
    /// Mask image file. For best results:
    /// - Use a .png mask.
    /// - Use a solid black design over a transparent background.
    /// - Make sure the corner pixels of the mask image are transparent. They are used for empty margins.
    /// - Make sure the non-transparent pixels span a height of 384px, using a 16px grid.
    /// If the height is 384px and the width is a multiple of 128px, each 64x64 tile will exactly align with 1 pixel at the smallest folder size.
    #[clap(verbatim_doc_comment)]
    mask: Option<PathBuf>,

    /// Target file or folder. If a target is specified, the resulting icon will
    /// be applied to the target file/folder. Else (unless --output-icns or
    /// --output-iconset is specified), a .iconset folder and .icns file will be
    /// created in the same folder as the mask (you can use "Get Info" in Finder
    /// to copy the icon from the .icns file).
    #[clap(verbatim_doc_comment)]
    target: Option<PathBuf>,

    /// Write the `.icns` file to the given path.
    /// (Will be written even if a target is also specified.)
    #[clap(verbatim_doc_comment, long)]
    output_icns: Option<PathBuf>,

    /// Write the `.iconset` folder to the given path.
    /// (Will be written even if a target is also specified.)
    #[clap(verbatim_doc_comment, long)]
    output_iconset: Option<PathBuf>,

    /// Reveal the target (or resulting .icns file) in Finder.
    #[clap(short, long)]
    reveal: bool,

    /// Version of the macOS folder icon, e.g. "10.13".
    /// Defaults to the version currently running.
    #[clap(long = "macOS", alias = "osx", short_alias = 'x')]
    mac_os: Option<String>, // TODO: enum, default?

    /// Color scheme — auto matches the current system value.
    #[clap(long, value_enum, default_value_t = ColorSchemeOrAuto::Auto)]
    color_scheme: ColorSchemeOrAuto,

    /// Don't trim margins from the mask.
    /// By default, transparent margins are trimmed from all 4 sides.
    #[clap(long, verbatim_doc_comment)]
    no_trim: bool,

    /// Legacy argument. Now ignored.
    #[clap(long, verbatim_doc_comment)]
    set_icon_using: Option<String>,

    /// Detailed output.
    #[clap(short, long)]
    verbose: bool,

    /// Print completions for the given shell (instead of generating any icons).
    /// These can be loaded/stored permanently (e.g. when using Homebrew), but they can also be sourced directly, e.g.:
    ///
    ///  folderify --completions fish | source # fish
    ///  source <(folderify --completions zsh) # zsh
    #[clap(long, verbatim_doc_comment, id = "SHELL")]
    completions: Option<Shell>,
}

#[derive(ValueEnum, Clone, Debug, PartialEq, Copy)]
pub enum ColorScheme {
    Light,
    Dark,
}

impl Display for ColorScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Light => "light",
                Self::Dark => "dark",
            }
        )
    }
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
enum ColorSchemeOrAuto {
    Auto,
    Light,
    Dark,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum SetIconUsing {
    SetIcon,
    Rez,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
enum SetIconUsingOrAuto {
    Auto,
    SetIcon,
    Rez,
}

#[derive(Debug, Clone)]
pub struct Options {
    pub mask_path: PathBuf,
    pub color_scheme: ColorScheme,
    pub no_trim: bool,
    pub target: Option<PathBuf>,
    pub output_icns: Option<PathBuf>,
    pub output_iconset: Option<PathBuf>,
    pub reveal: bool,
    pub verbose: bool,
    pub debug: bool,
}

fn completions_for_shell(cmd: &mut clap::Command, generator: impl Generator) {
    generate(generator, cmd, "folderify", &mut stdout());
}

pub fn get_options() -> Options {
    let mut command = FolderifyArgs::command();

    let args = FolderifyArgs::parse();
    if let Some(shell) = args.completions {
        completions_for_shell(&mut command, shell);
        exit(0);
    }

    let mask = match args.mask {
        Some(mask) => mask,
        None => {
            command.print_help().unwrap();
            exit(0);
        }
    };

    if args.mac_os.is_some() {
        println!("Warning: macOS version was specified, but this is not supported yet. Defaulting to the latest (Big Sur and later).")
    }
    let debug = var("FOLDERIFY_DEBUG") == Ok("1".into());
    Options {
        mask_path: mask,
        color_scheme: map_color_scheme_auto(args.color_scheme),
        no_trim: args.no_trim,
        target: args.target,
        output_icns: args.output_icns,
        output_iconset: args.output_iconset,
        reveal: args.reveal,
        verbose: args.verbose || debug,
        debug,
    }
}

fn map_color_scheme_auto(color_scheme: ColorSchemeOrAuto) -> ColorScheme {
    match color_scheme {
        ColorSchemeOrAuto::Dark => return ColorScheme::Dark,
        ColorSchemeOrAuto::Light => return ColorScheme::Light,
        ColorSchemeOrAuto::Auto => (),
    };

    match Command::new("/usr/bin/env")
        .args(["defaults", "read", "-g", "AppleInterfaceStyle"])
        .output()
    {
        Ok(val) => {
            if val.stdout == String::from("Dark\n").into_bytes() {
                ColorScheme::Dark
            } else {
                ColorScheme::Light
            }
        }
        Err(_) => {
            println!("Could not compute auto color scheme. Assuming light mode.");
            ColorScheme::Light
        }
    }
}
