use std::{fs::create_dir_all, path::PathBuf};

use crate::{args::Options, icon_conversion::WorkingDir};

pub(crate) struct FinalOutputPaths {
    pub iconset_dir: PathBuf,
    pub icns_path: PathBuf,
}

// TODO: separate printing from calculation
// Or just output everything to a temp path, and copy the desired results.
pub(crate) struct PotentialOutputPaths {
    pub iconset_dir: Option<PathBuf>,
    pub icns_path: Option<PathBuf>,
}

impl PotentialOutputPaths {
    pub fn new(options: &Options) -> PotentialOutputPaths {
        let mut output_paths = PotentialOutputPaths {
            iconset_dir: None,
            icns_path: None,
        };
        match (
            &options.target,
            &options.output_iconset,
            &options.output_icns,
        ) {
            (Some(target), output_iconset, output_icns) => {
                println!(
                    "[{}] => assign to [{}]",
                    options.mask_path.display(),
                    target.display()
                );
                Self::alt_outputs(options, &mut output_paths, output_iconset, output_icns);
            }
            (None, None, None) => {
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
                output_paths.iconset_dir = Some(iconset_dir_value);
                output_paths.icns_path = Some(icns_path_value);
            }
            (None, output_iconset, output_icns) => {
                Self::alt_outputs(options, &mut output_paths, output_iconset, output_icns);
            }
        }
        output_paths
    }

    fn alt_outputs(
        options: &Options,
        output_targets: &mut PotentialOutputPaths,
        output_iconset: &Option<PathBuf>,
        output_icns: &Option<PathBuf>,
    ) {
        if let Some(output_iconset) = output_iconset {
            println!(
                "[{}] => [{}]",
                options.mask_path.display(),
                output_iconset.display()
            );
            output_targets.iconset_dir = Some(output_iconset.to_owned());
        }
        if let Some(output_icns) = output_icns {
            println!(
                "[{}] => [{}]",
                options.mask_path.display(),
                output_icns.display()
            );
            output_targets.icns_path = Some(output_icns.to_owned());
        }
    }

    // This creates the iconset dir if needed (but not the icns path).
    pub fn finalize(&self, options: &Options, working_dir: &WorkingDir) -> FinalOutputPaths {
        let iconset_dir = match &self.iconset_dir {
            Some(iconset_dir) => {
                create_dir_all(iconset_dir).unwrap(); // TODO
                iconset_dir.to_owned()
            }
            None => working_dir.create_iconset_dir(options).unwrap(),
        };

        let icns_path = match &self.icns_path {
            Some(icns_path) => icns_path.to_owned(),
            None => working_dir.icon_file_with_extension("icns"),
        };

        FinalOutputPaths {
            iconset_dir,
            icns_path,
        }
    }
}
