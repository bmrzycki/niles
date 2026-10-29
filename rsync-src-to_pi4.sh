#!/bin/bash

err() {
    echo "error: $1" 1>&2
    exit 1
}

[[ -n $1 ]] || err "usage - '$(basename $0)' USER@PI4_IP"
BASE="$(readlink -f "$(dirname $0)")"
cd "$BASE/.." || err "cd '$BASE/..'"

rm -f niles/src/version.rs || err "rm version.rs"
niles/version.sh || err "version.sh"

rsync \
    -avz \
    --exclude .git/ \
    --exclude target/ \
    niles "$1":
