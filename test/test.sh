#!/usr/bin/env bash

set -euo pipefail

DIR="$(dirname "$0")"
source "${DIR}/helpers.sh"

echo -e "\nTest help flag."
python -m folderify -h

echo -e "\nGenerate icon file."
python -m folderify ./examples/src/apple.png

echo -e "\nCheck generated files."
check_file "./examples/src/apple.icns"
check_folder "./examples/src/apple.iconset"

echo -e "\nAssign folder icon."
python -m folderify ./examples/src/apple.png .

echo -e "\nCheck folder icon."
check_folder_icon "."

echo -e "\nTest that --verbose is accepted."
python -m folderify --verbose ./examples/src/apple.png

echo -e "\nTest that --no-trim is accepted."
python -m folderify --no-trim ./examples/src/apple.png

for version in "10.5" "10.6" "10.7" "10.8" "10.9" "10.10" "10.11" "10.12" "10.13" "10.14" "10.15" "11.0"
do
  echo -e "\nTest that --macOS ${version} is accepted."
  python -m folderify --macOS ${version} ./examples/src/apple.png
done
