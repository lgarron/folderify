#!/usr/bin/env python

import argparse
import functools
import multiprocessing
import os.path
import shutil
import subprocess
import sys
import tempfile

from string import Template


################################################################


parser = argparse.ArgumentParser(description='Generate a native OSX folder icon from a mask file.')

parser.add_argument(
  "mask_file",
  action="store",
  type=str,
  help="Mask image file. Recommendations: use .png with a black shape over a transparent background. \
      Should be at least 1024x1024 to look good at maximum Retina resolution.")

parser.add_argument(
  "--target", "-t",
  action="store",
  type=str,
  help="Target file or folder. If a target is specified, the resulting icon will be applied to this file/folder. \
    Else, a .iconset folder and .icns file will be created in the same folder as mask_file.")

templates = ["pre-Yosemite", "Yosemite"]
parser.add_argument(
  "--osx-version", "-x",
  default="Yosemite",
  choices=templates)


try:
  num_cores_available = multiprocessing.cpu_count()
except:
  num_cores_available = 1

parser.add_argument(
  '--num-workers', '-#',
  type=int,
  default=num_cores_available,
  help="Number of workers. Defaults to the number of cores available."
)


################################################################


args = parser.parse_args()


################################################################


data_folder = os.path.dirname(sys.argv[0])
template_folder = os.path.join(data_folder, "GenericFolderIcon.iconset")

convert_path = "convert"
iconutil_path = "iconutil"
seticon_path = os.path.join(data_folder, "lib", "seticon")

temp_folder = tempfile.mkdtemp()

if args.target:
  iconset_folder = os.path.join(temp_folder, "iconset.iconset")
  icns_file = os.path.join(temp_folder, "icns.icns")
else:
  root, _ = os.path.splitext(args.mask_file)
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
    args.mask_file,
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

  p = subprocess.call(command)
  print "AAA"


################################################################


print ""
print "Making icon file for %s" % args.mask_file
print "----------------"

# mkdir -p "${ICONSET_FOLDER}"

inputs = [
  ["16x16",       12,   8,  1], ["16x16@2x",    26,  14,  2],
  ["32x32",       26,  14,  2], ["32x32@2x",    52,  26,  2],

  ["128x128",    103,  53,  4], ["128x128@2x", 206, 106,  9],
  ["256x256",    206, 106,  9], ["256x256@2x", 412, 212, 18],
  ["512x512",    412, 212, 18], ["512x512@2x", 824, 424, 36]
]

print "Using %d workers." % args.num_workers
pool = multiprocessing.Pool(processes=args.num_workers)
f = functools.partial(folderify, args)
processes = pool.map(f, inputs)
print "CCC"
# for p in processes:

################################################################
print "BBB"

print "----------------"
print "Making the .icns file..."

p = subprocess.Popen([
  iconutil_path,
  "--convert", "icns",
  "--output", icns_file,
  iconset_folder
])
p.communicate()
p.wait()


################################################################

p = subprocess.Popen([
  seticon_path,
  "-d", icns_file,
  args.target
])
p.communicate()
p.wait()


################################################################


shutil.rmtree(temp_folder)


print "----------------"
print "Done."
print "AAA"
