# Math Lang

Toy scripting language to calculate math

## How to Play

1. get the `math-lang` binary

### Build binary from source

make sure you have installed `rustc` and `cargo`

using the [rustup]("https://rustup.rs") will be helpful

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

## Builtins

### Constants

| name | value |
|------|----------|
| pi   | 3.14159265358979323846264338327950288 |
| e    | 2.71828182845904523536028747135266250 |

### Functions

| name | document |
|------|----------|
| sin(x) | `CPP` `<cmath>` function |
| cos(x) | `CPP` `<cmath>` function |
| tan(x) | `CPP` `<cmath>` function |
| asin(x) | `CPP` `<cmath>` function |
| acos(x) | `CPP` `<cmath>` function |
| atan(x) | `CPP` `<cmath>` function |
| abs(x) | `CPP` `<cmath>` function |
| sqrt(x) | `CPP` `<cmath>` function |
| ceil(x) | `CPP` `<cmath>` function |
| floor(x) | `CPP` `<cmath>` function |
| round(x) | `CPP` `<cmath>` function |
| exp(x) | `CPP` `<cmath>` function |
| log(x) | `CPP` `<cmath>` function (`log10`) |

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