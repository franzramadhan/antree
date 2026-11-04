.PHONY: dev web serve image check words sprites music

WASM := target/wasm32-unknown-unknown/release/antree.wasm

dev:
	cargo run

web:
	cargo build --release --target wasm32-unknown-unknown
	rm -rf dist
	mkdir -p dist
	cp $(WASM) dist/antree.wasm
	cp web/index.html web/mq_js_bundle.js web/sapp_jsutils.js web/quad-storage.js web/boot.js dist/
	cp -r assets dist/assets

# Regenerate the pixel art sheets in assets/sprites (needs Pillow).
sprites:
	python3 tools/sprites.py

# Rewrite the background song, assets/music/hold.mid.
music:
	python3 tools/music.py

serve:
	python3 -m http.server 8080 --directory dist

image:
	./build.sh && docker build --platform linux/amd64 -t antree:local .

check: words
	cargo fmt --all -- --check
	cargo clippy --all-targets -- -D warnings
	cargo clippy --target wasm32-unknown-unknown -- -D warnings
	cargo test
	cargo fmt --manifest-path server/Cargo.toml -- --check
	cargo clippy --manifest-path server/Cargo.toml --target-dir target/server --all-targets -- -D warnings
	cargo test --manifest-path server/Cargo.toml --target-dir target/server

# Fail if any real-world reference slipped into the repo.
words:
	@! grep -rniE 'indonesia|\bKTP\b|\bSKCK\b|\bIMB\b|\bNIK\b|dukcapil|puskesmas|satpas|\bRT/RW\b|polsek|kelurahan|lurah|camat|kabid|loket|satpam|honorer|imigrasi|npwp|samsat|\bSPT\b|\bSTNK\b|\bBPKB\b|\bKUA\b|\bKPP\b|\bNIB\b' \
		--exclude-dir=target --exclude-dir=dist --exclude-dir=.git --exclude=Makefile . \
		|| (echo "banned word found" && exit 1)
