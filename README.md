# Grimoire

A collection of experiments and playground projects for VST plugin development using Rust.

## Purpose

This repository serves as a sandbox for trying out various ideas, frameworks, and techniques in audio plugin development.

## Projects

- **cantrip_gain**: A simple gain plugin.
- **cantrip_filter**: A simple filter plugin.
- **cantrip_compressor**: A simple compressor plugin.
- **cantrip_delay**: A simple delay plugin.

## Usage

Each project is a workspace member. You can build individual plugins using `cargo xtask`.

Example for `cantrip_gain`:

```bash
cargo xtask bundle cantrip_gain --release
```
