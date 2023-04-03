use convert::{Dimensions, ScaledMaskInputs};

use crate::convert::{BlurDown, DarkShadowInputs, IconConversion, IconInputs, RGBColor};

mod convert;
mod error;
mod options;

fn main() {
    let options = options::get_options();

    let icon_conversion = IconConversion::new("256x256".into());
    icon_conversion.open_working_dir().unwrap();

    let full_mask_path = icon_conversion
        .full_mask(
            &options,
            &Dimensions {
                width: 768,
                height: 384,
            },
        )
        .unwrap();
    let sized_mask_path = icon_conversion
        .sized_mask(
            &full_mask_path,
            &ScaledMaskInputs {
                icon_size: 256,
                mask_dimensions: Dimensions {
                    width: 192,
                    height: 96,
                },
                offset_y: -12,
            },
        )
        .unwrap();

    println!("full_mask_path: {}", full_mask_path.display());
    println!("sized_mask_path: {}", sized_mask_path.display());

    icon_conversion
        .icon(
            &sized_mask_path,
            &IconInputs {
                fill_color: RGBColor::new(6, 111, 194), // light: 8, 134, 206),
                dark_shadow: DarkShadowInputs {
                    color: RGBColor::new(58, 152, 208),
                    blur: BlurDown {
                        spread_px: 0,
                        page_y: 2,
                    },
                },
            },
        )
        .unwrap();
    icon_conversion.release_working_dir();
    // std::io::stdout()
    //     .write_all(&scaled_mask)
    //     .expect("Could not write result");
}
