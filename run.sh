build() {
	cargo build --release
}

run() {
	cargo run
}

bench() {
	build
	hyperfine \
		-n feap "target/release/benchmark feap" \
		-n rudac "target/release/benchmark rudac" \
		-n ytoml "target/release/benchmark ytoml" \
		-n compprog "target/release/benchmark compprog"
}

pushd benchmark
$1
popd