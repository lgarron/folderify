# folderify

![mask.png + folder = folderified!](examples/png/explanation.png)

Generate pixel-perfect macOS folder icons in the native style.

Works for macOS 10.5 (Leopard)
through 10.11 (Big Sur) and automatically includes all icon sizes from `16x16` through `512x512@2x`.

# Install using [Homebrew](https://formulae.brew.sh/formula/folderify)

```shell
brew install folderify
```

# Usage

Use a mask to assign an icon to a folder:

```shell
folderify mask.png /path/to/folder
```

Generate `mask.icns` and `mask.iconset` files:

```shell
folderify mask.png
```

Generate icon files for a specific version of macOS (the default is your current
version):

```shell
folderify mask.png --macOS 11.0
```

# Other installation options

If you don't have Homebrew but you already have ImageMagick (the `convert`
binary) on your system, you can use the following:

## Install using `pip`

```shell
pip install folderify
```

## Download the source code directly

Or download the code directly:

```shell
curl -L https://github.com/lgarron/folderify/archive/main.zip -o folderify-main.zip
unzip folderify-main.zip && cd folderify-main
python -m folderify examples/src/folder_outline.png . --reveal
```

The repository folder should now have a custom icon.

```shell
for file in examples/src/*.png; do python -m folderify $file; done
open examples/src/
```

You should see a bunch of new `.iconset` folders and `.icns` files that were automatically generated from the `.png` masks.

## Full Options

```shell
usage: folderify [-h] [--reveal] [--macOS VERSION] [--osx VERSION] [--cache]
                    [--cache-dir DIR] [--cache-list] [--cache-restore PATH]
                    [--cache-restore-all] [--cache-remove PATH] [--verbose]
                    [mask] [target]

Generate a native-style macOS folder icon from a mask file.

positional arguments:
    mask                  Mask image file. For best results:
                        - Use a .png mask.
                        - Use a solid black design over a transparent background.
                        - Make sure the corner pixels of the mask image are transparent. They are used for empty margins.
                        - Make sure the icon is at least around 1024x1024, in order to look good at maximum Retina resolution.
    target                Target file or folder. If a target is specified, the resulting icon will be applied to the target file/folder.
                          Else, a .iconset folder and .icns file will be created in the same folder as themask (you can use "Get Info" in Finder to copy the icon from the .icns file).

optional arguments:
    -h, --help            show this help message and exit
    --reveal, -r          Reveal the target (or resulting .icns file) in Finder.
    --macOS VERSION       Version of the macOS folder icon, e.g. "10.13". Defaults to the version currently running (10.15).
    --osx VERSION, -x VERSION
                          Synonym for the --macOS argument.
    --cache, -c           Cache the mask icon in the cache dir.
    --cache-dir DIR       Use the specified cache directory (default: /Users/lgarron/.folderify/cache).
    --cache-list          List all paths with cached masks.
    --cache-restore PATH  Restore folderified icon to the file/folder at PATH, using the mask image in the cache for that path.
    --cache-restore-all   Restore all paths that have been cached.
    --cache-remove PATH   Remove the cached mask for the file/folder at PATH.
    --verbose, -v         Detailed output.
```

### Dependencies

- Python (version 2 or 3).
- [ImageMagick](http://www.imagemagick.org/) - for image processing (you should be able to run <code>convert</code> on the commandline).
- Apple Developer Tools (for `iconutil`).

## Info

On Yosemite or earlier, `GenericFolderIcon.iconset` is generated from the macOS default folder icon using:

```shell
iconutil --convert iconset --output GenericFolderIcon.iconset "/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/GenericFolderIcon.icns"
```

Icons are set using [`osxiconutils`](http://www.sveinbjorn.org/osxiconutils), a GPL-licensed project by Sveinbjorn Thordarson (based on [`IconFamily`](http://iconfamily.sourceforge.net/)).

---

![apple.gif](examples/png/apple.gif)
