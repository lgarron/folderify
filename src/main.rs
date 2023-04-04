use std::thread::{self, JoinHandle};

use command::{run_command, OPEN_COMMAND};
use convert::CommandArgs;
use icon_conversion::{IconInputs, IconResolution, WorkingDir};

use crate::primitives::Dimensions;

mod command;
mod convert;
mod error;
mod generic_folder_icon;
mod icon_conversion;
mod options;
mod primitives;

const DEBUG: bool = false;

fn main() {
    let options = options::get_options();

    let working_dir = WorkingDir::new();
    if DEBUG {
        working_dir.open_in_finder().unwrap();
    }

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
        let output_path = iconset_dir.join(resolution.icon_file());
        let handle = thread::spawn(move || {
            icon_conversion
                .icon(
                    &options,
                    &full_mask_path,
                    &output_path,
                    &IconInputs {
                        color_scheme: options.color_scheme,
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

    if let Some(target) = &options.target {
        shared_icon_conversion
            .assign_icns(&icns_path, target)
            .unwrap();
    }

    if options.reveal {
        let reveal_path = options.target.unwrap_or(icns_path);
        let mut args = CommandArgs::new();
        args.push("-R");
        args.push_path(&reveal_path);
        run_command(OPEN_COMMAND, &args, None).unwrap();
    }

    if DEBUG {
        working_dir.release();
    }
}
