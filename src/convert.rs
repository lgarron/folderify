use std::io::Write;
use std::process::Command;
use std::process::Stdio;

use crate::options;

const CONVERT_COMMAND: &str = "convert";

pub fn convert(mut args: Vec<&str>) -> Vec<u8> {
    args.push("png:-");
    let cmd = Command::new(CONVERT_COMMAND)
        .args(args)
        .stdin(Stdio::piped())
        .output()
        .expect("Could not convert");

    if !cmd.status.success() {
        print!("Error running convert: ");
        std::io::stdout()
            .write_all(&cmd.stderr)
            .expect("(stderr unavailable)")
    }

    cmd.stdout
}

pub fn sized_mask(options: &options::Options) -> Vec<u8> {
    let centering_width = 768;
    let centering_height = 384;

    // let mut args = [
    //     "-background",
    //     "transparent",
    //     // "-density", ("%d" % density),
    //     options.mask.to_str().expect("Invalid mask path"),
    // ];
    // if (options.no_trim)

    let mut args = vec![
        "-background",
        "transparent",
        // "-density", ("%d" % density),
        options.mask.to_str().expect("Invalid mask path"),
    ];
    if options.no_trim {
        args.push("-trim")
    }
    let dimensions = format!("{}x{}", centering_width, centering_height)
        .as_str()
        .to_owned();
    args.extend([
        "-resize",
        &dimensions,
        "-gravity",
        "Center",
        "-extent",
        &dimensions,
    ]);

    convert(args)
}
