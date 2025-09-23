# Developer convenience targets

.PHONY: build run watch fmt clippy check db-up db-down migrate

build:
	@cargo build

run:
	@cargo run

watch:
	@cargo watch -x run

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
