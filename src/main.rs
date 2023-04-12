use std::thread::{self, JoinHandle};

use command::{run_command, OPEN_COMMAND};
use convert::CommandArgs;
use icon_conversion::{IconInputs, IconResolution, WorkingDir};
use indicatif::MultiProgress;

use crate::{output_paths::PotentialOutputPaths, primitives::Dimensions};

mod command;
mod convert;
mod error;
mod generic_folder_icon;
mod icon_conversion;
mod options;
mod output_paths;
mod primitives;

fn main() {
    let options = options::get_options();

    let potential_output_paths = PotentialOutputPaths::new(&options);

    println!(
        "[{}] Using folder style: BigSur",
        options.mask_path.display()
    );
    println!(
        "[{}] Using color scheme: {}",
        options.mask_path.display(),
        options.color_scheme
    );

    let working_dir = WorkingDir::new();
    if options.debug {
        working_dir.open_in_finder().unwrap();
    }

    let multi_progress_bar = match options.show_progress {
        true => Some(MultiProgress::new()),
        false => None,
    };

    let input_icon_conversion = working_dir.icon_conversion(
        icon_conversion::ProgressBarType::Input,
        "(Input)",
        multi_progress_bar.clone(),
    );
    let full_mask_path = input_icon_conversion
        .full_mask(
            &options,
            &Dimensions {
                width: 768,
                height: 384,
            },
        )
        .unwrap();

    let final_output_paths = potential_output_paths.finalize(&options, &working_dir);

    let mut handles = Vec::<JoinHandle<()>>::new();
    for resolution in IconResolution::values() {
        let icon_conversion = working_dir.icon_conversion(
            icon_conversion::ProgressBarType::Conversion,
            &resolution.to_string(),
            multi_progress_bar.clone(),
        );
        let options = options.clone();
        let full_mask_path = full_mask_path.clone();
        let output_path = final_output_paths.iconset_dir.join(resolution.icon_file());
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

    let output_iconset_only = match (
        &options.target,
        &options.output_icns,
        &options.output_iconset,
    ) {
        (None, None, Some(output_iconset)) => Some(output_iconset),
        _ => None,
    };

    // Deduplicate this `match` with the one that happens after handle joining.
    let output_progress_bar_type = match output_iconset_only {
        Some(_) => icon_conversion::ProgressBarType::OutputWithoutIcns,
        None => icon_conversion::ProgressBarType::OutputWithIcns,
    };
    let output_icon_conversion =
        working_dir.icon_conversion(output_progress_bar_type, "(Output)", multi_progress_bar);
    output_icon_conversion.step_unincremented("Waiting…");

    for handle in handles {
        handle.join().unwrap();
    }

    let reveal_path = match output_iconset_only {
        Some(output_iconset) => {
            // TODO: avoid `.icns assignment entirely?
            // TODO: Change the number of output steps?
            output_iconset
        }
        None => {
            output_icon_conversion
                .to_icns(
                    &options,
                    &final_output_paths.iconset_dir,
                    &final_output_paths.icns_path,
                )
                .unwrap();

            let icns_assignment_path = options
                .target
                .as_ref()
                .unwrap_or(&final_output_paths.icns_path);

            output_icon_conversion
                .assign_icns(
                    &options,
                    &final_output_paths.icns_path,
                    icns_assignment_path,
                )
                .unwrap();

            icns_assignment_path
        }
    };

    if options.reveal {
        match options.show_progress {
            true => output_icon_conversion.step_unincremented("Revealing in Finder…"),
            false => println!("Revealing in Finder…"),
        }
        let mut args = CommandArgs::new();
        args.push("-R");
        args.push_path(reveal_path);
        run_command(OPEN_COMMAND, &args, None).unwrap();
    }

    if options.debug {
        working_dir.release();
    }
}
