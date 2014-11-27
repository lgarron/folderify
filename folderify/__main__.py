#!/usr/bin/env python

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


def main():

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

  local_osx_version = ".".join(platform.mac_ver()[0].split(".")[:2])

  parser.add_argument(
    "--osx_version", "-x",
    type=str,
    default=local_osx_version,
    help=("Version of the OSX folder icon, e.g. \"10.9\" or \"10.10\". \
  Defaults to the version this computer is running (%s)." % local_osx_version))

  parser.add_argument(
    "--cache", "-c",
    action="store_true",
    help="Cache the mask icon in the cache dir.")

  parser.add_argument(
    "--cache-dir",
    type=str,
    default=os.path.expanduser("~/.folderify/cache"),
    help="Cache directory.")

  exclusive.add_argument(
    "--restore-from-cache",
    metavar="PATH",
    type=str,
    help="Restore folderified icon to the file/folder at PATH,\
  using the mask image in the cache for that path.")

  exclusive.add_argument(
    "--restore-all-from-cache",
    action="store_true",
    help="Restore all paths that have been cached.")


  ################################################################


  args = parser.parse_args()

  if args.cache and not args.mask:
    parser.error("Must specify mask in order to use --cache.")


  ################################################################

  data_folder = os.path.dirname(sys.modules[__name__].__file__)

  if args.osx_version == "10.10":
    folder_type = "Yosemite"
  elif args.osx_version in ["10.5", "10.6", "10.7", "10.8", "10.9"]:
    # http://arstechnica.com/apple/2007/10/mac-os-x-10-5/4/
    folder_type = "pre-Yosemite"
  else:
    print "Unknown OSX version(%s). Falling back to 10.10." % s
    folder_type = "Yosemite"
  template_folder = os.path.join(data_folder, "GenericFolderIcon.%s.iconset" % folder_type)

  convert_path = "convert"
  iconutil_path = "iconutil"
  seticon_path = os.path.join(data_folder, "lib", "seticon")

  cached_mask_suffix = ".mask"


  ################################################################


  def create_iconset(mask, temp_folder, iconset_folder, params):
    global processes

    name, width, height, offset_center = params

    print "Generating %s image..." % name

    TEMP_MASK_IMAGE = os.path.join(temp_folder, "trimmed_%s.png" % name)
    subprocess.check_call([
      convert_path,
      mask,
      "-trim",
      "-resize",
      ("%dx%d" % (width, height)),
      "-bordercolor", "none",
      "-border", str(10),
      TEMP_MASK_IMAGE
    ])

    FILE_OUT = os.path.join(iconset_folder, "icon_%s.png" % name)
    template_icon = os.path.join(template_folder, "icon_%s.png" % name)

    main_opacity = 15
    offset_white = 1
    opacity_white = 100

    # Here comes the magic.
    # TODO: rewrite in Python.
    command = [
      convert_path, template_icon, "(",
        "(", TEMP_MASK_IMAGE, "-negate", "-colorize", "3,23,40", "-negate", ")",
        "(",
          "(",
            "(",
              TEMP_MASK_IMAGE,
              "(",
                TEMP_MASK_IMAGE, "-negate", "-shadow", "100x1+10+0", "-geometry", "-2-2",
              ")",
              "-compose", "dst-out", "-composite", "+repage",
            ")",
            "(",
              TEMP_MASK_IMAGE,
              "(",
                TEMP_MASK_IMAGE, "-negate", "-geometry", "+0-1",
              ")",
              "-compose", "dst-out", "-composite", "+repage", "-negate", "-geometry", ("+0+%d" % offset_white),
            ")",
            "-compose", "dissolve", "-define", ("compose:args=%dx50" % opacity_white), "-composite", "+repage",
          ")",
          "(",
            TEMP_MASK_IMAGE,
            "(",
              TEMP_MASK_IMAGE, "-negate", "-geometry", "+0+1",
            ")",
            "-compose", "dst-out", "-composite", "+repage",
          ")",
          "-compose", "dissolve", "-define", "compose:args=50x80", "-composite",
        ")",
        "-compose", "dissolve", "-define", ("compose:args=60x%d" % main_opacity), "-composite", "+repage",
        "-gravity", "Center", "-geometry", ("+0+%d" % offset_center),
        "+repage",
      ")",
      "-compose", "over", "-composite", FILE_OUT
    ]

    return subprocess.Popen(command)


  ################################################################


  def cache_path_for_target(target):
    print args.cache_dir, target, ".mask"
    return args.cache_dir + os.path.abspath(target) + ".mask"

  def process_mask(mask, target=None, add_to_cache=False):
    print ""
    print "Making icon file for %s" % mask
    print "----------------"

    temp_folder = tempfile.mkdtemp()

    if (add_to_cache):
      mask_cache_path = cache_path_for_target(target)
      mask_cache_folder = os.path.dirname(mask_cache_path)
      if not os.path.exists(mask_cache_folder):
        os.makedirs(mask_cache_folder)
      shutil.copyfile(mask, mask_cache_path)

    if target:
      iconset_folder = os.path.join(temp_folder, "iconset.iconset")
      icns_file = os.path.join(temp_folder, "icns.icns")

      os.mkdir(iconset_folder)
    else:
      root, _ = os.path.splitext(mask)
      iconset_folder = root + ".iconset"
      icns_file = root + ".icns"

      if not os.path.exists(iconset_folder):
        os.mkdir(iconset_folder)
      target = icns_file

    # mkdir -p "${ICONSET_FOLDER}"

    inputs = {
      "Yosemite": [
        ["16x16",       12,   8,  1], ["16x16@2x",    26,  14,  2],
        ["32x32",       26,  14,  2], ["32x32@2x",    52,  26,  2],

        ["128x128",    103,  53,  4], ["128x128@2x", 206, 106,  9],
        ["256x256",    206, 106,  9], ["256x256@2x", 412, 212, 18],
        ["512x512",    412, 212, 18], ["512x512@2x", 824, 424, 36]
      ],
      "pre-Yosemite": [
        ["16x16",       12,   8,  1], ["16x16@2x",    26,  14,  2],
        ["32x32",       26,  14,  2], ["32x32@2x",    52,  60,  4],

        ["128x128",    103,  60,  9], ["128x128@2x", 206, 121, 18],
        ["256x256",    206, 121, 18], ["256x256@2x", 412, 242, 36],
        ["512x512",    412, 242, 36], ["512x512@2x", 824, 484, 72]
      ]
    }

    f = functools.partial(create_iconset, mask, temp_folder, iconset_folder)
    processes = map(f, inputs[folder_type])

    for process in processes:
      process.wait()

    print "----------------"
    print "Making the .icns file..."

    subprocess.check_call([
      iconutil_path,
      "--convert", "icns",
      "--output", icns_file,
      iconset_folder
    ])

    # Set icon  for target.
    subprocess.check_call([
      seticon_path,
      "-d", icns_file,
      target
    ])

    # Reveal target.
    if args.reveal:
      subprocess.check_call([
        "open",
        "-R", target
      ])

    # Clean up.
    shutil.rmtree(temp_folder)

    print "----------------"
    print "Done with %s: assigned to %s" % (mask, target)

  def target_for_cache_path(cache_path):
    assert(cache_path.endswith(cached_mask_suffix))
    intermediate = cache_path[:-len(cached_mask_suffix)]
    return os.path.join("/", os.path.relpath(intermediate, args.cache_dir))

  def restore_from_cache(target):
    mask_cache_path = cache_path_for_target(target)
    print "Mask path from cache: %s" % mask_cache_path
    process_mask(
      mask_cache_path,
      target=target,
      add_to_cache=False
    )

  if args.mask:
    if args.cache:
      assert(args.target)
    process_mask(
      args.mask,
      target=args.target,
      add_to_cache=args.cache
    )
  elif (args.restore_from_cache):
    restore_from_cache(args.restore_from_cache)
  elif (args.restore_all_from_cache):
    for folder, _, files in os.walk(args.cache_dir):
      for f in files:
        if f.endswith(cached_mask_suffix):
          cache_path = os.path.join(folder, f)
          target = target_for_cache_path(cache_path)
          if os.path.exists(target):
            restore_from_cache(target)
          else:
            print "Target no longer exists: %s" % target
  else:
    parser.print_help()

__main__ = main

if __name__ == "__main__":
    main()