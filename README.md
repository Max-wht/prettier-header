# prettier-header

`phmx` is a Rust CLI for generating pretty header comments.

## Quick Start

```bash
cargo run -- "Auction Participation"
```

This always prints the generated header to terminal and also tries to copy it to macOS clipboard (`pbcopy`).

## Usage

```bash
phmx [OPTIONS] <TITLE>
phmx -l | --list
```

Options:

- `-p, --pattern <ID>`: pattern ID, only `1` or `2` (default: `1`)
- `-l, --list`: show pattern list with preview
- `-h, --help`: show help

## Examples

```bash
phmx "Auction Participation"
phmx -p 2 "Auction Participation"
phmx --list
```
