#!/usr/bin/env python

from __future__ import print_function

import argparse
import functools
import os
import os.path
import platform
import shutil
import subprocess
import sys
import tempfile

from string import Template

################################################################

DEBUG = "FOLDERIFY_DEBUG" in os.environ and os.environ["FOLDERIFY_DEBUG"] == "1"
OLD_IMPLEMENTATION_FOLDER_STYLES = ["Yosemite", "pre-Yosemite"]

def main():

  LOCAL_MACOS_VERSION = ".".join(platform.mac_ver()[0].split(".")[:2])

  parser = argparse.ArgumentParser(
    description="Generate a native-style macOS folder icon from a mask file.",
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
- Make sure the non-transparent pixels span a height of 384px, using a 16px grid.\n\
  If the height is 384px and the width is a multiple of 128px, each 64x64 tile will exactly align with 1 pixel at the smallest folder size.")

  parser.add_argument(
    "target",
    action="store",
    nargs="?",
    type=str,
    help="Target file or folder. \
If a target is specified, the resulting icon will be applied to the target file/folder.\n\
Else, a .iconset folder and .icns file will be created in the same folder as the mask \
(you can use \"Get Info\" in Finder to copy the icon from the .icns file).")

  parser.add_argument(
    "--reveal", "-r",
    action="store_true",
    help="Reveal the target (or resulting .icns file) in Finder.")

  parser.add_argument(
    "--macOS",
    type=str,
    metavar="VERSION",
    default=LOCAL_MACOS_VERSION,
    help=("Version of the macOS folder icon, e.g. \"10.13\". \
Defaults to the version currently running (%s)." % LOCAL_MACOS_VERSION))

  parser.add_argument(
    "--osx", "-x",
    type=str,
    metavar="VERSION",
    help=("Synonym for the --macOS argument.")
  )

  parser.add_argument(
    "--color-scheme",
    type=str,
    metavar="COLOR_SCHEME",
    default="auto",
    help=("Color scheme: auto (match current system), light, dark.")
  )

  parser.add_argument(
    "--no-trim",
    action="store_true",
    help=("Don't trim margins from the mask. By default, transparent margins are trimmed from all 4 sides.")
  )

  parser.add_argument(
    "--set-icon-using",
    type=str,
    metavar="TOOL",
    default="auto",
    help="Tool to used to set the icon of the target: auto (default), seticon, Rez.\n\
Rez usually produces a smaller \"resource fork\" for the icon, but only works if \
XCode command line tools are already installed and if you're using a folder target.")

  parser.add_argument(
    "--verbose", "-v",
    action="store_true",
    help="Detailed output.")

  ################################################################

  args = parser.parse_args()

  if args.mask and not os.path.exists(args.mask):
    parser.error("Mask file does not exist: %s" % args.mask)

  if args.target and not os.path.exists(args.target):
    parser.error("Target file/folder does not exist: %s" % args.target)

  ################################################################

  data_folder = os.path.dirname(sys.modules[__name__].__file__)

  if args.osx:
    args.macOS = args.osx

  effective_color_scheme = args.color_scheme
  if effective_color_scheme == "auto":
    try:
      apple_interface_style = subprocess.check_output(["/usr/bin/env", "defaults", "read", "-g", "AppleInterfaceStyle"])
      # Comparison compatible with Python 2 and Python 3:
      if apple_interface_style.strip() == "Dark".encode("ascii"):
        effective_color_scheme = "dark"
      else:
        effective_color_scheme = "light"
    except:
      sys.stderr.write("Could not automatically calculate color scheme. Defaulting to light.\n")
      effective_color_scheme = "light"
  if effective_color_scheme not in ["light", "dark"]:
    sys.stderr.write("Invalid color scheme. Defaulting to light.\n")
    effective_color_scheme = "light"

  set_icon_using = args.set_icon_using
  if set_icon_using == "auto":
    # `seticon` is the most compatible at the moment
    set_icon_using = "seticon"
  if set_icon_using == "rez":
    # Accept lowercase. The actual binary is `Rez` (`man Rez` works but `man
    # rez` doesn't), but macOS is case-insensitive and `rez` matches the case
    # you'd expect if there weren't legacy considerations.
    set_icon_using = "Rez"
  if set_icon_using not in ["seticon", "Rez"]:
    sys.stderr.write("Invalid icon tool specified. Defaulting to seticon.\n")
    set_icon_using = "seticon"

  if args.macOS in ["10.5", "10.6", "10.7", "10.8", "10.9"]:
    # http://arstechnica.com/apple/2007/10/mac-os-x-10-5/4/
    folder_style = "pre-Yosemite"
    if effective_color_scheme == "dark":
      print("Dark mode is not currently implemented for pre-Yosemite. Defaulting to light.")
  elif args.macOS in ["10.10", "10.11", "10.12", "10.13", "10.14", "10.15"]:
    folder_style = "Yosemite"
    if effective_color_scheme == "dark":
      print("Dark mode is not currently implemented for Yosemite. Defaulting to light.")
  else:
    folder_style = "BigSur"
    if effective_color_scheme == "dark":
      folder_style = "BigSur.dark"

  template_folder = os.path.join(
    data_folder, "GenericFolderIcon.%s.iconset" % folder_style)

  convert_path = "convert"
  iconutil_path = "iconutil"
  sips_path = "sips"
  DeRez_path = "DeRez"
  Rez_path = "Rez"
  SetFile_path = "SetFile"
  seticon_path = os.path.join(data_folder, "lib", "seticon")

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

  def create_iconset(folder_style, print_prefix, mask, temp_folder, iconset_folder, colors, dimensions):
    if folder_style in OLD_IMPLEMENTATION_FOLDER_STYLES:
      return create_iconset_old_implementation(print_prefix, mask, temp_folder, iconset_folder, dimensions)

    global processes

    name, icon_size, centering, dims, b, w = dimensions
    centering_width, centering_height = centering
    width, height, offset_center = dims
    black_blur, black_offset = b
    white_blur, white_offset, white_opacity = w

    if args.verbose:
      print("[%s] %s" % (print_prefix, name))

    SIZED_MASK = os.path.join(temp_folder, "%s_1.0_SIZED_MASK.png" % name)
    try:
      subprocess.check_call(p(
        convert_path,
        g(
          # We first center the image in the max size, to avoid pixel
          # rounding when the trimmed image is an odd width. (The extra
          # pixel would extend to the right, making the right margin
          # 1px slimmer. This is quite noticable on 16x16.)
          mask,
          "-background", "transparent",
          [] if args.no_trim else "-trim",
          "-resize", ("%dx%d" % (centering_width, centering_height)),
          "-gravity", "Center",
          "-extent", ("%dx%d" % (centering_width, centering_height))
        ),
        "-background", "transparent",
        "-resize", ("%dx%d" % (width , height)),
        "-gravity", "Center",
        "-extent", ("%dx%d+0-%d" % (icon_size, icon_size, offset_center)),
        SIZED_MASK
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

    def process(step_name, args):
      if DEBUG:
        file_name = os.path.join(temp_folder, "%s_%s.png" % (name, step_name))
        print(file_name)
        subprocess.Popen(p(convert_path, args, file_name)).wait()
        return file_name
      else:
        return args

    def colorize(step_name, fill, input):
      return process(step_name, g(input, "-fill", fill, "-colorize", "100, 100, 100"))
    
    def opacity(step_name, fraction, input):
      return process(step_name, g(input, "-channel", "Alpha", "-evaluate", "multiply", fraction))
    
    def blur_down(step_name, blur_px, offset_px, input):
      return process(step_name, g(input, "-motion-blur", ("0x%d-90" % blur_px), "-page", ("+0+%d" % offset_px), "-background", "none", "-flatten"))

    def mask_down(step_name, mask_operation, input, mask):
      return process(step_name, g(input, mask, "-alpha", "Set", "-compose", mask_operation, "-composite"))
    
    def negate(step_name, input):
      return process(step_name, g(input, "-negate"))

    FILL_COLORIZED = colorize("2.1_FILL_COLORIZED", colors["fill"], SIZED_MASK)
    FILL = opacity("2.2_FILL", "0.5", FILL_COLORIZED)

    BLACK_NEGATED = negate("3.1_BLACK_NEGATED", SIZED_MASK)
    BLACK_COLORIZED = colorize("3.2_BLACK_COLORIZED", "rgb(58, 152, 208)", BLACK_NEGATED)
    BLACK_BLURRED = blur_down("3.3_BLACK_BLURRED", black_blur, black_offset, BLACK_COLORIZED)
    BLACK_MASKED = mask_down("3.4_BLACK_MASKED", "Dst_In", BLACK_BLURRED, SIZED_MASK)
    BLACK_SHADOW = opacity("3.5_BLACK_SHADOW", "0.5", BLACK_MASKED)

    WHITE_COLORIZED = colorize("4.1_WHITE_COLORIZED", "rgb(174, 225, 253)", SIZED_MASK)
    WHITE_BLURRED = blur_down("4.2_WHITE_BLURRED", white_blur, white_offset, WHITE_COLORIZED)
    WHITE_MASKED = mask_down("4.3_WHITE_MASKED", "Dst_Out", WHITE_BLURRED, SIZED_MASK)
    WHITE_SHADOW = opacity("4.4_WHITE_SHADOW", white_opacity, WHITE_MASKED)

    COMPOSITE = g(
      template_icon,
      WHITE_SHADOW,
      "-compose", "dissolve", "-composite",
      FILL,
      "-compose", "dissolve", "-composite",
      BLACK_SHADOW,
      "-compose", "dissolve", "-composite"
    )

    command = p(
      convert_path,
      COMPOSITE, # can be replaced with an intermediate step for debugging.
      FILE_OUT
    )

    return subprocess.Popen(command)

  # Messy implementation for from Yosemite
  # TODO: Unify this with the new implementation?
  def create_iconset_old_implementation(print_prefix, mask, temp_folder, iconset_folder, params):
    global processes

    name, width, height, offset_center = params

    if args.verbose:
      print("[%s] %s" % (print_prefix, name))

    TEMP_MASK_IMAGE = os.path.join(temp_folder, "trimmed_%s.png" % name)
    try:
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
    offset_white = 1
    opacity_white = 100

    # Here comes the magic.
    # TODO: rewrite in Python.
    command = [
      convert_path, template_icon, "(",
      "(", TEMP_MASK_IMAGE, "-colorize", "3,23,40", ")",
      "(",
      "(",
      "(",
      TEMP_MASK_IMAGE,
      "(",
      TEMP_MASK_IMAGE, "-channel", "rgb", "-negate", "+channel", "-shadow", "100x1+10+0", "-geometry", "-2-2",
      ")",
      "-compose", "dst-out", "-composite", "+repage",
      ")",
      "(",
      TEMP_MASK_IMAGE,
      "(",
      TEMP_MASK_IMAGE, "-channel", "rgb", "-negate", "+channel", "-geometry", "+0-1",
      ")",
      "-compose", "dst-out", "-composite", "+repage", "-channel", "rgb", "-negate", "+channel", "-geometry", (
        "+0+%d" % offset_white),
      ")",
      "-compose", "dissolve", "-define", ("compose:args=%dx50" %
                                          opacity_white), "-composite", "+repage",
      ")",
      "(",
      TEMP_MASK_IMAGE,
      "(",
      TEMP_MASK_IMAGE, "-channel", "rgb", "-negate", "+channel", "-geometry", "+0+1",
      ")",
      "-compose", "dst-out", "-composite", "+repage",
      ")",
      "-compose", "dissolve", "-define", "compose:args=50x80", "-composite",
      ")",
      "-compose", "dissolve", "-define", ("compose:args=60x%d" %
                                          main_opacity), "-composite", "+repage",
      "-gravity", "Center", "-geometry", ("+0+%d" % offset_center),
      "+repage",
      ")",
      "-compose", "over", "-composite", FILE_OUT
    ]

    return subprocess.Popen(command)

  ################################################################

  def create_and_set_icns(mask, target=None):

    if DEBUG:
      temp_folder = "%s-folderify-debug" % mask
      if not os.path.exists(temp_folder):
        os.mkdir(temp_folder)
      subprocess.Popen(["open", temp_folder]).wait()
    else:
      temp_folder = tempfile.mkdtemp()

    original_target = target
    if target:
      iconset_folder = os.path.join(temp_folder, "iconset.iconset")
      icns_file = os.path.join(temp_folder, "icns.icns")

      os.mkdir(iconset_folder)

      print_prefix = target
      print("[%s] => assign to [%s]" % (mask, target))
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

    # The following can be excluded, since they are essentially
    # indistinguishable from the preceding @2x resolution:
    #
    # - 32x32
    # - 256x256
    # - 512x512
    #
    # This saves about 20%. However, it breaks Quicklook preview for the
    # `.iconset`, and it's possible that some programs assume all sizes are present.
    #
    # Data: Name, icon size, dimensions, black shadow, white top shadow, white bottom shadow
    inputs = {
      "BigSur": {
        "colors": {
          "fill": "rgb(8, 134, 206)"
        },
        "dimensions": [
          ["16x16",      16, (768, 384), (12, 6, 2), (0, 2), (1, 0, "0.5")],
          ["16x16@2x",   32, (768, 384), (24, 12, 2), (0, 2), (2, 1, "0.35")],
          ["32x32",      32, (768, 384), (24, 12, 2), (0, 2), (2, 1, "0.35")], # Can be excluded
          ["32x32@2x",   64, (768, 384), (48, 24, 3), (0, 2), (2, 1, "0.6")],
          ["128x128",    128, (768, 384), (96, 48, 6), (0, 2), (2, 1, "0.6")],
          ["128x128@2x", 256, (768, 384), (192, 96, 12), (0, 2), (2, 1, "0.6")], # Can be excluded
          ["256x256",    256, (768, 384), (192, 96, 12), (0, 2), (2, 1, "0.6")],
          ["256x256@2x", 512, (768, 384), (384, 192, 24), (0, 2), (2, 1, "0.75")], # Can be excluded
          ["512x512",    512, (768, 384), (384, 192, 24), (0, 2), (2, 1, "0.75")],
          ["512x512@2x", 1024, (768, 384), (768, 384, 48), (0, 2), (2, 1, "0.75")]
        ]
      },
      "BigSur.dark": {
        "colors": {
          "fill": "rgb(6, 111, 194)"
        },
        "dimensions": [
          ["16x16",      16, (768, 384), (12, 6, 2), (0, 2), (1, 0, "0.5")],
          ["16x16@2x",   32, (768, 384), (24, 12, 2), (0, 2), (2, 1, "0.35")],
          ["32x32",      32, (768, 384), (24, 12, 2), (0, 2), (2, 1, "0.35")], # Can be excluded
          ["32x32@2x",   64, (768, 384), (48, 24, 3), (0, 2), (2, 1, "0.6")],
          ["128x128",    128, (768, 384), (96, 48, 6), (0, 2), (2, 1, "0.6")],
          ["128x128@2x", 256, (768, 384), (192, 96, 12), (0, 2), (2, 1, "0.6")], # Can be excluded
          ["256x256",    256, (768, 384), (192, 96, 12), (0, 2), (2, 1, "0.6")],
          ["256x256@2x", 512, (768, 384), (384, 192, 24), (0, 2), (2, 1, "0.75")], # Can be excluded
          ["512x512",    512, (768, 384), (384, 192, 24), (0, 2), (2, 1, "0.75")],
          ["512x512@2x", 1024, (768, 384), (768, 384, 48), (0, 2), (2, 1, "0.75")]
        ]
      },
      "Yosemite": {
        "colors": {},
        "dimensions": [
          ["16x16",       12,   8,  1], ["16x16@2x",    26,  14,  2],
          ["32x32",       26,  14,  2], ["32x32@2x",    52,  26,  2],

          ["128x128",    103,  53,  4], ["128x128@2x", 206, 106,  9],
          ["256x256",    206, 106,  9], ["256x256@2x", 412, 212, 18],
          ["512x512",    412, 212, 18], ["512x512@2x", 824, 424, 36]
        ]
      },
      "pre-Yosemite": {
        "colors": {},
        "dimensions": [
          ["16x16",       12,   8,  1], ["16x16@2x",    26,  14,  2],
          ["32x32",       26,  14,  2], ["32x32@2x",    52,  30,  4],

          ["128x128",    103,  60,  9], ["128x128@2x", 206, 121, 18],
          ["256x256",    206, 121, 18], ["256x256@2x", 412, 242, 36],
          ["512x512", 412, 242, 36], ["512x512@2x", 824, 484, 72]
        ]
      }
    }

    print("[%s] Using folder style: %s" % (print_prefix, folder_style))
    f = functools.partial(create_iconset, folder_style, print_prefix,
                    mask, temp_folder, iconset_folder, inputs[folder_style]["colors"])
    processes = map(f, inputs[folder_style]["dimensions"])

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

    if set_icon_using == "seticon":
      # Make sure seticon is executable.
      subprocess.check_call([
        "chmod",
        "+x",
        seticon_path
      ])

      if args.verbose:
        print("[%s] Setting icon for target %s with seticon." % (print_prefix, target))
      # Set icon for target.
      subprocess.check_call([
        seticon_path,
        icns_file,
        target
      ])
    else:
      if args.verbose:
        print("[%s] Setting icon for target %s with sips/DeRez/Rez/SetFile." % (print_prefix, target))

      # sips: add an icns resource fork to the icns file
      subprocess.check_call([
        sips_path,
        "-i",
        icns_file
      ])

      if original_target:
        if not os.path.isdir(target):
          sys.stderr.write("[%s] Warning: the target path does not appear to be a folder. Setting the icon using Rez will probably fail. Try --set-icon-using seticon instead.\n\n" % print_prefix)

        temp_file = os.path.join(temp_folder, "tmpicns.rsrc")
        target_icon = os.path.join(target, "Icon\r")

        # DeRez: export the icns resource from the icns file
        with open(temp_file, "w") as f:
          subprocess.check_call([
            DeRez_path,
            "-only",
            "icns",
            icns_file
          ], stdout=f)

        # Rez: add exported icns resource to the resource fork of target/Icon^M
        try:
          # If XCode command line tools are not installed, here's where we'd first run into issues.
          subprocess.check_call([
            Rez_path,
            "-append",
            temp_file,
            "-o",
            target_icon
          ])
        except OSError as e:
          # If `Rez` is not installed, we get `FileNotFoundError` in higher versions of Python. But we have to use `OSError` for compatibility with Python 2.7.
          sys.stderr.write("[%s] Could not set the target icon, probably because Rez is not installed. If you want to use it, make sure XCode command line tools are installed.\n" % print_prefix)
          sys.stderr.write("%s" % e)
          sys.exit(1)

        # SetFile: set custom icon attribute
        subprocess.check_call([
          SetFile_path,
          "-a",
          "C",
          target
        ])

        # SetFile: set invisible file attribute
        subprocess.check_call([
          SetFile_path,
          "-a",
          "V",
          target_icon
        ])

    # Clean up.
    if not DEBUG:
      shutil.rmtree(temp_folder)

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

  if args.mask:
    create_and_set_icns(
      args.mask,
      target=args.target
    )

  else:
    parser.print_help()

__main__ = main

if __name__ == "__main__":
  main()
