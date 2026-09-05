# Math Lang

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="./assets/file-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="./assets/file-light.svg">
  <img alt="LANG ICON" src="./assets/file-dark.svg">
</picture>


![test & build](https://img.shields.io/github/actions/workflow/status/revival0728/math-lang/build-test-multiplatform.yml)
![GitHub Release](https://img.shields.io/github/v/release/revival0728/math-lang)
![GitHub commits since latest release (branch)](https://img.shields.io/github/commits-since/revival0728/math-lang/latest/dev)
![GitHub Downloads (all assets, all releases)](https://img.shields.io/github/downloads/revival0728/math-lang/total)

Toy scripting language to calculate math

Everything in this language are math expressions

## Features

- Intuitive scripting experience for math
- Eazy to learn with simple language syntax
- With [VSCode language extension](https://github.com/revival0728/math-lang-vsc-extension) support
- Mathematic skills are all matters!

## How to Play

1. get the `math-lang` binary

### Build binary from source

make sure you have installed `git`, `rustc` and `cargo`

using the [rustup](https://rustup.rs) will be helpful

```bash
git clone https://github.com/revival0728/math-lang.git
cd math-lang 
cargo build --profile release
```

### Install binary from source

make sure you have installed `git`, `rustc` and `cargo`

using the [rustup](https://rustup.rs) will be helpful

```bash
git clone https://github.com/revival0728/math-lang.git
cd math-lang 
cargo install --path .
```

### Download from Release

You can download `math-lang` binary from [Release](https://github.com/revival0728/math-lang/releases) page.

2. run the binary

```bash
./target/release/math-lang    # build from source
math-lang                     # install from source
$(DOWNLOAD_FOLDER)/math-lang  # download from release
```

## Binary Usage

```
$ math-lang --help
toy scripting language to calculate math

Usage: math-lang [SOURCE]

Arguments:
  [SOURCE]  Path of source file

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## About the Language
please checkout the [document](/docs/README.md)

## How to Write Rust Library Module
1. clone the repository
2. run `cargo doc --open` to checkout the document

## Development
### Main Changes
- added builtin function `type(x)`
- added builtin function `hash(x)`
- implemented BigNum type
- renamed BigNum type to Real type
- deprecated f64 type

### TODO
- [ ] improve error message
- [ ] fix tail call optimization to recursion
- [ ] add function info expression
- [ ] add custom type support (parsed as None but bytes has meanings)
- [ ] fix cannot access parent scope with same function name (?)
- [ ] flatten inst, flat Inst::Mul with new inst Inst::Jump(cond, dest)
- [ ] package manager
