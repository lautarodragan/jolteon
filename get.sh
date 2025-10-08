#!/bin/env bash

JSON=`curl -s 'https://api.github.com/repos/lautarodragan/jolteon/releases/latest'`

URLS=`echo "$JSON" | jq -r ".assets[].browser_download_url"`
# curl -s 'https://api.github.com/repos/lautarodragan/jolteon/releases/latest' | grep "browser_download_url"

DOWNLOAD_URL=""
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
  DOWNLOAD_URL=`echo "$URLS" | grep linux`
elif [[ "$OSTYPE" == "darwin"* ]]; then
  # echo "$URLS" | grep darwin
  if [[ `uname -m` == "arm64" ]]; then
    DOWNLOAD_URL=`echo "$URLS" | grep darwin | grep aarch`
  elif [[ "$OSTYPE" == "x86_64" ]]; then
    DOWNLOAD_URL=`echo "$URLS" | grep darwin | grep x86_64`
  else
    echo "No release found for your OS."
    echo "If you're running on one of the supported OSes (Linux, MacOS Intel/ARM), then this is a bug in the installation script."
    echo "Please report this by submitting a bug to https://github.com/lautarodragan/jolteon/issues."
    exit 1
  fi
else
  echo "No release found for your OS."
  echo "If you're running on one of the supported OSes (Linux, MacOS Intel/ARM), then this is a bug in the installation script."
  echo "Please report this by submitting a bug to https://github.com/lautarodragan/jolteon/issues."
  exit 1
fi

echo "Downloading $DOWNLOAD_URL..."

curl -s -L -O "$DOWNLOAD_URL"

echo "Extracting..."

tar xzf jolteon*.tar.gz
