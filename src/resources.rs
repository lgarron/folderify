use std::path::PathBuf;

use include_dir::{include_dir, Dir};

use crate::{
    args::{Badge, ColorScheme, FolderStyle},
    icon_conversion::IconResolution,
};

static RESOURCES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/resources");

pub struct IconInputs {
    pub folder_style: FolderStyle,
    pub color_scheme: ColorScheme,
    pub resolution: IconResolution,
    pub empty_folder: bool,
}

pub fn get_folder_icon(icon_inputs: &IconInputs) -> &'static [u8] {
    let mut path = PathBuf::new();
    path.push("folders");
    path.push(
        match (
            icon_inputs.color_scheme,
            icon_inputs.folder_style,
            icon_inputs.empty_folder,
        ) {
            (ColorScheme::Light, FolderStyle::BigSur, _) => "GenericFolderIcon.BigSur.iconset",
            (ColorScheme::Dark, FolderStyle::BigSur, _) => "GenericFolderIcon.BigSur.dark.iconset",
            (_, FolderStyle::Tahoe, true) => "GenericFolderIcon.empty.Tahoe.iconset",
            (_, FolderStyle::Tahoe, false) => "GenericFolderIcon.non-empty.Tahoe.iconset",
        },
    );
    path.push(icon_inputs.resolution.icon_file());
    RESOURCES_DIR.get_file(&path).unwrap().contents()
}

pub fn get_badge_icon(badge: Badge, resolution: &IconResolution) -> &'static [u8] {
    let mut path = PathBuf::new();
    path.push("badges");
    path.push(match badge {
        Badge::Alias => "AliasBadgeIcon.iconset",
        Badge::Locked => "LockedBadgeIcon.iconset",
    });
    path.push(resolution.icon_file());
    RESOURCES_DIR.get_file(&path).unwrap().contents()
}
