use std::path::{Path, PathBuf};

use mktemp::Temp;

use crate::{
    convert::{
        density, run_command, run_convert, BlurDown, CommandArgs, CompositingOperation, Dimensions,
        Extent, Offset, RGBColor,
    },
    error::FolderifyError,
    options,
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
