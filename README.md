# folderify

![Apple Folder](examples/png/apple_folder_256.png)
![Cube Folder](examples/png/cube_folder_256.png)
![Octocat Folder](examples/png/octocat_folder_256.png)
![Rhombic Hexecontrahedron Folder](examples/png/rhombic_hexecontahedron_folder_256.png)
![Octocat Folder](examples/png/sysprefs_folder_256.png)


![apple.gif](examples/png/apple.gif)


# Install from `pip`

    pip install folderify

## Simple Usage

    curl https://raw.githubusercontent.com/lgarron/folderify/master/examples/src/octocat.png -o octocat.png
    mkdir new_dir
    folderify octocat.png new_dir --reveal


## Cache your folderified icons.

    # Set the icon for the folder, and cache it.
    curl https://raw.githubusercontent.com/lgarron/folderify/master/examples/src/octocat.png -o octocat.png
    mkdir new_dir
    folderify --cache octocat.png new_dir

    # Remove the source image and the folder.
    rm octocat.png && rm -rf new_dir

    # Recreate the directory and add the icon from the cache.
    mkdir new_dir
    folderify --cache-restore new_dir

    # View cache contents.
    folderify --cache-list

You are now safe(r) from programs that steamroll over metadata!


# Use without `pip`

    git clone git://github.com/lgarron/folderify.git
    cd folderify
    python -m folderify examples/src/folder_outline.png . --reveal

The repository folder should now have a custom icon.

    for file in examples/src/*.png; do python -m folderify $file; done
    open examples/src/

You should see a bunch of new `.iconset` folders and `.icns` files that were automatically generated from the `.png` masks.


## Usage

    usage: __main__.py [-h] [--reveal] [--osx VERSION] [--cache] [--cache-dir DIR]
                       [--cache-list] [--cache-restore PATH] [--cache-restore-all]
                       [--cache-remove PATH]
                       [mask] [target]

    Generate a native OSX folder icon from a mask file.

    positional arguments:
      mask                  Mask image file. For best results:
                              - Use a .png mask.
                              - Use a solid black design over a transparent background.
                              - Make sure the corner pixels of the mask image are transparent. They are used for empty margins.
                              - Make sure the icon is at least around 1024x1024, in order to look good at maximum Retina resolution.
      target                Target file or folder.   If a target is specified, the resulting icon will be applied to the target file/folder.   Else, a .iconset folder and .icns file will be created in the same folder as mask   (you can use "Get Info" in Finder to copy the icon from the .icns file).

    optional arguments:
      -h, --help            show this help message and exit
      --reveal, -r          Reveal the target (or resulting .icns file) in Finder.
      --osx VERSION, -x VERSION
                            Version of the OSX folder icon, e.g. "10.9" or "10.10".   Defaults to the version this computer is running (10.10).
      --cache, -c           Cache the mask icon in the cache dir.
      --cache-dir DIR       Use the specified cache directory (default: /Users/lgarron/.folderify/cache).
      --cache-list          List all paths with cached masks.
      --cache-restore PATH  Restore folderified icon to the file/folder at PATH,  using the mask image in the cache for that path.
      --cache-restore-all   Restore all paths that have been cached.
      --cache-remove PATH   Remove the cached mask for the file/folder at PATH.


### Dependencies

- [ImageMagick](http://www.imagemagick.org/) - for image processing (you should be able to run <code>convert</code> on the commandline).
- Python 2 - to help assign the icon file to itself.
- Apple Developer Tools (for `iconutil`)


## Info

`GenericFolderIcon.iconset` is generated from the OSX default folder icon using:

    iconutil --convert iconset --output GenericFolderIcon.iconset "/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/GenericFolderIcon.icns"

Icons are set using [`osxiconutils`](http://www.sveinbjorn.org/osxiconutils), a GPL-licensed project by Sveinbjorn Thordarson (based on [`IconFamily`](http://iconfamily.sourceforge.net/)).
