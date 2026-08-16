# Math Lang

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

make sure you have installed `rustc` and `cargo`

using the [rustup](https://rustup.rs) will be helpful

```bash
git clone https://github.com/revival0728/math-lang.git
cd math-lang 
cargo build --profile release
```

### Download from Release

You can download `math-lang` binary from [Release](https://github.com/revival0728/math-lang/releases) page.

2. run the binary

```bash
./target/release/math-lang
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
- massively improved recursion performance
- support rust library module
- added `import_rlib()`
- added new operator `@`, for accessing and declaring variables only in current scope

### Details Without Impact
Builtin functions are now has two types, `rust function` and `runtime function`. Only `Module Relative Functions` and `Output Functions` are `runtime function`, the remainings are `rust function`. `rust function` works the same as external rust module while `runtime function` works the same as old builtin functions. The key difference between two types of builtins is that `runtime function` is dependent to the runtime and can be called directly from Rust while `rust function` is independent which works just like a math function. This change will not has actual impact on the execution.

### TODO
- [ ] Change Cargo.toml version
- [ ] implement BigNum, fix i64 and f64 overflow problem (rarely happened)
- [ ] improve error message
- [ ] fix tail call optimization to recursion
- [ ] add type(), hash()
- [ ] add function info expression
- [ ] add custom type support (parsed as None but bytes has meanings)
- [ ] fix cannot access parent scope with same function name (?)
- [ ] flatten inst, flat Inst::Mul with new inst Inst::Jump(cond, dest)
