# Math Lang

Toy scripting language to calculate math

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

## Language Syntax
```
rnum = [raw-number] | [scientific-notation]
idnt = [user-defined-variable] | [builtins] | [rnum]
b_op = [+][-][*][/][^][=]
s_op = [-]
expr = [idnt] |
       [fun-call] |
       ([expr]) |
       [s_op][expr] |
       [expr][b_op][expr] |
fun-call = [fun-name]([[] | [expr] | [expr], ...])
```
### Breaking Changes
- no more `a^b` rnum, use operator `^` instead
- `=` is now operator
- accept no argument function

You can checkout MLS script exmaples in the [`examples/`](/examples/) folder.

## Type System

types are automatically determined, depends on precision and size of value

```
BigNum > f64 > i64 > i32
```

where `f64`, `i64`, `i32` are Rust primitve types, `BigNum` is used only when precision or size of value is required

`BigNum` is not implemented yet

## Builtins

### Constants

| name | value |
|------|----------|
| pi   | 3.14159265358979323846264338327950288 |
| e    | 2.71828182845904523536028747135266250 |

### Functions

| name | document |
|------|----------|
| sin(x) | `Rust` function |
| cos(x) | `Rust` function |
| tan(x) | `Rust` function |
| asin(x) | `Rust` function |
| acos(x) | `Rust` function |
| atan(x) | `Rust` function |
| abs(x) | `Rust` function |
| sqrt(x) | `Rust` function |
| ceil(x) | `Rust` function |
| floor(x) | `Rust` function |
| round(x) | `Rust` function |
| exp(x) | `Rust` function |
| log(x) | `Rust` function (`log10`) |
| log2(x) | `Rust` function (`log2`) |

- using `f64::builtin(x)` for all Rust primitive types

### Breaking Changes
- remove `trunc(x)` and `cbrt(x)`
- temporary remove `ln(x)` and `mod(a, b)` due to reimplement
- `pow(a, b)` is now `a^b`
- `lg(x)` is now `log2(x)`

## TODO
- [ ] implement BigNum
- [ ] improve error message
- [ ] fix i64 and f64 overflow problem
- [ ] add ln(x) and mod(a, b)
