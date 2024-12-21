# pest_railroad

[![Crate](https://img.shields.io/crates/v/pest_railroad)](https://crates.io/crates/pest_railroad)
[![Docs](https://docs.rs/pest_railroad/badge.svg)](https://docs.rs/pest_railroad)

Railroad (aka syntax) SVG diagram generator for pest parsers. It supports most (but not all) Pest grammar rules.

# Install

Library crate:

```
cargo add pest_raiload
```

The binary:

```
cargo install pest_railroad_gen
```

# Example

```
cargo run -- grammars/json.pest > json.svg
```

This results in:


<img src="grammars/json.svg" alt="JSON syntax diagram" style="width: 800px; height: auto;">

## Status

This does what I need it to, so it is more or less "finished", but may get support for more Pest rules if I need them. Contributions might be accepted as long as they align to my vision for the tool.
