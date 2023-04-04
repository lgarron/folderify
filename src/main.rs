use std::{
    fs::create_dir_all,
    path::PathBuf,
    thread::{self, JoinHandle},
};

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

fn main() {
    let options = options::get_options();

    let mut iconset_dir: Option<PathBuf> = None;
    let mut icns_path: Option<PathBuf> = None;
    match &options.target {
        Some(target) => {
            println!(
                "[{}] => assign to [{}]",
                options.mask_path.display(),
                target.display()
            )
        }
        None => {
            let iconset_dir_value = options.mask_path.with_extension("iconset");
            let icns_path_value = options.mask_path.with_extension("icns");
            println!(
                "[{}] => [{}]",
                options.mask_path.display(),
                iconset_dir_value.display()
            );
            println!(
                "[{}] => [{}]",
                options.mask_path.display(),
                icns_path_value.display()
            );
            iconset_dir = Some(iconset_dir_value);
            icns_path = Some(icns_path_value);
        }
    }
    println!(
        "[{}] Using folder style: BigSur",
        options.mask_path.display()
    );

    let working_dir = WorkingDir::new();
    if options.debug {
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

    let iconset_dir = match iconset_dir {
        Some(iconset_dir) => {
            create_dir_all(&iconset_dir).unwrap(); // TODO
            iconset_dir
        }
        None => working_dir.create_iconset_dir(&options).unwrap(),
    };
    let icns_path = icns_path.unwrap_or_else(|| working_dir.mask_with_extension(&options, "icns"));

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
        .to_icns(&options, &iconset_dir, &icns_path)
        .unwrap();

    match &options.target {
        Some(target) => {
            shared_icon_conversion
                .assign_icns(&options, &icns_path, target)
                .unwrap();
        }
        // TODO: merge calculation with the `Some` path using `reveal_path`.
        None => shared_icon_conversion
            .assign_icns(&options, &icns_path, &icns_path)
            .unwrap(),
    }

    if options.reveal {
        let reveal_path = options.target.unwrap_or(icns_path);
        let mut args = CommandArgs::new();
        args.push("-R");
        args.push_path(&reveal_path);
        run_command(OPEN_COMMAND, &args, None).unwrap();
    }

    if options.debug {
        working_dir.release();
    }
}
