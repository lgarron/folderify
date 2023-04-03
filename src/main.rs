use icon_conversion::{IconInputs, IconResolution, WorkingDir};

use crate::primitives::Dimensions;

mod convert;
mod error;
mod icon_conversion;
mod options;
mod primitives;

fn main() {
    let options = options::get_options();

    let working_dir = WorkingDir::new();
    working_dir.open_in_finder().unwrap();

    let icon_conversion = working_dir.icon_conversion("shared");
    let full_mask_path = icon_conversion
        .full_mask(
            &options,
            &Dimensions {
                width: 768,
                height: 384,
            },
        )
        .unwrap();

    for resolution in IconResolution::values() {
        let icon_conversion = working_dir.icon_conversion(&resolution.to_string());
        icon_conversion
            .icon(
                &full_mask_path,
                &IconInputs {
                    color_scheme: options::ColorScheme::Dark,
                    resolution,
                },
            )
            .unwrap();
    }
    working_dir.release();
}
