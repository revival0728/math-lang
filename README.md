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
vars = [user-defined-variable] | [builtin-constants]
idnt = [vars] | [rnum]
b_op = [+][-][*][/][^][=][mod]
s_op = [-]
expr = [idnt] |
       [fun-call] |
       ([expr]) |
       [s_op][expr] |
       [rnum][[expr]/[rnum]]
       [vars][[expr]/[rnum]]
       [expr][b_op][expr] |
fun-call = [fun-name]([[] | [expr] | [expr], ...])
```
### Breaking Changes
- no more `a^b` rnum, use operator `^` instead
- `=` is now operator
- accept no argument function

You can checkout MLS script exmaples in the [`examples/`](/examples/) folder.

## About Variable Name
the variable name must matches regex `([_]|[A-Z]|[a-z])([_]|[A-Z]|[a-z]|[0-9])*` which `n`, `_n`, `n1`, `_n1`, `N`, ... are all valid

### NOTICE
you should never use variable name with format `__[name]__`, it is used by ENV and compiler

## About Operator
### Precedence
```
(s_op)(-) > (^) > (/) = (*) > (b_op)(-) = (+) > (mod) > (=)
```

### Notice
- `mod`: calculates euclid remainder, which is `r` and `0 <= r < abs(rhs)`
- `*`: auto insertion bewteen
  1. number and variable
  2. variable and variable

## Type System
types are automatically determined, depends on precision and size of value

```
BigNum > f64 > i64 > i32
```

where `f64`, `i64`, `i32` are Rust primitve types, `BigNum` is used only when precision or size of value is required

### Value Limit
currently values are limited by Rust primitive type `f64`, will change after implementing `BigNum`

which is in [-1.7976931348623157e+308, 1.7976931348623157e+308]

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
| ln(x) | `Rust` function (`log(x, std::f64::consts::E)`) |
| trunc(x) | `Rust` function, return integer part of the argument |
| cbrt(x) | `Rust` function, return cube root of the argument |

- using `f64::builtin(x)` for all Rust primitive types

### Breaking Changes
- remove `mod(a, b)`, use `a mod b` instead
- remove `pow(a, b)`, use `a^b` instead
- rename `lg(x)` to `log2(x)`

## About Environment Variable (ENV)
it is possible to read or get ENV by function `__[ENV_name]__()`, where `[ENV_name]` only contains lower case alphabets

- the function always returns current ENV value
- function with argument sets current ENV to the argument value

| ENV  | document |
|------|----------|
| PRECISION | the precision of float output |


## TODO
- [ ] implement BigNum, fix i64 and f64 overflow problem
- [ ] improve error message
