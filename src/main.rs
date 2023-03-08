use clap::Parser;

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

    /// Color scheme: auto (match current system), light, dark.
    color_scheme: Option<String>,
}

fn main() {
    let args = Args::parse();

    println!("Target: {}", args.mask.display());
}
