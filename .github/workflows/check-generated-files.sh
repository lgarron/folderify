#!/bin/bash

function check_file {
  FILE="${1}"
  if test -f "${FILE}"
  then
      echo "✅ File ${FILE} exists as expected."
  else
      echo "❌ File ${FILE} should exist, but doesn't."
      exit 1
  fi
}

function check_folder {
  FILE="${1}"
  if test -d "${FILE}"
  then
      echo "✅ Folder ${FILE} exists as expected."
  else
      echo "❌ Folder ${FILE} should exist, but doesn't."
      exit 1
  fi
}

check_file "./examples/src/apple.icns"
check_folder "./examples/src/apple.iconset"
