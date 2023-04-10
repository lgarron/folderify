use std::thread::{self, JoinHandle};

use command::{run_command, OPEN_COMMAND};
use convert::CommandArgs;
use icon_conversion::{IconInputs, IconResolution, WorkingDir};
use indicatif::{MultiProgress, ProgressBar};

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

    let shared_icon_conversion = working_dir.icon_conversion("shared", None);
    let full_mask_path = shared_icon_conversion
        .full_mask(
            &options,
            &Dimensions {
                width: 768,
                height: 384,
            },
        )
        .unwrap();

    let final_output_paths = potential_output_paths.finalize(&options, &working_dir);

    let multi_progress_bar = match options.show_progress {
        true => Some(MultiProgress::new()),
        false => None,
    };

    let mut handles = Vec::<JoinHandle<()>>::new();
    // We keep around references to the progress bars to prevent the fast ones from disappearing super quickly.
    let mut _progress_bars = Vec::<Option<ProgressBar>>::new();
    for resolution in IconResolution::values() {
        let icon_conversion =
            working_dir.icon_conversion(&resolution.to_string(), multi_progress_bar.clone());
        _progress_bars.push(icon_conversion.progress_bar.clone());
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

    for handle in handles {
        handle.join().unwrap();
    }

    let reveal_path = match (
        &options.target,
        &options.output_icns,
        &options.output_iconset,
    ) {
        (None, None, Some(output_iconset)) => output_iconset, // TODO: avoid `.icns assignment entirely?
        _ => {
            shared_icon_conversion
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

            shared_icon_conversion
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
        let mut args = CommandArgs::new();
        args.push("-R");
        args.push_path(reveal_path);
        run_command(OPEN_COMMAND, &args, None).unwrap();
    }

    if options.debug {
        working_dir.release();
    }
}
