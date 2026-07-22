# Math Lang

Toy scripting language to calculate math

Everything in this language are math expressions

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
b_op = [+][-][*][/][^][=][mod][:]
s_op = [-]
expr = [idnt] |
       [fun-call] |
       ([expr]) |
       [s_op][expr] |
       [rnum][[expr]/[rnum]]
       [vars][[expr]/[rnum]]
       [expr][b_op][expr] |
fun-call = [fun-name]([[] | [expr] | [expr], ...])
def-fun = ([fun-name]([[] | [var] | [var], ...]) = [expr])
```

- compiler parses the source line by line
- you can checkout MLS script exmaples in the [`examples/`](/examples/) folder.

## About Variable and Function Name
the variable or function name must matches regex `([_]|[A-Z]|[a-z])([_]|[A-Z]|[a-z]|[0-9])*` which `n`, `_n`, `n1`, `_n1`, `N`, ... are all valid

### NOTICE
you should never use variable name with format `__[name]__`, it is used by ENV, compiler and runtime

## About Operator
### Precedence
```
(s_op)(-) > (:) > (^) > (/) = (*) > (b_op)(-) = (+) > (mod) > (=)
```

### Notice
- `:`: is for indexing an array
- `mod`: calculates euclid remainder, which is `r` and `0 <= r < abs(rhs)`
- `*`: auto insertion bewteen
  - number and variable
  - variable and variable
  - right paren and left paren
  - number and left paren

## Type System
types are automatically determined, depends on precision and size of value

```
Sequence | BigNum > f64 > i64 > i32
```

where `f64`, `i64`, `i32` are Rust primitve types, `BigNum` is used only when precision or size of value is required

`BigNum` is not implemented yet

### Value Limit
currently values are limited by Rust primitive type `f64`, will change after implementing `BigNum`

which is in [-1.7976931348623157e+308, 1.7976931348623157e+308]

### About Sequence
- just like array in other language
- create a sequence with function `Sequence(len)`
- indexing a sequence with operator `:`
- you can chose to use 0-base or 1-base

checkout the [example](/examples/sequence.mls)

## About Comment
use char `#` to comment something, check out example [here](/examples/formattive.mls)

## About Module System
- use `import()` to import module
- module path is Unix like, independant to OS, relative to main script file path
- `import()` checks for duplicate function name
- `import()` WON'T checks for duplicate variable name
- scripts in module will be executed, variables and functions in global scope of module will be imported to parent script
- modules cannot produce outputs

checkout [example](/examples/module.mls)

## Builtins
### Constants

| name | value |
|------|----------|
| pi   | 3.14159265358979323846264338327950288 (std::f64::consts::PI) |
| e    | 2.71828182845904523536028747135266250 (std::f64::consts::E)|

### Special Functions

| name | document |
|------|----------|
| $(x) | always return `None`, usually uses as entry point |
| .(x) | always return `1`, usually to connect expressions |

- special functions allows you to write formattive code
- the compiler still parses it as single line
- check out example [here](/examples/formattive.mls)

> In math, it just all ONEs, or should I say only ONE?

### Logic Functions

| name | document |
|------|----------|
| if(x) | return `x == 0`  where `x` must be an integer |
| else(x) | return `x != 0` where `x` must be an integer |
| sign(x) | return `1` if `x > 0`, `0` if `x == 0`, `-1` if `x < 0`, using Rust [`f64::total_cmp`](https://doc.rust-lang.org/stable/std/primitive.f64.html#method.total_cmp) to compare |

- it works because runtime will check `lhs == 0`, if true then will not calcuate `rhs`

### Output Functions

| name | document |
|------|----------|
| print(x) | output `x` |
| println(x) | output `x` and a newline |

- outputs by output functions will be buffered until calling `println()` or reached end of source
- source in REPL is single line
- source in file is entire file
- output functions always output data of variables (including sequences)

### Sequence Relative Functions

| name | document |
|------|----------|
| Sequence(len) | return a sequence of length `len` with elements set to 0 |
| len(seq) | return the length of `seq`(must be Sequence) |

### Module Relative Functions

| name | document |
|------|----------|
| import(module) | import the module from Unix like path `module` which is relative to main script file path |

### Math Functions

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

## About Environment Variable (ENV)
it is possible to read or get ENV by function `__[ENV_name]__()`, where `[ENV_name]` only contains lower case alphabets

- the function always returns current ENV value
- function with argument sets current ENV to the argument value

| ENV  | document | default | constraint |
|------|----------| --------| ---------- |
| PRECISION | the precision of float output | 7 | [0, 15] |
| PRINT_SET_INST | to print `set` intruction result or not | repl=1, source=0 | [0, 1] |
| DETAIL_DEPTH | the output level of detail information | repl=0, source=1 |[0, 1] |
| MAX_STACK_DEPTH | the maximum stack depth of recrusive function | 512 |[0, inf] |
| INDEX_BASE | the base of indexing | 0 | [0, 1] |

- maximum stack size is not ENV, but can be configured by execution argument, by default is 64MB

## About String literal
- string literals cannot be stored in variable, parsed as None in this language
- use it to have better output
- escape chars are not supported

## Development
### Main Changes
- support return value of sequence
- added builtin function len() to get length of sequence
- rename init functions to `Sequence Relative Functions`

### Fixed Bugs
- import information output wrong module name
- import of sequence only import first demension
- return of sequence only return first demension
- alone right paren caused crashing

### TODO
- [ ] implement BigNum, fix i64 and f64 overflow problem (rarely happened)
- [ ] improve error message
- [ ] fix tail call optimization to recursion