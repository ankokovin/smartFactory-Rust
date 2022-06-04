cd smart-factory-wasm-port
echo "Starting tests"
cargo test
echo "Compiling to wasm"
wasm-pack build --scope rycarok . --release -t web
echo "Publishing to npm"
cd pkg
npm publish --access=public