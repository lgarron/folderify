use std::path::{Path, PathBuf};

use mktemp::Temp;

use crate::{
    convert::{density, run_command, run_convert, BlurDown, CommandArgs, CompositingOperation},
    error::FolderifyError,
    options,
    primitives::{Dimensions, Extent, Offset, RGBColor},
};

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
    pub top_bezel: BezelInputs,
    pub bottom_bezel: BezelInputs,
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

        let top_bezel_negated = self.simple_operation(
            sized_mask,
            "3.1_TOP_BEZEL_NEGATED",
            |args: &mut CommandArgs| {
                args.negate();
            },
        )?;

        let top_bezel_colorized = self.simple_operation(
            &top_bezel_negated,
            "3.2_TOP_BEZEL_COLORIZED",
            |args: &mut CommandArgs| {
                args.fill_colorize(&inputs.top_bezel.color);
            },
        )?;

        let top_bezel_blurred = self.simple_operation(
            &top_bezel_colorized,
            "3.3_TOP_BEZEL_BLURRED",
            |args: &mut CommandArgs| {
                args.blur_down(&inputs.top_bezel.blur);
            },
        )?;

        let top_bezel_masked = self.simple_operation(
            &top_bezel_blurred,
            "3.4_TOP_BEZEL_MASKED",
            |args: &mut CommandArgs| {
                args.mask_down(sized_mask, &inputs.top_bezel.mask_operation);
            },
        )?;

        let top_bezel = self.simple_operation(
            &top_bezel_masked,
            "3.5_TOP_BEZEL",
            |args: &mut CommandArgs| {
                args.opacity(inputs.top_bezel.opacity);
            },
        )?;

        let bottom_bezel_colorized = self.simple_operation(
            sized_mask,
            "4.1_BOTTOM_BEZEL_COLORIZED",
            |args: &mut CommandArgs| {
                args.fill_colorize(&inputs.bottom_bezel.color);
            },
        )?;

        let bottom_bezel_blurred = self.simple_operation(
            &bottom_bezel_colorized,
            "4.2_BOTTOM_BEZEL_BLURRED",
            |args: &mut CommandArgs| {
                args.blur_down(&inputs.bottom_bezel.blur);
            },
        )?;

        let bottom_bezel_masked = self.simple_operation(
            &bottom_bezel_blurred,
            "4.3_BOTTOM_BEZEL_MASKED",
            |args: &mut CommandArgs| {
                args.mask_down(sized_mask, &inputs.bottom_bezel.mask_operation);
            },
        )?;

        let bottom_bezel = self.simple_operation(
            &bottom_bezel_masked,
            "4.4_bottom_bezel",
            |args: &mut CommandArgs| {
                args.opacity(inputs.bottom_bezel.opacity);
            },
        )?;

        self.simple_operation(template_icon, "final", |args: &mut CommandArgs| {
            args.path(&bottom_bezel);
            args.composite(&CompositingOperation::dissolve);
            args.path(&fill);
            args.composite(&CompositingOperation::dissolve);
            args.path(&top_bezel);
            args.composite(&CompositingOperation::dissolve);
        })?;
        Ok(())
    }
}
