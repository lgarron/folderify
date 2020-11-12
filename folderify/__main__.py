#!/usr/bin/env python

from __future__ import print_function

import argparse
import functools
import os.path
import platform
import shutil
import subprocess
import sys
import tempfile

from string import Template


################################################################


# There are clever ways to do recursive flattening, but this works just fine.
def p(*args):
    group = []
    for arg in args:
        if isinstance(arg, list):
            for entry in arg:
                group.append(entry)
        else:
            group.append(arg)
    return group

# There are clever ways to do recursive flattening, but this works just fine.
def g(*args):
    return ["("] + p(*args) + [")"]

def main():

    DEFAULT_CACHE_DIR = os.path.expanduser("~/.folderify/cache")
    LOCAL_OSX_VERSION = ".".join(platform.mac_ver()[0].split(".")[:2])

    parser = argparse.ArgumentParser(
        description="Generate a native OSX folder icon from a mask file.",
        formatter_class=argparse.RawTextHelpFormatter
    )

    exclusive = parser.add_mutually_exclusive_group()

    exclusive.add_argument(
        "mask",
        action="store",
        nargs="?",
        type=str,
        help="Mask image file. For best results:\n\
- Use a .png mask.\n\
- Use a solid black design over a transparent background.\n\
- Make sure the corner pixels of the mask image are transparent. They are used for empty margins.\n\
- Make sure the icon is at least around 1024x1024, in order to look good at maximum Retina resolution.")

    parser.add_argument(
        "target",
        action="store",
        nargs="?",
        type=str,
        help="Target file or folder. \
If a target is specified, the resulting icon will be applied to the target file/folder. \
Else, a .iconset folder and .icns file will be created in the same folder as mask \
(you can use \"Get Info\" in Finder to copy the icon from the .icns file).")

    parser.add_argument(
        "--reveal", "-r",
        action="store_true",
        help="Reveal the target (or resulting .icns file) in Finder.")

    parser.add_argument(
        "--osx", "-x",
        type=str,
        metavar="VERSION",
        default=LOCAL_OSX_VERSION,
        help=("Version of the OSX folder icon, e.g. \"10.13\". Pass in \"10.9\" to get the pre-Yosemite style. \
Defaults to the version this computer is running (%s)." % LOCAL_OSX_VERSION))

    parser.add_argument(
        "--cache", "-c",
        action="store_true",
        help="Cache the mask icon in the cache dir.")

    parser.add_argument(
        "--cache-dir",
        type=str,
        metavar="DIR",
        default=DEFAULT_CACHE_DIR,
        help="Use the specified cache directory (default: %s)." % DEFAULT_CACHE_DIR)

    exclusive.add_argument(
        "--cache-list",
        action="store_true",
        help="List all paths with cached masks.")

    exclusive.add_argument(
        "--cache-restore",
        metavar="PATH",
        type=str,
        help="Restore folderified icon to the file/folder at PATH, \
using the mask image in the cache for that path.")

    exclusive.add_argument(
        "--cache-restore-all",
        action="store_true",
        help="Restore all paths that have been cached.")

    exclusive.add_argument(
        "--cache-remove",
        metavar="PATH",
        type=str,
        help="Remove the cached mask for the file/folder at PATH.")

    parser.add_argument(
        "--verbose", "-v",
        action="store_true",
        help="Detailed output.")

    ################################################################

    def cache_path_for_target(target):
        return args.cache_dir + os.path.abspath(target) + ".mask"

    ################################################################

    args = parser.parse_args()

    if args.cache and not (args.mask and args.target):
        parser.error("Must specify mask and target in order to use --cache.")

    if args.mask and not os.path.exists(args.mask):
        parser.error("Mask file does not exist: %s" % args.mask)

    if args.target and not os.path.exists(args.target):
        parser.error("Target file/folder does not exist: %s" % args.target)

    if args.cache_restore and not os.path.exists(args.cache_restore):
        parser.error(
            "File/folder does not exist (so the icon cannot be restored): %s" % args.cache_restore)
    if args.cache_restore and not os.path.exists(cache_path_for_target(args.cache_restore)):
        parser.error(
            "File/folder is not in cache (so the icon cannot be restored): %s" % args.cache_restore)

    if args.cache_remove and not os.path.exists(cache_path_for_target(args.cache_remove)):
        parser.error(
            "File/folder is not in cache (and cannot be removed): %s" % args.cache_remove)

    ################################################################

    data_folder = os.path.dirname(sys.modules[__name__].__file__)

    if args.osx in ["10.5", "10.6", "10.7", "10.8", "10.9"]:
        # http://arstechnica.com/apple/2007/10/mac-os-x-10-5/4/
        folder_type = "pre-Yosemite"
    elif args.osx in ["10.10", "10.11", "10.12", "10.13", "10.14", "10.15"]:
        folder_type = "Yosemite"
    else:
        folder_type = "BigSur"
    template_folder = os.path.join(
        data_folder, "GenericFolderIcon.%s.iconset" % folder_type)

    convert_path = "convert"
    iconutil_path = "iconutil"
    seticon_path = os.path.join(data_folder, "lib", "seticon")

    cached_mask_suffix = ".mask"

    ################################################################

    def create_iconset(print_prefix, mask, temp_folder, iconset_folder, params):
        global processes

        name, icon_size, width, height, offset_center = params

        if args.verbose:
            print("[%s] %s" % (print_prefix, name))

        # TEMP_MASK_IMAGE = os.path.join(temp_folder, "trimmed_%s.png" % name)
        TEMP_MASK_IMAGE = "/Users/lgarron/Desktop/folderify/working/TEMP_MASK_IMAGE.png"
        try:
            subprocess.check_call(p(
                convert_path,
                "-background",
                "transparent",
                g(mask,
                    "-trim",
                    "-resize",
                    ("%dx%d" % (width, height)),
                    "-gravity", "Center",
                ),
                "-extent",
                ("%dx%d+0-%d" % (icon_size, icon_size, offset_center)),
                TEMP_MASK_IMAGE
            ))
        except OSError as e:
            print("""ImageMagick command failed.
Make sure you have ImageMagick installed, for example:

  brew install imagemagick

or

  sudo port install ImageMagick
""")
            sys.exit(1)

        FILE_OUT = os.path.join(iconset_folder, "icon_%s.png" % name)
        template_icon = os.path.join(template_folder, "icon_%s.png" % name)

        main_opacity = 15
        offset_white = 2
        opacity_white = 100

        aligned_image = g(TEMP_MASK_IMAGE) #, "-gravity", "Center", "-geometry", ("+0+%d" % offset_center))

        COLORIZED_ICON_IMAGE = "/Users/lgarron/Desktop/folderify/working/1.1.COLORIZED_ICON_IMAGE.png"
        subprocess.Popen(p(
            convert_path,
            "-fill", "rgb(8, 134, 206)",
            TEMP_MASK_IMAGE,
            "-colorize", "100, 100, 100",
            "-channel", "Alpha", "-evaluate", "multiply", "0.5",
            COLORIZED_ICON_IMAGE
        )).wait()

        WHITE_TOP_COLORIZED_IMAGE = "/Users/lgarron/Desktop/folderify/working/2.1.WHITE_TOP_COLORIZED_IMAGE.png"
        subprocess.Popen(p(
            convert_path,
            TEMP_MASK_IMAGE,
            "-fill", "rgb(90, 170, 230)",
            "-colorize", "100, 100, 100",
            WHITE_TOP_COLORIZED_IMAGE
        )).wait()

        WHITE_TOP_BLUR_IMAGE = "/Users/lgarron/Desktop/folderify/working/2.2.WHITE_TOP_BLUR_IMAGE.png"
        subprocess.Popen(p(
            convert_path,
            WHITE_TOP_COLORIZED_IMAGE,
            "-motion-blur", "0x0-90",
            "-page", "+0+1", "-background", "none", "-flatten",
            WHITE_TOP_BLUR_IMAGE
        )).wait()

        WHITE_TOP_MASKED_IMAGE = "/Users/lgarron/Desktop/folderify/working/2.3.WHITE_TOP_MASKED_IMAGE.png"
        subprocess.Popen(p(
            convert_path,
            WHITE_TOP_BLUR_IMAGE,
            TEMP_MASK_IMAGE,
            "-alpha", "Set", "-compose", "Dst_Out", "-composite",
            WHITE_TOP_MASKED_IMAGE
        )).wait()

        WHITE_TOP_SHADOW_IMAGE = "/Users/lgarron/Desktop/folderify/working/2.4.WHITE_TOP_SHADOW_IMAGE.png"
        subprocess.Popen(p(
            convert_path,
            WHITE_TOP_MASKED_IMAGE,
            "-channel", "Alpha", "-evaluate", "multiply", "0.5",
            WHITE_TOP_SHADOW_IMAGE
        )).wait()

        WHITE_BOTTOM_COLORIZED_IMAGE = "/Users/lgarron/Desktop/folderify/working/3.1.WHITE_BOTTOM_COLORIZED_IMAGE.png"
        subprocess.Popen(p(
            convert_path,
            TEMP_MASK_IMAGE,
            "-fill", "rgb(174, 225, 253)",
            "-colorize", "100, 100, 100",
            WHITE_BOTTOM_COLORIZED_IMAGE
        )).wait()

        WHITE_BOTTOM_BLUR_IMAGE = "/Users/lgarron/Desktop/folderify/working/3.2.WHITE_BOTTOM_BLUR_IMAGE.png"
        subprocess.Popen(p(
            convert_path,
            WHITE_BOTTOM_COLORIZED_IMAGE,
            "-motion-blur", "0x2-90",
            "-page", "+0+2", "-background", "none", "-flatten",
            WHITE_BOTTOM_BLUR_IMAGE
        )).wait()

        WHITE_BOTTOM_MASKED_IMAGE = "/Users/lgarron/Desktop/folderify/working/3.3.WHITE_BOTTOM_MASKED_IMAGE.png"
        subprocess.Popen(p(
            convert_path,
            WHITE_BOTTOM_BLUR_IMAGE,
            TEMP_MASK_IMAGE,
            "-alpha", "Set", "-compose", "Dst_Out", "-composite",
            WHITE_BOTTOM_MASKED_IMAGE
        )).wait()

        WHITE_BOTTOM_SHADOW_IMAGE = "/Users/lgarron/Desktop/folderify/working/3.4.WHITE_BOTTOM_SHADOW_IMAGE.png"
        subprocess.Popen(p(
            convert_path,
            WHITE_BOTTOM_MASKED_IMAGE,
            "-channel", "Alpha", "-evaluate", "multiply", "0.5",
            WHITE_BOTTOM_SHADOW_IMAGE
        )).wait()

        COLORIZED_BLACK_IMAGE = "/Users/lgarron/Desktop/folderify/working/4.1.COLORIZED_BLACK_IMAGE.png"
        subprocess.Popen(p(
            convert_path,
            TEMP_MASK_IMAGE,
            "-fill", "rgb(58, 152, 208)",
            "-colorize", "100, 100, 100",
            "-channel", "Alpha", "-negate",
            COLORIZED_BLACK_IMAGE
        )).wait()

        BLACK_BLUR_IMAGE = "/Users/lgarron/Desktop/folderify/working/4.2.BLACK_BLUR_IMAGE.png"
        subprocess.Popen(p(
            convert_path,
            COLORIZED_BLACK_IMAGE,
            "-motion-blur", "0x0-90",
            "-page", "+0+2", "-background", "none", "-flatten",
            BLACK_BLUR_IMAGE
        )).wait()

        BLACK_SHADOW_IMAGE = "/Users/lgarron/Desktop/folderify/working/4.3.BLACK_SHADOW_IMAGE.png"
        subprocess.Popen(p(
            convert_path,
            "-compose", "dst-in",
            BLACK_BLUR_IMAGE,
            TEMP_MASK_IMAGE,
            "-composite",
            "-channel", "Alpha", "-evaluate", "multiply", "0.5",
            BLACK_SHADOW_IMAGE
        )).wait()

        # Here comes the magic.
        # TODO: rewrite in Python.
        COMPOSITE_IMAGE = "/Users/lgarron/Desktop/folderify/working/5.1.COMPOSITE_IMAGE.png"
        subprocess.Popen(p(
            convert_path,
            template_icon,
            COLORIZED_ICON_IMAGE,
            "-composite",
            WHITE_TOP_SHADOW_IMAGE,
            "-composite",
            WHITE_BOTTOM_SHADOW_IMAGE,
            "-composite",
            BLACK_SHADOW_IMAGE,
            "-composite",
            COMPOSITE_IMAGE
        )).wait()

        return subprocess.Popen(["ls"])

    ################################################################

    def create_and_set_icns(mask, target=None, add_to_cache=False, is_from_cache=False):

        temp_folder = tempfile.mkdtemp()

        if target:
            iconset_folder = os.path.join(temp_folder, "iconset.iconset")
            icns_file = os.path.join(temp_folder, "icns.icns")

            os.mkdir(iconset_folder)

            print_prefix = target
            if is_from_cache:
                print("[%s] Restoring from cache." % (print_prefix))
            else:
                print("[%s] <= [%s]" % (print_prefix, mask))
        else:
            root, _ = os.path.splitext(mask)
            iconset_folder = root + ".iconset"
            icns_file = root + ".icns"

            if not os.path.exists(iconset_folder):
                os.mkdir(iconset_folder)
            target = icns_file

            print_prefix = mask
            print("[%s] => [%s]" % (print_prefix, iconset_folder))
            print("[%s] => [%s]" % (print_prefix, icns_file))

        if (add_to_cache):
            mask_cache_path = cache_path_for_target(target)
            mask_cache_folder = os.path.dirname(mask_cache_path)
            if not os.path.exists(mask_cache_folder):
                os.makedirs(mask_cache_folder)
            shutil.copyfile(mask, mask_cache_path)
            print("[%s] Storing in cache => [%s]" %
                  (print_prefix, mask_cache_path))

        inputs = {
            # TODO: adjust this
            "BigSur": [
                # ["16x16", 1,      12,   8,  1], ["16x16@2x", 32,    26,  14,  2],
                # ["32x32", 3,      26,  14,  2], ["32x32@2x", 64,    52,  26,  2],

                # ["128x128", 128,   103,  53,  4], ["128x128@2x", 256, 206, 106,  9],
                # ["256x256", 256,   206, 106,  9], ["256x256@2x", 512, 412, 212, 18],
                # ["512x512", 512, 412, 212, 18],
                ["512x512@2x", 1024, 760, 380, 52]
            ],
            # "Yosemite": [
            #     ["16x16",       12,   8,  1], ["16x16@2x",    26,  14,  2],
            #     ["32x32",       26,  14,  2], ["32x32@2x",    52,  26,  2],

            #     ["128x128",    103,  53,  4], ["128x128@2x", 206, 106,  9],
            #     ["256x256",    206, 106,  9], ["256x256@2x", 412, 212, 18],
            #     ["512x512",    412, 212, 18], ["512x512@2x", 824, 424, 36]
            # ],
            # "pre-Yosemite": [
            #     ["16x16",       12,   8,  1], ["16x16@2x",    26,  14,  2],
            #     ["32x32",       26,  14,  2], ["32x32@2x",    52,  30,  4],

            #     ["128x128",    103,  60,  9], ["128x128@2x", 206, 121, 18],
            #     ["256x256",    206, 121, 18], ["256x256@2x", 412, 242, 36],
            #     ["512x512",    412, 242, 36], ["512x512@2x", 824, 484, 72]
            # ]
        }

        f = functools.partial(create_iconset, print_prefix,
                              mask, temp_folder, iconset_folder)
        processes = map(f, inputs[folder_type])

        for process in processes:
            process.wait()

        if args.verbose:
            print("[%s] Creating the .icns file..." % print_prefix)

        subprocess.check_call([
            iconutil_path,
            "--convert", "icns",
            "--output", icns_file,
            iconset_folder
        ])

        # Make sure seticon is executable.
        subprocess.check_call([
            "chmod",
            "+x",
            seticon_path
        ])

        if args.verbose:
            print("[%s] Setting icon for target." % (print_prefix))
        # Set icon  for target.
        subprocess.check_call([
            seticon_path,
            icns_file,
            target
        ])

        # Clean up.
        # shutil.rmtree(temp_folder)
        print(temp_folder)

        # Reveal target.
        if args.reveal:
            if args.verbose:
                print("[%s] Revealing target." % (print_prefix))
            subprocess.check_call([
                "open",
                "-R", target
            ])

        if args.verbose:
            print("[%s] Done." % (print_prefix))

    def target_for_cache_path(cache_path):
        assert(cache_path.endswith(cached_mask_suffix))
        intermediate = cache_path[:-len(cached_mask_suffix)]
        return os.path.join("/", os.path.relpath(intermediate, args.cache_dir))

    def restore_from_cache(target):
        mask_cache_path = cache_path_for_target(target)
        if args.verbose:
            print("[%s] Mask path from cache: %s" % (target, mask_cache_path))
        create_and_set_icns(
            mask_cache_path,
            target=target,
            add_to_cache=False,
            is_from_cache=True
        )

    def cached_targets():
        for folder, _, files in os.walk(args.cache_dir):
            for f in files:
                if f.endswith(cached_mask_suffix):
                    cache_path = os.path.join(folder, f)
                    yield target_for_cache_path(cache_path)

    if args.mask:
        if args.cache:
            assert(args.target)
        create_and_set_icns(
            args.mask,
            target=args.target,
            add_to_cache=args.cache,
            is_from_cache=False
        )

    elif args.cache_restore:
        restore_from_cache(args.cache_restore)

    elif args.cache_remove:
        os.remove(cache_path_for_target(args.cache_remove))

    elif args.cache_restore_all:
        for target in cached_targets():
            if os.path.exists(target):
                restore_from_cache(target)
            else:
                print("[%s] Target no longer exists. Skipping." % target)

    elif args.cache_list:
        for target in cached_targets():
            print(target)

    else:
        parser.print_help()


__main__ = main

if __name__ == "__main__":
    main()
