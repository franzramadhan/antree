.PHONY: dev web serve image check words

WASM := target/wasm32-unknown-unknown/release/antri.wasm

dev:
	cargo run

web:
	cargo build --release --target wasm32-unknown-unknown
	rm -rf dist
	mkdir -p dist
	cp $(WASM) dist/antri.wasm
	cp web/index.html web/mq_js_bundle.js web/sapp_jsutils.js web/quad-storage.js dist/
	cp -r assets dist/assets

serve:
	python3 -m http.server 8080 --directory dist

image:
	docker build -t antri:local .

check: words
	cargo fmt --all -- --check
	cargo clippy --all-targets -- -D warnings
	cargo clippy --target wasm32-unknown-unknown -- -D warnings
	cargo test

# Fail if any real-world reference slipped into the repo.
words:
	@! grep -rniE 'indonesia|\bKTP\b|\bSKCK\b|\bIMB\b|\bNIK\b|dukcapil|puskesmas|satpas|\bRT/RW\b' \
		--exclude-dir=target --exclude-dir=dist --exclude-dir=.git --exclude=Makefile . \
		|| (echo "banned word found" && exit 1)
