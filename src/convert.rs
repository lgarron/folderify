use std::cmp::max;
use std::fmt;
use std::fmt::Display;
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

pub struct CommandArgs {
    pub args: Vec<String>,
}

impl CommandArgs {
    pub fn background_transparent(&mut self) {
        self.args.push("-background".into());
        self.args.push("transparent".into());
    }

    pub fn path(&mut self, path: &Path) {
        self.args.push(
            path.to_str()
                .expect("Could not set path for command")
                .into(),
        );
    }

    pub fn resize(&mut self, dimensions: &Dimensions) {
        self.args.push("-resize".into());
        self.args.push(dimensions.to_string());
    }

    pub fn extent(&mut self, extent: &Extent) {
        self.args.push("-extent".into());
        self.args.push(extent.to_string());
    }

    pub fn format_width(&mut self) {
        self.args.push("-format".into());
        self.args.push("%w".into());
    }

    pub fn format_height(&mut self) {
        self.args.push("-format".into());
        self.args.push("%h".into());
    }

    pub fn density(&mut self, d: u32) {
        self.args.push("-density".into());
        self.args.push(d.to_string());
    }

    pub fn trim(&mut self) {
        self.args.push("-trim".into());
    }

    pub fn center(&mut self) {
        self.args.push("-gravity".into());
        self.args.push("Center".into());
    }

    fn default() -> Self {
        CommandArgs { args: vec![] }
    }
}

pub fn run_command(command_name: &str, args: &CommandArgs) -> Result<Vec<u8>, FolderifyError> {
    println!("args: {}", args.args.join(" "));
    let child = Command::new(command_name)
        .args(args.args.iter())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn();
    let child = match child {
        Ok(child) => child,
        Err(_) => {
            return Err(FolderifyError::CommandInvalid(CommandInvalidError {
                command_name: command_name.into(),
            }));
        }
    };

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

pub fn run_convert(args: &CommandArgs) -> Result<(), FolderifyError> {
    run_command(CONVERT_COMMAND, args)?;
    Ok(())
}

pub fn identify_read_u32(args: &CommandArgs) -> Result<u32, FolderifyError> {
    let stdout = run_command(IDENTIFY_COMMAND, args)?;
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

#[derive(Clone)]
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

fn density(mask_path: &Path, centering_dimensions: &Dimensions) -> Result<u32, FolderifyError> {
    let mut width_args = CommandArgs::default();
    width_args.format_width();
    width_args.path(mask_path);
    let input_width = identify_read_u32(&width_args)?;

    let mut height_args = CommandArgs::default();
    height_args.format_height();
    height_args.path(mask_path);
    let input_height = identify_read_u32(&height_args)?;

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
    let mut args = CommandArgs::default();
    args.background_transparent();
    args.density(density(&options.mask_path, centering_dimensions)?);
    args.path(&options.mask_path);
    if !options.no_trim {
        args.trim()
    }
    args.resize(centering_dimensions);
    args.center();
    args.extent(&Extent::no_offset(centering_dimensions));
    args.path(output_path);
    run_convert(&args)
}

pub struct Offset {
    pub x: i32,
    pub y: i32,
}

impl Offset {
    pub fn from_y(y: i32) -> Self {
        Offset { x: 0, y }
    }

    fn default() -> Offset {
        Offset { x: 0, y: 0 }
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

impl Extent {
    pub fn no_offset(size: &Dimensions) -> Self {
        Self {
            size: size.clone(),
            offset: Offset::default(),
        }
    }
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
    let mut args = CommandArgs::default();
    args.background_transparent();
    args.path(input_path);
    args.resize(&inputs.mask_dimensions);
    args.center();
    args.extent(&Extent {
        size: Dimensions::square(inputs.icon_size),
        offset: Offset::from_y(inputs.offset_y),
    });
    args.path(output_path);
    run_convert(&args)
}
