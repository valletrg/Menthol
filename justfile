test:
	cargo test --workspace

test-proto:
	cargo test -p slsk-proto

run:
	cargo run -p slsk-gui

update-spec:
	curl -s "https://raw.githubusercontent.com/nicotine-plus/nicotine-plus/master/doc/SLSKPROTOCOL.md" \
		> doc/SLSKPROTOCOL.md
	echo "Updated SLSKPROTOCOL.md"

lint:
	cargo clippy --workspace -- -D warnings

fmt:
	cargo fmt --all
