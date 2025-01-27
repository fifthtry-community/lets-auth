#!/bin/bash

# Define global variable for wasm-opt command
WASM_OPT_CMD=""
export BINARYEN_VERSION="${BINARYEN_VERSION:-version_119}"


function download_release() {
  repo_name=$1
  version=$2
  binary_name=$3

  url="https://github.com/${repo_name}/releases/download/${version}/${binary_name}"

  echo "Downloading release from $url"

  # Use 'command time' to ensure cross-platform compatibility
  { time curl -L -o "${binary_name}" "${url}"; } 2>&1
}

function ensure_wasm_opt() {
  # Platform-specific handling for the binary name
  platform=$(uname | tr '[:upper:]' '[:lower:]') # Convert to lowercase
  case "$platform" in
    "linux")
      binary_name="binaryen-${BINARYEN_VERSION}-x86_64-linux.tar.gz"
      ;;
    "darwin")
      binary_name="binaryen-${BINARYEN_VERSION}-x86_64-macos.tar.gz"
      ;;
    *)
      echo "Unsupported platform: $platform"
      exit 1
      ;;
  esac

  # Variables for the download
  repo_name="WebAssembly/binaryen"

  local_install_dir="./bin/binaryen-${BINARYEN_VERSION}"

  # Check if wasm-opt exists globally in PATH
  if command -v wasm-opt &> /dev/null; then
    echo "Using the globally installed wasm-opt."
    export WASM_OPT_CMD="wasm-opt" # Use global version
  else
    echo "wasm-opt not found in PATH. Setting up local version..."

    # Ensure the ./bin directory exists
    mkdir -p ./bin

    # Download the tar.gz file if necessary
    download_release "${repo_name}" "${BINARYEN_VERSION}" "${binary_name}"

    # Extract the tar.gz file
    echo "Extracting ${binary_name}..."
    tar -xzf "${binary_name}" -C ./bin/

    # Ensure the extracted binary is accessible (optional: verify existence)
    if [ ! -f "${local_install_dir}/bin/wasm-opt" ]; then
      echo "wasm-opt not found in the extracted files."
      exit 1
    fi

    # Remove the tar.gz file after extraction
    echo "Removing ${binary_name}..."
    rm -f "${binary_name}"

    # Set the local version of wasm-opt
    export WASM_OPT_CMD="${local_install_dir}/bin/wasm-opt"
  fi

  # shellcheck disable=SC2016
  echo 'use $WASM_OPT_CMD as wasm-opt'
}

function wasm_opt() {
  if [ -z "$WASM_OPT_CMD" ]; then
    echo "Error: WASM_OPT_CMD is not set."
    return 1
  fi
  if [ $# -ne 1 ] || [ ! -f "$1" ]; then
    echo "Error: Provide a valid WASM file."
    return 1
  fi

  wasm_file=$1

  # Define a helper function to get file size
  get_size() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
      stat -f%z "$1" # macOS
    else
      stat -c%s "$1" # Linux
    fi
  }

  # Get the file size before optimization
  before_size=$(get_size "$wasm_file")
  $WASM_OPT_CMD -Oz "$wasm_file" -o "$wasm_file"
  if [ $? -ne 0 ]; then
    echo "Error: Optimization failed."
    return 1
  fi

  # Get the file size after optimization
  after_size=$(get_size "$wasm_file")
  size_diff=$((before_size - after_size))

  # Print results
  echo "$wasm_file: Before = ${before_size} bytes, After = ${after_size} bytes, Reduction = ${size_diff} bytes"
}

ensure_wasm_opt

wasm_opt app/mobile_auth_provider.wasm
wasm_opt app/email_auth_provider.wasm
