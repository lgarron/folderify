#!/usr/bin/env bash

set -euo pipefail

DIR="$(dirname "$0")"
source "${DIR}/helpers.sh"

echo -e "\nTest help flag."
cargo run -- -h

echo -e "\nGenerate icon file."
cargo run -- ./examples/src/apple.png

echo -e "\nCheck generated files."
check_file "./examples/src/apple.icns"
check_folder "./examples/src/apple.iconset"

TEMP_DIR=$(make_temp_folder)
echo -e "\nAssign folder icon."
cargo run -- ./examples/src/apple.png "${TEMP_DIR}"
echo -e "\nCheck folder icon."
check_folder_icon "${TEMP_DIR}"
rm -r "${TEMP_DIR}"

TEMP_DIR_REZ=$(make_temp_folder)
echo -e "\nAssign folder icon with --set-icon-using Rez."
cargo run -- --set-icon-using Rez ./examples/src/apple.png "${TEMP_DIR_REZ}"
echo -e "\nCheck folder icon assigned with --set-icon-using Rez."
check_folder_icon "${TEMP_DIR_REZ}"
rm -r "${TEMP_DIR_REZ}"

echo -e "\nTest that \`--verbose\` is accepted."
cargo run -- --verbose ./examples/src/apple.png

echo -e "\nTest that \`--no-trim\` is accepted."
cargo run -- --no-trim ./examples/src/apple.png

echo -e "\nTest that \`--color-scheme auto\` is accepted."
cargo run -- --color-scheme auto ./examples/src/apple.png

echo -e "\nTest that \`--color-scheme light\` is accepted."
cargo run -- --color-scheme light ./examples/src/apple.png

echo -e "\nTest that \`--color-scheme dark\` is accepted."
cargo run -- --color-scheme dark ./examples/src/apple.png

echo -e "\nTest that \`--output-icns\` is accepted."
cargo run -- --output-icns ./examples/src/folder_outline_custom_path_1.icns ./examples/src/folder_outline.png
check_file ./examples/src/folder_outline_custom_path_1.icns
check_no_file_nor_folder ./examples/src/folder_outline.icns
check_no_file_nor_folder ./examples/src/folder_outline.iconset

echo -e "\nTest that \`--output-iconset\` is accepted."
cargo run -- --output-iconset ./examples/src/folder_outline_custom_path_2.iconset ./examples/src/folder_outline.png
check_folder ./examples/src/folder_outline_custom_path_2.iconset
check_no_file_nor_folder ./examples/src/folder_outline.icns
check_no_file_nor_folder ./examples/src/folder_outline.iconset

echo -e "\nTest that \`--output-icns\` and \`--output-iconset\` are accepted together."
cargo run -- --output-icns ./examples/src/folder_outline_custom_path_3.icns --output-iconset ./examples/src/folder_outline_custom_path_4.iconset ./examples/src/folder_outline.png
check_file ./examples/src/folder_outline_custom_path_3.icns
check_folder ./examples/src/folder_outline_custom_path_4.iconset
check_no_file_nor_folder ./examples/src/folder_outline.icns
check_no_file_nor_folder ./examples/src/folder_outline.iconset

# for version in "10.5" "10.6" "10.7" "10.8" "10.9" "10.10" "10.11" "10.12" "10.13" "10.14" "10.15" "11.0"
for version in "10.15" "13.3"
do
  echo -e "\nTest that --macOS ${version} is accepted."
  cargo run -- --macOS ${version} ./examples/src/apple.png
done
