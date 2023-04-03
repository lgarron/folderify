use std::cmp::max;
use std::fmt;
use std::fmt::Display;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::str::from_utf8;

use mktemp::Temp;

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
    fn new() -> Self {
        CommandArgs { args: vec![] }
    }

    fn push_string(&mut self, s: String) {
        self.args.push(s);
    }

    fn push(&mut self, s: &str) {
        self.push_string(s.into());
    }

    pub fn background_transparent(&mut self) {
        self.push("-background");
        self.push("transparent");
    }

    pub fn background_none(&mut self) {
        self.push("-background");
        self.push("transparent");
    }

    pub fn path(&mut self, path: &Path) {
        self.push(path.to_str().expect("Could not set path for command"));
    }

    pub fn resize(&mut self, dimensions: &Dimensions) {
        self.push("-resize");
        self.push(&dimensions.to_string());
    }

    pub fn extent(&mut self, extent: &Extent) {
        self.push("-extent");
        self.push(&extent.to_string());
    }

    pub fn format_width(&mut self) {
        self.push("-format");
        self.push("%w");
    }

    pub fn format_height(&mut self) {
        self.push("-format");
        self.push("%h");
    }

    pub fn density(&mut self, d: u32) {
        self.push("-density");
        self.push(&d.to_string());
    }

    pub fn trim(&mut self) {
        self.push("-trim");
    }

    pub fn center(&mut self) {
        self.push("-gravity");
        self.push("Center");
    }

    pub fn fill_colorize(&mut self, fill_color: &RGBColor) {
        self.push("-fill");
        self.push(&fill_color.to_string());
        self.push("-colorize");
        self.push("100, 100, 100");
    }

    pub fn opacity(&mut self, alpha: f32) {
        self.push("-channel");
        self.push("Alpha");
        self.push("-evaluate");
        self.push("multiply");
        self.push_string(alpha.to_string());
    }

    pub fn negate(&mut self) {
        self.push("-negate");
    }

    pub fn flatten(&mut self) {
        self.push("-flatten");
    }

    pub fn page(&mut self, offset: &Offset) {
        self.push("-page");
        self.push(&offset.to_string());
    }

    pub fn motion_blur_down(&mut self, spread_px: u32) {
        self.push("-motion-blur");
        self.push_string(format!("0x{}-90", spread_px));
    }

    pub fn blur_down(&mut self, blur_down: &BlurDown) {
        self.motion_blur_down(blur_down.spread_px);
        self.page(&Offset {
            x: 0,
            y: blur_down.page_y,
        });
        self.background_none();
        self.flatten();
    }

    // TODO: take `CompositingOperation` instead of `&CompositingOperation`?
    pub fn composite(&mut self, compositing_operation: &CompositingOperation) {
        self.push("-compose");
        self.push(match compositing_operation {
            CompositingOperation::Dst_In => "Dst_In",
            CompositingOperation::Dst_Out => "Dst_Out",
            CompositingOperation::dissolve => "dissolve",
        });
        self.push("-composite");
    }

    pub fn mask_down(&mut self, mask_path: &Path, compositing_operation: &CompositingOperation) {
        self.path(mask_path);
        self.push("-alpha");
        self.push("Set");
        self.composite(compositing_operation);
    }
}

pub struct BlurDown {
    pub spread_px: u32,
    pub page_y: i32,
}

#[allow(non_camel_case_types)] // Match ImageMagick args
pub enum CompositingOperation {
    Dst_In,
    Dst_Out,
    dissolve,
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

pub struct RGBColor {
    r: u8,
    g: u8,
    b: u8,
}

impl RGBColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl Display for RGBColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rgb({}, {}, {})", self.r, self.g, self.b)
    }
}

