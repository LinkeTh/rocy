# Developer convenience targets

.PHONY: build run watch fmt clippy check db-up db-down migrate

build:
	cd ./web && bun run build && cd .. && cargo build --release
#	cd ./web && bun run build

run:
	cd ./web && bun run build && cd .. && cargo run --release
	@#cargo run& cd ./web && bun run build

watch:
	cd ./web && bun run build && cd .. && cargo watch -x run
	@#cargo watch -x run& cd ./web && bun start

fmt:
	@cargo fmt --all

clippy:
	@cargo clippy --all-targets -- -D warnings

check:
	@cargo check

db-up:
	@docker compose up -d db

db-down:
	@docker compose down

migrate:
	@cargo sqlx migrate run
