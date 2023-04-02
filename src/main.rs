use std::io::Write;

use convert::{full_mask, scaled_mask, Dimensions, ScaledMaskInputs};

mod convert;
mod error;
mod options;

fn main() {
    let options = options::get_options();

    let centering_dimensions = Dimensions {
        width: 768,
        height: 384,
    };
    let full_mask = full_mask(&options, &centering_dimensions).unwrap();
    let scaled_mask = scaled_mask(
        &full_mask,
        &ScaledMaskInputs {
            icon_size: 256,
            mask_dimensions: Dimensions {
                width: 192,
                height: 96,
            },
            offset_y: 12,
        },
    )
    .unwrap();

    std::io::stdout()
        .write_all(&scaled_mask)
        .expect("Could not write result");
}
