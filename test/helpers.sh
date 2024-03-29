#!/usr/bin/env bash

function success {
  MESSAGE="${1}"
  echo "✅ ${MESSAGE}"
}

function failure {
  MESSAGE="${1}"
  echo "❌ ${MESSAGE}"
  exit 1
}

function make_temp_folder {
  mktemp -d
}

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
  ICON="${FILE}"Icon$'\r'
  # Check for "custom icon" attribute on target
  if ! xattr -p com.apple.FinderInfo "${FILE}" > /dev/null
  then
    echo "❌ Folder ${FILE} should have a FinderInfo attribute, but doesn't."
    exit 1
  fi
  # Check for "invisible" attribute on icon
  if ! xattr -p com.apple.FinderInfo "${ICON}" > /dev/null
  then
    echo "❌ Folder ${FILE} icon should have a FinderInfo attribute, but doesn't."
    exit 1
  fi
  # Check for icns data in icon
  if ! xattr -p com.apple.ResourceFork "${ICON}" > /dev/null
  then
    echo "❌ Folder ${FILE} icon should have a resource fork, but doesn't."
    exit 1
  fi
  echo "✅ Folder ${FILE} has an icon as expected."
}

function check_no_file_nor_folder {
  local TEST_PATH="${1}"
  if test -f "${TEST_PATH}"
  then
    echo "❌ Path ${TEST_PATH} should not exist, but is a file."
    exit 1
  elif test -d "${TEST_PATH}"
  then
    echo "❌ Path ${TEST_PATH} should not exist, but is a folder."
    exit 1
  else
    echo "✅ Path ${TEST_PATH} does not exist, as expected."
  fi
}
