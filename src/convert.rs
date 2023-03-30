use std::cmp::max;
use std::fmt;
use std::fmt::Display;
use std::process::Command;
use std::process::Stdio;
use std::str::from_utf8;

use crate::error::CommandFailedError;
use crate::error::CommandInvalidError;
use crate::error::FolderifyError;
use crate::error::GeneralError;
use crate::options;

const CONVERT_COMMAND: &str = "convert";
const IDENTIFY_COMMAND: &str = "identify";
const DEFAULT_DENSITY: u32 = 72;

pub fn run_command(command_name: &str, args: Vec<&str>) -> Result<Vec<u8>, FolderifyError> {
    let cmd = Command::new(command_name)
        .args(args)
        .stdin(Stdio::piped())
        .output();

    let cmd = match cmd {
        Ok(cmd) => cmd,
        Err(_) => {
            return Err(FolderifyError::CommandInvalid(CommandInvalidError {
                command_name: command_name.into(),
            }));
        }
    };

    if !cmd.status.success() {
        return Err(FolderifyError::CommandFailed(CommandFailedError {
            command_name: command_name.into(),
            stderr: cmd.stderr,
        }));
    }

    Ok(cmd.stdout)
}

pub fn convert_to_stdout(mut args: Vec<&str>) -> Result<Vec<u8>, FolderifyError> {
    // TODO: test if the `convert` command exists
    args.push("png:-");
    println!("{:?}", args.clone().join(" "));
    run_command(CONVERT_COMMAND, args)
}

pub fn identify_read_u32(args: Vec<&str>) -> Result<u32, FolderifyError> {
    let stdout = run_command(IDENTIFY_COMMAND, args)?;
    let s: &str = match from_utf8(&stdout) {
        Ok(s) => s,
        Err(_) => {
            return Err((GeneralError {
                message: "Could not read input dimensions".into(),
            })
            .into())
        }
    };
    let value = match s.parse::<u32>() {
        Ok(value) => value,
        Err(_) => {
            // TODO
            return Err((GeneralError {
                message: "Could not read input dimensions".into(),
            })
            .into());
        }
    };
    Ok(value)
}

pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

impl Display for Dimensions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

fn density(mask_path: &str, centering_dimensions: &Dimensions) -> Result<u32, FolderifyError> {
    let input_width = identify_read_u32(vec!["-format", "%w", mask_path])?;
    let input_height = identify_read_u32(vec!["-format", "%w", mask_path])?;

    println!("Iput width: {}", input_width);

    Ok(max(
        DEFAULT_DENSITY * centering_dimensions.width / input_width,
        DEFAULT_DENSITY * centering_dimensions.height / input_height,
    ))
}

pub fn full_mask(
    options: &options::Options,
    centering_dimensions: &Dimensions,
) -> Result<Vec<u8>, FolderifyError> {
    let mask_path = options.mask.to_str().expect("Invalid mask path");

    let density_string = density(mask_path, centering_dimensions)?.to_string();
    let mut args = vec![
        "-background",
        "transparent",
        "-density",
        &density_string,
        mask_path,
    ];
    if !options.no_trim {
        args.push("-trim")
    }
    let centering_dimensions_string = centering_dimensions.to_string();
    args.extend([
        "-resize",
        &centering_dimensions_string,
        "-gravity",
        "Center",
        "-extent",
        &centering_dimensions_string,
    ]);

    convert_to_stdout(args)
}
