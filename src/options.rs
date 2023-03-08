use clap::{Parser, ValueEnum};
use std::process::Command;

/// Generate a native-style macOS folder icon from a mask file.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Mask image file. For best results:
    /// - Use a .png mask.
    /// - Use a solid black design over a transparent background.
    /// - Make sure the corner pixels of the mask image are transparent. They are used for empty margins.
    /// - Make sure the non-transparent pixels span a height of 384px, using a 16px grid.
    /// If the height is 384px and the width is a multiple of 128px, each 64x64 tile will exactly align with 1 pixel at the smallest folder size.
    #[clap(verbatim_doc_comment)]
    mask: std::path::PathBuf,
    /// Target file or folder.
    /// If a target is specified, the resulting icon will be applied to the target file/folder.
    /// Else, a .iconset folder and .icns file will be created in the same folder as the mask
    /// (you can use "Get Info" in Finder to copy the icon from the .icns file).
    #[clap(verbatim_doc_comment)]
    target: Option<std::path::PathBuf>,

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

    /// Tool to used to set the icon of the target: auto (default), seticon, Rez.
    /// Rez usually produces a smaller "resource fork" for the icon, but only works if
    /// XCode command line tools are already installed and if you're using a folder target.
    #[clap(long, verbatim_doc_comment)]
    set_icon_using: Option<SetIconUsingOrAuto>, // TODO: accept capitalized Rez

    /// Detailed output.
    #[clap(short, long)]
    verbose: bool,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
enum ColorSchemeOrAuto {
    Auto,
    Light,
    Dark,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
enum SetIconUsingOrAuto {
    Auto,
    SetIcon,
    Rez,
}

// impl SetIconUsingOrAuto {
//     fn value_variants<'a>() -> &'a [Self] {
//         return &[Self::Auto, Self::SetIcon, Self::Rez];
//     }
//     fn to_possible_value(&self) -> Option<PossibleValue> {
//         match self {
//             Self::Auto => Some(PossibleValue::new("auto")),
//             Self::SetIcon => Some(PossibleValue::new("seticon")),
//             Self::Rez => Some(PossibleValue::new("Rez")),
//         }
//     }
//     fn from_str(input: &str, ignore_case: bool) -> Result<Self, String> {
//         match input {
//             "auto" => Ok(Self::Auto),
//             "seticon" => Ok(Self::SetIcon),
//             "Rez" => Ok(Self::Rez),
//             _ => Err("Invalid icon setter".into()),
//         }
//     }

//     // pub fn value_variants<'a>(&self) -> Option<clap::PossibleValue<'a>> {
//     //     match self {
//     //         Auto => clap::PossibleValue::new(Some("auto")),
//     //         SetIcon => clap::PossibleValue::new(Some("seticon")),
//     //         Rez => clap::PossibleValue::new(Some("rez")),
//     //     }
//     // }
// }

#[derive(Debug)]
pub struct Options {
    pub mask: std::path::PathBuf,
    pub dark: bool,
    pub use_rez: bool,
    pub no_trim: bool,
}

pub fn get_options() -> Options {
    let args = Args::parse();
    Options {
        mask: args.mask,
        dark: use_dark_scheme(args.color_scheme),
        use_rez: args.set_icon_using == Some(SetIconUsingOrAuto::Rez),
        no_trim: args.no_trim,
    }
}

fn use_dark_scheme(color_scheme: ColorSchemeOrAuto) -> bool {
    match color_scheme {
        ColorSchemeOrAuto::Dark => return true,
        ColorSchemeOrAuto::Light => return false,
        ColorSchemeOrAuto::Auto => (),
    };

    match Command::new("/usr/bin/env")
        .args(["defaults", "read", "-g", "AppleInterfaceStyle"])
        .output()
    {
        Ok(val) => val.stdout == String::from("Dark\n").into_bytes(),
        Err(_) => {
            println!("Could not compute auto color scheme. Assuming light mode.");
            false
        }
    }
}
