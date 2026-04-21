# fledge-plugin-todo

A [fledge](https://github.com/CorvidLabs/fledge) plugin that scans your codebase for TODO, FIXME, HACK, and XXX comments.

Built in Rust.

## Install

```bash
fledge plugin install CorvidLabs/fledge-plugin-todo
```

## Usage

```bash
# Scan current directory
fledge todo

# Scan a specific directory
fledge todo src/

# JSON output
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

## Requirements

- Rust toolchain (for first build)

## License

MIT
