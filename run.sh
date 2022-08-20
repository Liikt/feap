build() {
	cargo build --release
}

run() {
	cargo run
}

bench() {
	build
	hyperfine -n feap "target/release/benchmark feap" -n rudac "target/release/benchmark rudac"
}

pushd benchmark
$1
popd