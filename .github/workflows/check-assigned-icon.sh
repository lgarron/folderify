#!/bin/bash

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

check_folder_icon "."
