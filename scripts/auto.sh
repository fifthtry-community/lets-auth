export PROJ_ROOT=$(pwd)
FASTN=${FASTN_BINARY:-fastn}

export BINARYEN_VERSION="version_119"

export PATH=$PATH:${PROJ_ROOT}/bin/binaryen-${BINARYEN_VERSION}/bin
export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8
export DB_FILE=${DB_FILE:-${PROJ_ROOT}/db.sqlite}
export DATABASE_URL=${DATABASE_URL:-sqlite:///${DB_FILE}}

function pushd2() {
    PUSHED=$(pwd)
    cd "${PROJDIR}""$1" >> /dev/null || return
}

function popd2() {
    cd "${PUSHED:-$PROJDIR}" >> /dev/null || return
    unset PUSHED
}

function build-mobile-wasm() {
    pushd2 "${PROJ_ROOT}/mobile-auth-provider" || return 1
    # cargo clean
    cargo build --target wasm32-unknown-unknown --release || return 1
    cp ../target/wasm32-unknown-unknown/release/mobile_auth_provider.wasm . || return 1
    popd2
}


function build-email-wasm() {
    pushd2 "${PROJ_ROOT}/email-auth-provider" || return 1
    # cargo clean
    cargo build --target wasm32-unknown-unknown --release || return 1
    cp ../target/wasm32-unknown-unknown/release/email_auth_provider.wasm . || return 1
    popd2
}

function dotcom() {
  build-email-wasm || return 1
  pushd2 "${PROJ_ROOT}" || return 1
  cp ./email-auth-provider/email_auth_provider.wasm ../dotcom/frontend/email_auth.wasm || return 1
  popd2
}


function update-ui() {
  pushd2 "${PROJ_ROOT}/lets-auth-system.fifthtry.site" || return 1
  $FASTN update
  popd2
}

function run-ui() {
  pushd2 "${PROJ_ROOT}/lets-auth-system.fifthtry.site" || return 1

  echo "Using $FASTN to serve lets-auth-system.fifthtry.site/"

  $FASTN --trace serve --port 8002 --offline

  popd2
}

function update-www() {
  pushd2 "${PROJ_ROOT}/lets-auth.fifthtry-community.com" || return 1
  $FASTN update
  popd2
}

function run-www() {
  pushd2 "${PROJ_ROOT}/lets-auth.fifthtry-community.com" || return 1

  echo "Using $FASTN to serve lets-auth.fifthtry-community.com/"

  $FASTN --trace serve --port 8003 --offline

  popd2
}

function update-template() {
  pushd2 "${PROJ_ROOT}/lets-auth-template.fifthtry.site" || return 1
  $FASTN update
  popd2
}

function build-wasm() {
  pushd2 "${PROJ_ROOT}" || return 1

  # this script should be used both for local development and for building on ci
  sh scripts/build-wasm.sh

  popd2
}


function run-template() {
  pushd2 "${PROJ_ROOT}/lets-auth-template.fifthtry.site" || return 1

  build-wasm
  echo "consider update-template if this fails or if you modify dependencies"
  $FASTN --trace serve --offline
}
