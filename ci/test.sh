#!/bin/bash

# Script for building your rust projects.
set -e

# shellcheck source=ci/common.sh
source ci/common.sh

# $1 {path} = Path to cross/cargo executable
CROSS=$1
# $1 {string} = <Target Triple>
TARGET_TRIPLE=$2

required_arg "$CROSS" 'CROSS'
required_arg "$TARGET_TRIPLE" '<Target Triple>'

$CROSS test --target "$TARGET_TRIPLE"
$CROSS test --target "$TARGET_TRIPLE" --all-features
