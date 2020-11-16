#!/usr/bin/env bash

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

function check_folder_icon {
  FILE="${1}/"
  if test -f "${FILE}"Icon$'\r'
  then
      echo "✅ Folder ${FILE} has an icon as expected."
  else
      echo "❌ Folder ${FILE} should have an icon, but doesn't."
      exit 1
  fi
}
