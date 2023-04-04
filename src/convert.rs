use std::cmp::max;
use std::path::Path;

use crate::command::identify_read_u32;
use crate::error::FolderifyError;
use crate::primitives::{Dimensions, Extent, Offset, RGBColor};

const DEFAULT_DENSITY: u32 = 144;

// TODO: Place a version of this in `command.rs`
pub(crate) struct CommandArgs {
    pub args: Vec<String>,
}

impl CommandArgs {
    pub fn new() -> Self {
        CommandArgs { args: vec![] }
    }

    pub fn push_string(&mut self, s: String) {
        self.args.push(s);
    }

    pub fn push(&mut self, s: &str) {
        self.push_string(s.into());
    }

    pub fn push_path(&mut self, path: &Path) {
        self.push(path.to_str().expect("Could not set path for command"));
    }

    pub fn background_transparent(&mut self) {
        self.push("-background");
        self.push("transparent");
    }

    pub fn background_none(&mut self) {
        self.push("-background");
        self.push("transparent");
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
        self.push_path(mask_path);
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

pub(crate) fn density(
    mask_path: &Path,
    centering_dimensions: &Dimensions,
) -> Result<u32, FolderifyError> {
    let mut width_args = CommandArgs::new();
    width_args.format_width();
    width_args.push_path(mask_path);
    let input_width = identify_read_u32(&width_args)?;

    let mut height_args = CommandArgs::new();
    height_args.format_height();
    height_args.push_path(mask_path);
    let input_height = identify_read_u32(&height_args)?;

    Ok(max(
        DEFAULT_DENSITY * centering_dimensions.width / input_width,
        DEFAULT_DENSITY * centering_dimensions.height / input_height,
    ))
}
