#!/usr/bin/env bash

set -euo pipefail

DIR="$(dirname "$0")"
source "${DIR}/helpers.sh"

# Homebrew expects `seticon` at an exact path (so that it can overwrite the
# binary with an architecture-specific build).
# https://github.com/Homebrew/homebrew-core/pull/87346
check_file ./folderify/lib/seticon

echo -e "\nTest help flag."
python -m folderify -h

echo -e "\nGenerate icon file."
python -m folderify ./examples/src/apple.png

echo -e "\nCheck generated files."
check_file "./examples/src/apple.icns"
check_folder "./examples/src/apple.iconset"

TEMP_DIR=$(make_temp_folder)
echo -e "\nAssign folder icon."
python -m folderify ./examples/src/apple.png "${TEMP_DIR}"
echo -e "\nCheck folder icon."
check_folder_icon "${TEMP_DIR}"
rm -r "${TEMP_DIR}"

TEMP_DIR_REZ=$(make_temp_folder)
echo -e "\nAssign folder icon with --set-icon-using Rez."
python -m folderify --set-icon-using Rez ./examples/src/apple.png "${TEMP_DIR_REZ}"
echo -e "\nCheck folder icon assigned with --set-icon-using Rez."
check_folder_icon "${TEMP_DIR_REZ}"
rm -r "${TEMP_DIR_REZ}"

echo -e "\nTest that \`--verbose\` is accepted."
python -m folderify --verbose ./examples/src/apple.png

echo -e "\nTest that \`--no-trim\` is accepted."
python -m folderify --no-trim ./examples/src/apple.png

echo -e "\nTest that \`--color-scheme auto\` is accepted."
python -m folderify --color-scheme auto ./examples/src/apple.png

echo -e "\nTest that \`--color-scheme light\` is accepted."
python -m folderify --color-scheme light ./examples/src/apple.png

echo -e "\nTest that \`--color-scheme dark\` is accepted."
python -m folderify --color-scheme dark ./examples/src/apple.png

for version in "10.5" "10.6" "10.7" "10.8" "10.9" "10.10" "10.11" "10.12" "10.13" "10.14" "10.15" "11.0"
do
  echo -e "\nTest that --macOS ${version} is accepted."
  python -m folderify --macOS ${version} ./examples/src/apple.png
done
