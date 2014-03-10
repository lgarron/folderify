# folderify

![Apple Folder](examples/png/apple_folder_256.png)
![Cube Folder](examples/png/cube_folder_256.png)
![Octocat Folder](examples/png/octocat_folder_256.png)
![Rhombic Hexecontrahedron Folder](examples/png/rhombic_hexecontahedron_folder_256.png)
![Octocat Folder](examples/png/sysprefs_folder_256.png)

# Try it!

    git clone git://github.com/lgarron/folderify.git
    cd folderify
    for file in examples/src/*.png; do ./folderify $file; done
    open examples/src/

You should see a bunch of new `.icns` files that were automatically generated from the `.png` masks.

Or try this:

    git clone git://github.com/lgarron/folderify.git
    cd folderify
    ./folderify examples/src/folder_outline.png .

The repository folder should now have an icon.

## Usage

Command:

    folderify <image.png> [<optional_target_file_or_folder>]
 
- The input file should be an image with a transparent background. For best results:
  - Use a `.png` file
  - Use a black figure on a transparent background. (Colored images also work, but may produce a weaker effect.)
  - Make sure the corner pixels of the image are transparent. They are currently used for empty margins.
- folderify will produce a `.icns` file containing 5 resolutions of folder icons. The `.icns` file will also have itself as its icon.
  - (Note that normal `.icns` files do NOT have themselves as an icon.)
- The icon can be copied from the generated `.icns` to any other file folder using the "Get Info" pane in the Finder.
  - (Note that this does NOT normally work for `.icns` files, but it does work for such files that have been generated using folderify.)
 
### Dependencies

- [ImageMagick](http://www.imagemagick.org/) - for image processing (you should be able to run <code>convert</code> on the commandline).
- Python 2 - to help assign the icon file to itself.
- Apple Developer Tools (for /Developer/Tools/Rez)
