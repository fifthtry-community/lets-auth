#!/bin/bash

set -uxe
set -o pipefail

sh -c "$(curl -fsSL https://fastn.com/install.sh)"

./scripts/build-wasm.sh || exit 1
./scripts/optimise-wasm.sh || exit 1

rm .gitignore

echo .packages > .gitignore
echo .fastn >> .gitignore
echo .is-local >> .gitignore

cd lets-auth.fifthtry.site/ && fastn upload lets-auth
