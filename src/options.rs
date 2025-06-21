use clap::{CommandFactory, Parser, ValueEnum};
use clap_complete::generator::generate;
use clap_complete::{Generator, Shell};
use std::io::stdout;
use std::process::exit;
use std::str::from_utf8;
use std::{env::var, fmt::Display, path::PathBuf, process::Command};

use crate::build::CLAP_LONG_VERSION;

/// Generate a native-style macOS folder icon from a mask file.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[clap(name = "folderify", long_version = CLAP_LONG_VERSION )]
struct FolderifyArgs {
    #[allow(clippy::doc_lazy_continuation)] // We want concise text.
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
    #[clap(verbatim_doc_comment, long, id = "ICNS_FILE")]
    output_icns: Option<PathBuf>,

    /// Write the `.iconset` folder to the given path.
    /// (Will be written even if a target is also specified.)
    #[clap(verbatim_doc_comment, long, id = "ICONSET_FOLDER")]
    output_iconset: Option<PathBuf>,

    /// Reveal either the target, `.icns`, or `.iconset` (in that order of preference) in Finder.
    #[clap(short, long)]
    reveal: bool,

    /// Version of the macOS folder icon, e.g. "14.2.1".
    /// Defaults to the version currently running.
    #[clap(long = "macOS", alias = "osx", short_alias = 'x', id = "MACOS_VERSION")]
    mac_os: Option<String>, // TODO: enum, default?

    /// Render the folder as empty.
    #[clap(long, default_value_t = false)]
    empty_folder: bool,

    /// Color scheme — auto matches the current system value.
    #[clap(long, value_enum, default_value_t = ColorSchemeOrAuto::Auto)]
    color_scheme: ColorSchemeOrAuto,

    /// Don't trim margins from the mask.
    /// By default (i.e. without this flag), transparent margins are trimmed from all 4 sides.
    #[clap(long, verbatim_doc_comment)]
    no_trim: bool,

    /// Don't show progress bars.
    #[arg(long)]
    no_progress: bool,

    /// Program used to set the icon. `osascript` should work in most circumstances, `fileicon` performs more checks, and `Rez` produces smaller but less accurate icons.
    #[arg(long, hide(true))]
    set_icon_using: Option<SetIconUsingOrAuto>,

    /// Add a badge to the icon. Currently only supports one badge at a time.
    #[arg(long)]
    badge: Option<Badge>,

    /// Detailed output. Also sets `--no-progress`.
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

#[derive(ValueEnum, Clone, Debug, PartialEq, Copy)]
pub enum Badge {
    Alias,
    Locked,
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
    Fileicon,
    Osascript,
    #[clap(name = "Rez")]
    Rez,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
enum SetIconUsingOrAuto {
    Auto,
    Fileicon,
    Osascript,
    #[clap(name = "Rez")]
    Rez,
}

#[derive(Debug, Clone)]
pub struct Options {
    pub mask_path: PathBuf,
    pub color_scheme: ColorScheme,
    pub no_trim: bool,
    pub target: Option<PathBuf>,
    pub folder_style: FolderStyle,
    pub empty_folder: bool,
    pub output_icns: Option<PathBuf>,
    pub output_iconset: Option<PathBuf>,
    pub set_icon_using: SetIconUsing,
    pub show_progress: bool,
    pub badge: Option<Badge>,
    pub reveal: bool,
    pub verbose: bool,
    pub debug: bool,
}

fn completions_for_shell(cmd: &mut clap::Command, generator: impl Generator) {
    generate(generator, cmd, "folderify", &mut stdout());
}

fn is_major_macos_version_one_of(mac_os: &str, versions: &[&str]) -> bool {
    for major_version_string in versions {
        if mac_os.starts_with(&format!("{}.", major_version_string))
            || mac_os == *major_version_string
        {
            return true;
        }
    }
    false
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FolderStyle {
    BigSur,
    Tahoe,
}

impl Display for FolderStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FolderStyle::BigSur => write!(f, "Big Sur"),
            FolderStyle::Tahoe => write!(f, "Tahoe"),
        }
    }
}

