use std::io::Write;

use convert::{full_mask, Dimensions};

mod convert;
mod error;
mod options;

fn main() {
    let options = options::get_options();

    let centering_dimensions = Dimensions {
        width: 768,
        height: 384,
    };
    let common_sized_mask = full_mask(&options, &centering_dimensions).unwrap();
    std::io::stdout()
        .write_all(&common_sized_mask)
        .expect("Could not write result");
}
