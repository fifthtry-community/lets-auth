export PROJ_ROOT=$(pwd)

export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8
export DATABASE_URL=${DATABASE_URL:-postgresql://127.0.0.1/fifthtry}

function pushd2() {
    PUSHED=$(pwd)
    cd "${PROJDIR}""$1" >> /dev/null || return
}

function popd2() {
    cd "${PUSHED:-$PROJDIR}" >> /dev/null || return
    unset PUSHED
}


function build-email-wasm() {
    pushd2 "${PROJ_ROOT}/ft-email-auth-provider" || return 1
    # cargo clean
    cargo build --target wasm32-unknown-unknown --release || return 1
    cp ./target/wasm32-unknown-unknown/release/ft_email_auth_provider.wasm . || return 1
    popd2
}