use convert::{full_mask, scaled_mask, Dimensions, ScaledMaskInputs};
use mktemp::Temp;

mod convert;
mod error;
mod options;

fn main() {
    let options = options::get_options();

    let working_dir = Temp::new_dir().expect("Couldn't create a temp dir.");
    let mut full_mask_path = working_dir.to_path_buf();
    full_mask_path.push("FULL_MASK.png");
    let mut scaled_mask_path = working_dir.to_path_buf();
    scaled_mask_path.push("SCALED_MASK.png");

    let centering_dimensions = Dimensions {
        width: 768,
        height: 384,
    };
    full_mask(&options, &centering_dimensions, &full_mask_path).unwrap();
    scaled_mask(
        &full_mask_path,
        &ScaledMaskInputs {
            icon_size: 256,
            mask_dimensions: Dimensions {
                width: 192,
                height: 96,
            },
            offset_y: 12,
        },
        &scaled_mask_path,
    )
    .unwrap();

    working_dir.release();
    // std::io::stdout()
    //     .write_all(&scaled_mask)
    //     .expect("Could not write result");
}
