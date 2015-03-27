from setuptools import setup, find_packages  # Always prefer setuptools over distutils
from os import path

# Get the long description from the relevant file
with open("README.md") as f:
    long_description = f.read()

setup(
  name="folderify",
  version="1.0.1",
  description="A tool to generate pretty, beveled OSX folder icons.",
  long_description=long_description,
  url="https://github.com/lgarron/folderify",

  author="Lucas Garron",
  author_email="code@garron.net",

  license="MIT",

  classifiers=[
    "Development Status :: 5 - Production/Stable",

    "Intended Audience :: Developers",
    "Topic :: Multimedia :: Graphics",
    "Topic :: Software Development :: Build Tools",
    "Topic :: Software Development :: Libraries",
    "Topic :: Utilities",

    "License :: OSI Approved :: MIT License",

    "Programming Language :: Python :: 2",
    "Programming Language :: Python :: 2.6",
    "Programming Language :: Python :: 2.7",
    # "Programming Language :: Python :: 3",
    # "Programming Language :: Python :: 3.2",
    # "Programming Language :: Python :: 3.3",
    # "Programming Language :: Python :: 3.4",
  ],

  # # What does your project relate to?
  keywords="icon OSX Mac Darwin graphics folder imagemagick",

  # If there are data files included in your packages that need to be
  # installed, specify them here.  If using Python 2.6 or less, then these
  # have to be included in MANIFEST.in as well.
  package_data={
    "folderify": [
      "GenericFolderIcon.pre-Yosemite.iconset/*.png",
      "GenericFolderIcon.Yosemite.iconset/*.png",
      "lib/seticon",
    ]
  },

  packages=[
    "folderify"
  ],
  package_dir={
    "folderify": "folderify"
  },


  # To provide executable scripts, use entry points in preference to the
  # "scripts" keyword. Entry points provide cross-platform support and allow
  # pip to create the appropriate form of executable for the target platform.
  entry_points={
    "console_scripts": [
        "folderify = folderify.__main__:main",
    ],
  },
)