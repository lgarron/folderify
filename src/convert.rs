use std::cmp::max;
use std::fmt;
use std::fmt::Display;
use std::io::Write;
use std::path::Path;
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

pub fn run_command(
    command_name: &str,
    args: Vec<&str>,
    stdin_buf: Option<&Vec<u8>>,
) -> Result<Vec<u8>, FolderifyError> {
    println!(
        "\n\n\n<<<<<<<<<\n{} {}\n>>>>>>>>\n\n\n",
        command_name,
        args.join(" ")
    );
    let child = Command::new(command_name)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn();
    let mut child = match child {
        Ok(child) => child,
        Err(_) => {
            return Err(FolderifyError::CommandInvalid(CommandInvalidError {
                command_name: command_name.into(),
            }));
        }
    };

    // if let Some(stdin_buf) = stdin_buf {
    let stdin = match child.stdin.as_mut() {
        Some(stdin) => stdin,
        None => {
            return Err(FolderifyError::CommandInvalid(CommandInvalidError {
                command_name: command_name.into(),
            }));
        }
    };
    let empty_buf = Vec::<u8>::new();
    let buf = stdin_buf.unwrap_or(&empty_buf);
    match stdin.write(buf) {
        Ok(_) => (),
        Err(_) => {
            return Err(FolderifyError::CommandInvalid(CommandInvalidError {
                command_name: command_name.into(),
            }));
        }
    };
    // }

    let output = match child.wait_with_output() {
        Ok(output) => output,
        Err(_) => {
            return Err(FolderifyError::CommandInvalid(CommandInvalidError {
                command_name: command_name.into(),
            }))
        }
    };

    if !output.status.success() {
        return Err(FolderifyError::CommandFailed(CommandFailedError {
            command_name: command_name.into(),
            stderr: output.stderr,
        }));
    }

    Ok(output.stdout)
}

pub fn convert_to_stdout(
    args: Vec<&str>,
    stdin_buf: Option<&Vec<u8>>,
) -> Result<Vec<u8>, FolderifyError> {
    // TODO: test if the `convert` command exists
    // args.push("png:-");
    println!("{:?}", args.clone().join(" "));
    run_command(CONVERT_COMMAND, args, stdin_buf)
}

pub fn identify_read_u32(args: Vec<&str>) -> Result<u32, FolderifyError> {
    let stdout = run_command(IDENTIFY_COMMAND, args, None)?;
    let s: &str = match from_utf8(&stdout) {
        Ok(s) => s,
        Err(s) => {
            println!("errerrerr{}+++++", s);
            return Err((GeneralError {
                message: "Could not read input dimensions".into(),
            })
            .into());
        }
    };
    let value = match s.parse::<u32>() {
        Ok(value) => value,
        Err(s) => {
            // TODO
            println!("errerrerr{}+++++", s);
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

impl Dimensions {
    pub fn square(side_size: u32) -> Self {
        Dimensions {
            width: side_size,
            height: side_size,
        }
    }
}

impl Display for Dimensions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

fn density(mask_path: &str, centering_dimensions: &Dimensions) -> Result<u32, FolderifyError> {
    let input_width = identify_read_u32(vec!["-format", "%w", mask_path])?;
    let input_height = identify_read_u32(vec!["-format", "%w", mask_path])?;
    Ok(max(
        DEFAULT_DENSITY * centering_dimensions.width / input_width,
        DEFAULT_DENSITY * centering_dimensions.height / input_height,
    ))
}

pub fn full_mask(
    options: &options::Options,
    centering_dimensions: &Dimensions,
    output_path: &Path,
) -> Result<(), FolderifyError> {
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
        output_path.to_str().expect("Invalid temp path name???"),
    ]);

    convert_to_stdout(args, None)?;
    Ok(())
}

pub struct Offset {
    pub x: i32,
    pub y: i32,
}

impl Offset {
    pub fn from_y(y: i32) -> Self {
        Offset { x: 0, y }
    }
}

fn sign(v: i32) -> char {
    if v < 0 {
        '-'
    } else {
        '+'
    }
}

impl Display for Offset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}{}",
            sign(self.x),
            self.x.abs(),
            sign(self.y),
            self.y.abs()
        )
    }
}

pub struct Extent {
    pub size: Dimensions,
    pub offset: Offset,
}

impl Display for Extent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.size, self.offset)
    }
}

pub struct ScaledMaskInputs {
    pub icon_size: u32,
    pub mask_dimensions: Dimensions,
    pub offset_y: i32,
}

pub fn scaled_mask(
    input_path: &Path,
    inputs: &ScaledMaskInputs,
    output_path: &Path,
) -> Result<(), FolderifyError> {
    let extent = Extent {
        size: Dimensions::square(inputs.icon_size),
        offset: Offset::from_y(inputs.offset_y),
    };
    convert_to_stdout(
        vec![
            //
            "-background",
            "transparent",
            input_path.to_str().expect("Invalid temp path name???"),
            //
            "-resize",
            &inputs.mask_dimensions.to_string(),
            //
            "-gravity",
            "Center",
            //
            "-extent",
            &extent.to_string(),
            output_path.to_str().expect("Invalid temp path name???"),
        ],
        None,
    )?;
    Ok(())
}
