use std::thread::{self, JoinHandle};

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

    let shared_icon_conversion = working_dir.icon_conversion("shared");
    let full_mask_path = shared_icon_conversion
        .full_mask(
            &options,
            &Dimensions {
                width: 768,
                height: 384,
            },
        )
        .unwrap();

    let iconset_dir = working_dir.create_iconset_dir(&options).unwrap();
    let icns_path = working_dir.mask_with_extension(&options, "icns");

    let mut handles = Vec::<JoinHandle<()>>::new();
    for resolution in IconResolution::values() {
        let icon_conversion = working_dir.icon_conversion(&resolution.to_string());
        let options = options.clone();
        let full_mask_path = full_mask_path.clone();
        let output_path = iconset_dir.join(format!("icon_{}.png", resolution));
        let handle = thread::spawn(move || {
            icon_conversion
                .icon(
                    &options,
                    &full_mask_path,
                    &output_path,
                    &IconInputs {
                        color_scheme: options::ColorScheme::Dark,
                        resolution,
                    },
                )
                .unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    shared_icon_conversion
        .to_icns(&iconset_dir, &icns_path)
        .unwrap();

    if let Some(target) = options.target {
        shared_icon_conversion
            .assign_icns(&icns_path, &target)
            .unwrap();
    }

    working_dir.release();
}
