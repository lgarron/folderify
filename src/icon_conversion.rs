use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use mktemp::Temp;

const RETINA_SCALE: u32 = 2;

use crate::{
    convert::{density, run_command, run_convert, BlurDown, CommandArgs, CompositingOperation},
    error::FolderifyError,
    options::{self, ColorScheme},
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

pub struct IconBezelInputs {
    pub fill_color: RGBColor,
    pub top_bezel: BezelInputs,
    pub bottom_bezel: BezelInputs,
}

pub struct WorkingDir {
    working_dir: Temp,
}

impl WorkingDir {
    pub fn new() -> Self {
        Self {
            working_dir: Temp::new_dir().expect("Couldn't create a temp dir."),
        }
    }

    pub fn icon_conversion(&self, resolution_prefix: &str) -> IconConversion {
        IconConversion {
            working_dir: &self.working_dir,
            resolution_prefix: resolution_prefix.into(),
        }
    }
    pub fn open_in_finder(&self) -> Result<(), FolderifyError> {
        let mut open_args = CommandArgs::new();
        open_args.path(&self.working_dir);
        run_command("open", &open_args)?;
        Ok(())
    }

    pub fn release(self) {
        self.working_dir.release(); // TODO
    }
}

pub enum IconResolution {
    NonRetina16,
    Retina16,
    NonRetina32,
    Retina32,
    NonRetina128,
    Retina128,
    NonRetina256,
    Retina256,
    NonRetina512,
    Retina512,
}

impl IconResolution {
    // TODO: return iterator?
    pub fn values() -> Vec<IconResolution> {
        vec![
            Self::NonRetina16,
            Self::Retina16,
            Self::NonRetina32,
            Self::Retina32,
            Self::NonRetina128,
            Self::Retina128,
            Self::NonRetina256,
            Self::Retina256,
            Self::NonRetina512,
            Self::Retina512,
        ]
    }

    pub fn size(&self) -> u32 {
        match self {
            IconResolution::NonRetina16 => 16,
            IconResolution::Retina16 => 16 * RETINA_SCALE,
            IconResolution::NonRetina32 => 32,
            IconResolution::Retina32 => 32 * RETINA_SCALE,
            IconResolution::NonRetina128 => 128,
            IconResolution::Retina128 => 128 * RETINA_SCALE,
            IconResolution::NonRetina256 => 256,
            IconResolution::Retina256 => 256 * RETINA_SCALE,
            IconResolution::NonRetina512 => 512,
            IconResolution::Retina512 => 512 * RETINA_SCALE,
        }
    }

    pub fn offset_y(&self) -> i32 {
        match self {
            IconResolution::NonRetina16 => -2,
            IconResolution::Retina16 => -2,
            IconResolution::NonRetina32 => -2,
            IconResolution::Retina32 => -3,
            IconResolution::NonRetina128 => -6,
            IconResolution::Retina128 => -12,
            IconResolution::NonRetina256 => -12,
            IconResolution::Retina256 => -24,
            IconResolution::NonRetina512 => -24,
            IconResolution::Retina512 => -48,
        }
    }

    pub fn bottom_bezel_blur_down(&self) -> BlurDown {
        match self {
            IconResolution::NonRetina16 => BlurDown {
                spread_px: 1,
                page_y: 0,
            },
            _ => BlurDown {
                spread_px: 2,
                page_y: 1,
            },
        }
    }

    pub fn bottom_bezel_alpha(&self) -> f32 {
        match self {
            IconResolution::NonRetina16 => 0.5,
            IconResolution::Retina16 => 0.35,
            IconResolution::NonRetina32 => 0.35,
            IconResolution::Retina32 => 0.6,
            IconResolution::NonRetina128 => 0.6,
            IconResolution::Retina128 => 0.6,
            IconResolution::NonRetina256 => 0.6,
            IconResolution::Retina256 => 0.75,
            IconResolution::NonRetina512 => 0.75,
            IconResolution::Retina512 => 0.75,
        }
    }
}

impl Display for IconResolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::NonRetina16 => "16x16",
                Self::Retina16 => "16x16@2x",
                Self::NonRetina32 => "32x32",
                Self::Retina32 => "32x32@2x",
                Self::NonRetina128 => "128x128",
                Self::Retina128 => "128x128@2x",
                Self::NonRetina256 => "256x256",
                Self::Retina256 => "256x256@2x",
                Self::NonRetina512 => "512x512",
                Self::Retina512 => "512x512@2x",
            }
        )
    }
}

pub struct IconConversion<'a> {
    working_dir: &'a Temp,
    resolution_prefix: String,
}

pub struct IconInputs {
    pub color_scheme: ColorScheme,
    pub resolution: IconResolution,
}

impl IconConversion<'_> {
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

    pub fn add_bezels(
        &self,
        sized_mask: &Path,
        template_icon: &Path,
        inputs: &IconBezelInputs,
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

    // TODO
    pub fn icon(&self, full_mask_path: &Path, inputs: &IconInputs) -> Result<(), FolderifyError> {
        let size = inputs.resolution.size();
        let offset_y = inputs.resolution.offset_y();

        let sized_mask_path = self
            .sized_mask(
                full_mask_path,
                &ScaledMaskInputs {
                    icon_size: size,
                    mask_dimensions: Dimensions {
                        width: size * 3 / 4,
                        height: size / 2,
                    },
                    offset_y,
                },
            )
            .unwrap();

        // TODO
        let template_icon = PathBuf::from(
            format!("/Users/lgarron/Code/git/github.com/lgarron/folderify/old/folderify/GenericFolderIcon.BigSur.iconset/icon_{}.png", inputs.resolution),
        );

        let fill_color = match inputs.color_scheme {
            ColorScheme::Light => RGBColor::new(8, 134, 206),
            ColorScheme::Dark => RGBColor::new(6, 111, 194),
        };

        self.add_bezels(
            &sized_mask_path,
            &template_icon,
            &IconBezelInputs {
                fill_color,
                top_bezel: BezelInputs {
                    color: RGBColor::new(58, 152, 208),
                    blur: BlurDown {
                        spread_px: 0,
                        page_y: 2,
                    },
                    mask_operation: CompositingOperation::Dst_In,
                    opacity: 0.5,
                },
                bottom_bezel: BezelInputs {
                    color: RGBColor::new(174, 225, 253),
                    blur: inputs.resolution.bottom_bezel_blur_down(),
                    mask_operation: CompositingOperation::Dst_Out,
                    opacity: inputs.resolution.bottom_bezel_alpha(),
                },
            },
        )
    }
}