impl FolderStyle {
    pub fn dark_mode_and_light_mode_are_identical(&self) -> bool {
        match self {
            FolderStyle::BigSur => false,
            FolderStyle::Tahoe => true,
        }
    }
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

    let folder_style = {
        let mac_os: String = args
            .mac_os
            .map(|s| s.to_owned())
            .unwrap_or_else(current_macOS_version);
        let mac_os = mac_os.as_str();
        // macOS 11.0 reports itself as macOS 10.16 in some APIs. Someone might pass such a value on to `folderify`, so we can't just check for major version 10.
        // Instead, we denylist the versions that previously had different folder icons, so that we don't accidentally apply the Big Sur style when one of these versions was specified.
        if matches!(
            mac_os,
            "10.5"
                | "10.6"
                | "10.7"
                | "10.8"
                | "10.9"
                | "10.10"
                | "10.11"
                | "10.12"
                | "10.13"
                | "10.14"
                | "10.15"
        ) {
            eprintln!("Error: OS X / macOS 10 was specified. This is no longer supported by folderify v3.\nTo generate these icons, please use folderify v2: https://github.com/lgarron/folderify/tree/main#os-x-macos-10");
            exit(1)
        }
        if is_major_macos_version_one_of(mac_os, &["15", "14", "13", "12", "11"]) {
            FolderStyle::BigSur
        } else if is_major_macos_version_one_of(
            mac_os,
            &["26"], // Note: macOS 16 through 25 do not exist.
        ) {
            eprintln!("Warning: macOS Tahoe is still in beta. The icon may not match the final macOS 26 release.");
            FolderStyle::Tahoe
        } else {
            eprintln!(
                "Warning: Unknown macOS version specified. Assuming Big Sur (macOS 11 through 15)."
            );
            FolderStyle::BigSur
        }
    };
    let debug = var("FOLDERIFY_DEBUG") == Ok("1".into());
    let verbose = args.verbose || debug;
    let show_progress = !args.no_progress && !args.verbose;
    let set_icon_using = match args.set_icon_using {
        Some(SetIconUsingOrAuto::Rez) => SetIconUsing::Rez,
        Some(SetIconUsingOrAuto::Fileicon) => SetIconUsing::Fileicon,
        _ => SetIconUsing::Osascript,
    };
    Options {
        mask_path: mask,
        color_scheme: map_color_scheme_auto(args.color_scheme, folder_style),
        no_trim: args.no_trim,
        target: args.target,
        folder_style,
        empty_folder: args.empty_folder,
        output_icns: args.output_icns,
        output_iconset: args.output_iconset,
        badge: args.badge,
        set_icon_using,
        show_progress,
        reveal: args.reveal,
        verbose,
        debug,
    }
}

fn map_color_scheme_auto(
    color_scheme: ColorSchemeOrAuto,
    folder_style: FolderStyle,
) -> ColorScheme {
    if folder_style.dark_mode_and_light_mode_are_identical() {
        match color_scheme {
            ColorSchemeOrAuto::Auto => {}
            _ => {
                eprintln!("Dark and light mode folder icons are identical for the folder style of this macOS version. Ignoring the `--color-scheme` argument.");
            }
        }
        return ColorScheme::Light;
    }

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

const DEFAULT_MACOS_VERSION: &str = "15.0";

#[allow(non_snake_case)]
fn current_macOS_version() -> String {
    let Ok(o) = Command::new("sw_vers").args(["-productVersion"]).output() else {
        return DEFAULT_MACOS_VERSION.to_owned();
    };

    let Ok(stdout) = from_utf8(&o.stdout) else {
        return DEFAULT_MACOS_VERSION.to_owned();
    };

    stdout.to_owned()
}
