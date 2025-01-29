sh -c "$(curl -fsSL https://fastn.com/install.sh)"

sh ./scripts/build-wasm.sh
sh ./scripts/optimise-wasm.sh

rm .gitignore

echo .packages > .gitignore
echo .fastn >> .gitignore
echo .is-local >> .gitignore

cd lets-auth.fifthtry.site/ && fastn upload lets-auth
