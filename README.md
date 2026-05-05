# fledge-plugin-todo

Fledge plugin: scan your codebase for TODO/FIXME/HACK/XXX comments. Built in Rust.

A [fledge](https://github.com/CorvidLabs/fledge) plugin that recursively walks
your project and reports every `TODO`, `FIXME`, `HACK`, and `XXX` comment it
finds. Output is available as a human-readable table or as JSON for
machine consumption.

## Install

```bash
fledge plugin install corvid-agent/fledge-plugin-todo
```

## Usage

```bash
# Scan current directory
fledge todo

# Scan a specific directory
fledge todo src/

# JSON output (for piping to other tools)
fledge todo --json
```

## Example Output

```
Found 5 items:
  TODO: 3
  FIXME: 2

  src/main.rs:42 [TODO] implement error handling
  src/lib.rs:18 [FIXME] this breaks on Windows
  src/lib.rs:99 [TODO] add caching
  tests/mod.rs:7 [TODO] add edge case tests
  src/main.rs:55 [FIXME] race condition
```

## Markers

| Marker  | Meaning                         |
|---------|---------------------------------|
| `TODO`  | Planned work                    |
| `FIXME` | Known bug or broken behaviour   |
| `HACK`  | Workaround that should be fixed |
| `XXX`   | Needs attention / review        |

## Skipped Directories

The scanner automatically skips directories that typically contain generated or
third-party code:

`.git`, `node_modules`, `target`, `.build`, `vendor`, `dist`, `__pycache__`

## Requirements

- Rust toolchain (the plugin compiles itself on first run via `bin/build.sh`)

## Development

```bash
cargo build
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

## License

MIT
