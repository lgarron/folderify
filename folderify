#!/usr/bin/env python

import argparse
import functools
import os.path
import shutil
import subprocess
import sys
import tempfile

from string import Template


################################################################


parser = argparse.ArgumentParser(description="Generate a native OSX folder icon from a mask file.")

parser.add_argument(
  "mask",
  action="store",
  type=str,
  help="Mask image file. Recommendations: use .png with a black shape over a transparent background. \
      Should be at least 1024x1024 to look good at maximum Retina resolution.")

parser.add_argument(
  "target",
  action="store",
  nargs="?",
  type=str,
  help="Target file or folder. If a target is specified, the resulting icon will be applied to the target file/folder. \
    Else, a .iconset folder and .icns file will be created in the same folder as mask.")

parser.add_argument(
  "--reveal", "-r",
  action="store_true",
  help="Reveal the target (or resulting .icns file) in Finder.")

parser.add_argument(
  "--pre-yosemite",
  action="store_true",
  help="Generate pre-Yosemite folder icons.")


################################################################


args = parser.parse_args()


################################################################


data_folder = os.path.dirname(sys.argv[0])

osx_version = "pre-Yosemite" if args.pre_yosemite else "Yosemite"
template_folder = os.path.join(data_folder, "GenericFolderIcon.%s.iconset" % osx_version)

convert_path = "convert"
iconutil_path = "iconutil"
seticon_path = os.path.join(data_folder, "lib", "seticon")

temp_folder = tempfile.mkdtemp()

if args.target:
  iconset_folder = os.path.join(temp_folder, "iconset.iconset")
  icns_file = os.path.join(temp_folder, "icns.icns")

  os.mkdir(iconset_folder)
else:
  root, _ = os.path.splitext(args.mask)
  iconset_folder = root + ".iconset"
  icns_file = root + ".icns"

  if not os.path.exists(iconset_folder):
    os.mkdir(iconset_folder)
  args.target = icns_file


################################################################


def folderify(args, arg_list):
  global processes

  name, width, height, offset_center = arg_list

  print "Generating %s image..." % name

  TEMP_MASK_IMAGE = os.path.join(temp_folder, "trimmed_%s.png" % name)
  subprocess.check_call([
    convert_path,
    args.mask,
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


print ""
print "Making icon file for %s" % args.mask
print "----------------"

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

f = functools.partial(folderify, args)
processes = map(f, inputs[osx_version])

for process in processes:
  process.wait()


################################################################


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
  args.target
])

# Reveal target.
if args.reveal:
  subprocess.check_call([
    "open",
    "-R", args.target
  ])

# Clean up.
shutil.rmtree(temp_folder)


print "----------------"
print "Done."
