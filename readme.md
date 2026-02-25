# tasks

A minimal command-line task manager written in Rust.

## Usage
```bash
tasks add "My task" --description "Optional description"
tasks list
tasks remove 1
tasks toggle 1
```

## Built with

- [Rust](https://www.rust-lang.org/)
- [SQLx](https://github.com/launchbadge/sqlx) — async SQLite
- [Clap](https://github.com/clap-rs/clap) — CLI parsing
- [Colored](https://github.com/mackwic/colored) — terminal colors
- [Tokio](https://tokio.rs/) — async runtime

## Installation
```bash
cargo install --path .
```
