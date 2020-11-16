#!/usr/bin/env bash

set -euo pipefail

DIR="$(dirname "$0")"
source "${DIR}/helpers.sh"

echo -e "\nGenerate icon file."
python -m folderify ./examples/src/apple.png

echo -e "\nCheck generated files."
check_file "./examples/src/apple.icns"
check_folder "./examples/src/apple.iconset"

echo -e "\nAssign folder icon."
python -m folderify ./examples/src/apple.png ./Makefile

echo -e "\nCheck folder icon."
check_folder_icon "."
