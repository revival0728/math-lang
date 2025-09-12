# Math Lang

Toy scripting language to calculate math

## How to Play

1. get the `math-lang` binary

### Build binary from source

make sure you have installed `git`, `cmake`, any C++ build system (`make` by default), and any C++ compiler (`g++`, `clang++`, `MSVC`)
```bash
git clone https://github.com/revival0728/math-lang.git
cd math-lang && cmake -B build -S src && cmake --build build --config Release
```

### Download from Release

You can download `math-lang` binary from [Release](https://github.com/revival0728/math-lang/releases) page.

2. run the binary

```bash
./math-lang/build/math-lang
```

## Binary Usage

```
math-lang 
  [Math-Lang-Script File Path (.mls)]
```

## Language Syntax
```
rnum = [raw-number] | [scientific-notation] | [[rnum]^[rnum]]
idnt = [user-defined-variable] | [builtin-constants] | [rnum]
oper = [+][-][*][/]
expr = [idnt] |
       [fun-call] |
       ([expr]) |
       [idnt][=][expr] |
       [expr][oper][expr] |
fun-call = [fun-name]([[expr] | [expr], ...])
```
The scripts execute based on `expr`.

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
| cbrt(x) | `CPP` `<cmath>` function |
| ceil(x) | `CPP` `<cmath>` function |
| floor(x) | `CPP` `<cmath>` function |
| trunc(x) | `CPP` `<cmath>` function |
| round(x) | `CPP` `<cmath>` function |
| pow(a, b) | `CPP` `<cmath>` function |
| exp(x) | `CPP` `<cmath>` function |
| log(x) | `CPP` `<cmath>` function (`log10`) |
| lg(x) | `CPP` `<cmath>` function (`log2`) |
| ln(x) | `CPP` `<cmath>` function (`log` which computes natural logarithm) |
| mod(a, b) | `CPP` `<cmath>` function (`fmod` which computes remainder of division) |

You can checkout [`mathlib.hpp`](/src/mathlib.hpp) for more specific information.

## TODO
- [ ] optimize temporary memory usage
