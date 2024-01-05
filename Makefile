.PHONY: s c r l rel rsync webfast web
rel:
	cargo r --release --features bevy/dynamic_linking --bin client
m:
	cargo r --features bevy/dynamic_linking --bin client -- --autoconnect main
mr:
	cargo r --release --features bevy/dynamic_linking --bin client -- --autoconnect main
s:
	cargo r --features bevy/dynamic_linking --bin server
c:
	cargo r --features bevy/dynamic_linking --bin client -- --autoconnect local
r:
	cargo r --features bevy/dynamic_linking --bin client -- --autoconnect local -n R
l:
	cargo r --features bevy/dynamic_linking --bin client -- --autoconnect local -n L

rr:
	cargo r --release --features bevy/dynamic_linking --bin client -- --autoconnect local -n R
lr:
	cargo r --release --features bevy/dynamic_linking --bin client -- --autoconnect local -n L


web:
	# requires wasm-bindgen-cli + binaryen
	bash -c "cd client; RUSTFLAGS=--cfg=web_sys_unstable_apis cargo b --profile release-web --no-default-features --features web --target wasm32-unknown-unknown --bin client"
	wasm-bindgen --no-typescript --target web --out-dir ./wasm_dist/ --out-name "rots" ./target/wasm32-unknown-unknown/debug/client.wasm

webfast: web
	mv ./wasm_dist/rots_bg.wasm ./wasm_dist/rots_bg.big.wasm
	wasm-opt -Oz -o ./wasm_dist/rots_bg.wasm ./wasm_dist/rots_bg.big.wasm

sync: web
	rsync -r ./wasm_dist/ server.local:john2143.com/rots

syncfast: webfast
	rsync -r ./wasm_dist/ server.local:john2143.com/rots