fn density(mask_path: &Path, centering_dimensions: &Dimensions) -> Result<u32, FolderifyError> {
    let mut width_args = CommandArgs::new();
    width_args.format_width();
    width_args.path(mask_path);
    let input_width = identify_read_u32(&width_args)?;

    let mut height_args = CommandArgs::new();
    height_args.format_height();
    height_args.path(mask_path);
    let input_height = identify_read_u32(&height_args)?;

    Ok(max(
        DEFAULT_DENSITY * centering_dimensions.width / input_width,
        DEFAULT_DENSITY * centering_dimensions.height / input_height,
    ))
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

pub struct BezelInputs {
    pub color: RGBColor,
    pub blur: BlurDown,
    pub mask_operation: CompositingOperation,
    pub opacity: f32,
}

pub struct IconInputs {
    pub fill_color: RGBColor,
    pub dark_bezel: BezelInputs,
    pub light_bezel: BezelInputs,
}

pub struct IconConversion {
    working_dir: Temp,
    resolution_prefix: String,
}

impl IconConversion {
    pub fn new(resolution_prefix: String) -> Self {
        IconConversion {
            working_dir: Temp::new_dir().expect("Couldn't create a temp dir."),
            resolution_prefix,
        }
    }

    pub fn open_working_dir(&self) -> Result<(), FolderifyError> {
        let mut open_args = CommandArgs::new();
        open_args.path(&self.working_dir);
        run_command("open", &open_args)?;
        Ok(())
    }

    pub fn release_working_dir(self) {
        self.working_dir.release(); // TODO
    }

    fn output_path(&self, file_name: &str) -> PathBuf {
        let mut path = self.working_dir.to_path_buf();
        path.push(format!("{}_{}", self.resolution_prefix, file_name));
        path
    }

    pub fn full_mask(
        &self,
        options: &options::Options,
        centering_dimensions: &Dimensions,
    ) -> Result<PathBuf, FolderifyError> {
        let mut args = CommandArgs::new();
        args.background_transparent();
        args.density(density(&options.mask_path, centering_dimensions)?);
        args.path(&options.mask_path);
        if !options.no_trim {
            args.trim()
        }
        args.resize(centering_dimensions);
        args.center();
        args.extent(&Extent::no_offset(centering_dimensions));
        let output_path = self.output_path("0.0_FULL_MASK.png");
        args.path(&output_path);
        run_convert(&args)?;
        Ok(output_path)
    }

    pub fn sized_mask(
        &self,
        input_path: &Path,
        inputs: &ScaledMaskInputs,
    ) -> Result<PathBuf, FolderifyError> {
        let mut args = CommandArgs::new();
        args.background_transparent();
        args.path(input_path);
        args.resize(&inputs.mask_dimensions);
        args.center();
        args.extent(&Extent {
            size: Dimensions::square(inputs.icon_size),
            offset: Offset::from_y(inputs.offset_y),
        });
        let output_path = self.output_path("1.0_SIZED_MASK.png");
        args.path(&output_path);
        run_convert(&args)?;
        Ok(output_path)
    }

    fn simple_operation(
        &self,
        input_path: &Path,
        output_filename: &str,
        f: impl Fn(&mut CommandArgs),
    ) -> Result<PathBuf, FolderifyError> {
        let mut args = CommandArgs::new();
        args.path(input_path);
        f(&mut args);
        let file_name = format!("{}.png", output_filename);
        let output_path = self.output_path(&file_name);
        args.path(&output_path);
        run_convert(&args)?;
        Ok(output_path)
    }

    pub fn icon(
        &self,
        sized_mask: &Path,
        template_icon: &Path,
        inputs: &IconInputs,
    ) -> Result<(), FolderifyError> {
        let fill_colorized = self.simple_operation(
            sized_mask,
            "2.1_FILL_COLORIZED",
            |args: &mut CommandArgs| {
                args.fill_colorize(&inputs.fill_color);
            },
        )?;
        let fill =
            self.simple_operation(&fill_colorized, "2.2_FILL", |args: &mut CommandArgs| {
                args.opacity(0.5);
            })?;

        let dark_negated =
            self.simple_operation(sized_mask, "3.1_DARK_NEGATED", |args: &mut CommandArgs| {
                args.negate();
            })?;

        let dark_colorized = self.simple_operation(
            &dark_negated,
            "3.2_DARK_COLORIZED",
            |args: &mut CommandArgs| {
                args.fill_colorize(&inputs.dark_bezel.color);
            },
        )?;

        let dark_blurred = self.simple_operation(
            &dark_colorized,
            "3.3_DARK_BLURRED",
            |args: &mut CommandArgs| {
                args.blur_down(&inputs.dark_bezel.blur);
            },
        )?;

        let dark_masked = self.simple_operation(
            &dark_blurred,
            "3.4_DARK_MASKED",
            |args: &mut CommandArgs| {
                args.mask_down(sized_mask, &inputs.dark_bezel.mask_operation);
            },
        )?;

        let dark_bezel =
            self.simple_operation(&dark_masked, "3.5_DARK_BEZEL", |args: &mut CommandArgs| {
                args.opacity(inputs.dark_bezel.opacity);
            })?;

        let light_colorized = self.simple_operation(
            sized_mask,
            "4.1_LIGHT_COLORIZED",
            |args: &mut CommandArgs| {
                args.fill_colorize(&inputs.light_bezel.color);
            },
        )?;

        let light_blurred = self.simple_operation(
            &light_colorized,
            "4.2_LIGHT_BLURRED",
            |args: &mut CommandArgs| {
                args.blur_down(&inputs.light_bezel.blur);
            },
        )?;

        let light_masked = self.simple_operation(
            &light_blurred,
            "4.3_LIGHT_MASKED",
            |args: &mut CommandArgs| {
                args.mask_down(sized_mask, &inputs.light_bezel.mask_operation);
            },
        )?;

        let light_bezel = self.simple_operation(
            &light_masked,
            "4.4_LIGHT_BEZEL",
            |args: &mut CommandArgs| {
                args.opacity(inputs.light_bezel.opacity);
            },
        )?;

        self.simple_operation(template_icon, "final", |args: &mut CommandArgs| {
            args.path(&light_bezel);
            args.composite(&CompositingOperation::dissolve);
            args.path(&fill);
            args.composite(&CompositingOperation::dissolve);
            args.path(&dark_bezel);
            args.composite(&CompositingOperation::dissolve);
        })?;
        Ok(())
    }
}
