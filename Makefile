.PHONY: s c r l rel
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
	fish -c "cd client; cargo b --no-default-features --features web --target wasm32-unknown-unknown --bin client"
	wasm-bindgen --no-typescript --target web --out-dir ./out/ --out-name "rots" ./target/wasm32-unknown-unknown/debug/client.wasm
	rsync -r ./client/out/ server.local:john2143.com/pages/
