cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --no-typescript --out-name bevy_game --out-dir wasm --target web target/wasm32-unknown-unknown/release/bevy_jam_5.wasm
cp -r assets wasm/
cd wasm
wasm-opt -Os bevy_game_bg.wasm -o bevy_game_bg.wasm
zip --recurse-paths ../wasm.zip .
cd ..

