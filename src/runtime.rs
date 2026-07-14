use crate::comiler::{Compiler, Expr, Inst};
use crate::env::*;
use crate::error::{GlobalError, RuntimeError};
use crate::var::{Var, VarType};
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::Into;
use std::rc::Rc;

const PI: f64 = std::f64::consts::PI;
const E: f64 = std::f64::consts::E;

#[derive(Debug, Clone, Default)]
pub struct Fun<'input> {
    para_name: Vec<&'input str>,
    data: Vec<Inst<'input>>,
}

#[derive(Debug, Default, Clone)]
pub struct Scope<'input> {
    name: &'input str,
    stack_depth: u32,
    var_table: HashMap<&'input str, Rc<RefCell<Var>>>,
    fun_table: HashMap<&'input str, Rc<RefCell<Fun<'input>>>>,
}

#[derive(Debug, Default, Clone)]
pub struct Runtime<'input> {
    builtin: Scope<'input>,
    locals: Vec<Scope<'input>>,
    output: Vec<String>,
}

impl<'input> Scope<'input> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn get_var(&self, name: &'input str) -> Option<Rc<RefCell<Var>>> {
        self.var_table.get(name).cloned()
    }
    pub fn get_fun(&self, name: &'input str) -> Option<Rc<RefCell<Fun<'input>>>> {
        self.fun_table.get(name).cloned()
    }
    pub fn add_var(&mut self, name: &'input str, value: Var) {
        self.var_table.insert(name, Rc::new(RefCell::new(value)));
    }
    pub fn add_fun(&mut self, name: &'input str, para_name: Vec<&'input str>) {
        self.fun_table
            .insert(name, Rc::new(RefCell::new(Fun::new(para_name))));
    }
    pub fn has_var(&self, name: &'input str) -> bool {
        self.var_table.contains_key(name)
    }
    pub fn has_fun(&self, name: &'input str) -> bool {
        self.fun_table.contains_key(name)
    }
    pub fn set_fun(&mut self, name: &'input str, fun: Fun<'input>) {
        self.fun_table.insert(name, Rc::new(RefCell::new(fun)));
    }
    pub fn add_ref_var(&mut self, name: &'input str, ref_var: Rc<RefCell<Var>>) {
        self.var_table.insert(name, ref_var);
    }
}

impl<'input> Fun<'input> {
    pub fn new(para_name: Vec<&'input str>) -> Self {
        Self {
            para_name,
            data: Vec::new(),
        }
    }
    pub fn push_inst(&mut self, inst: Inst<'input>) {
        self.data.push(inst);
    }
}

impl<'input> Runtime<'input> {
    pub fn new() -> Self {
        let mut runtime = Self::default();

        runtime.builtin.name = "__builtin__";
        // add builtin constant
        runtime.builtin.add_var("pi", Var::from(PI));
        runtime.builtin.add_var("e", Var::from(E));
        runtime.builtin.add_var("0", Var::from(0));
        runtime.builtin.add_var("1", Var::from(1));
        runtime
            .builtin
            .add_var("None", Var::from_string(String::new()));

        // add builtin functions
        // NOTE: need to handle exec_inst()::BuiltinFnCall(_)
        macro_rules! group_to_literal {
            ( $($arg:ident),* ) => {
                concat!( $( stringify!($arg) ),* )
            };
            ( $($arg:tt)* ) => {
                concat!( $( stringify!($arg) ),* )
            };
        }
        macro_rules! add_builtin_fn {
            ($name:ident($($para:ident),*)) => {
                runtime.builtin.set_fun(
                    stringify!($name),
                    Fun {
                        para_name: vec![group_to_literal!($($para),*)],
                        data: vec![Inst::BultinFnCall(stringify!($name))],
                    },
                )
            };
        }
        // IO functions
        add_builtin_fn! { print(x) };
        // math functions
        add_builtin_fn! { sin(x) };
        add_builtin_fn! { cos(x) };
        add_builtin_fn! { tan(x) };
        add_builtin_fn! { asin(x) };
        add_builtin_fn! { acos(x) };
        add_builtin_fn! { atan(x) };
        add_builtin_fn! { abs(x) };
        add_builtin_fn! { sqrt(x) };
        add_builtin_fn! { ceil(x) };
        add_builtin_fn! { floor(x) };
        add_builtin_fn! { round(x) };
        add_builtin_fn! { exp(x) };
        add_builtin_fn! { log(x) };
        add_builtin_fn! { log2(x) };
        add_builtin_fn! { ln(x) };
        add_builtin_fn! { trunc(x) };
        add_builtin_fn! { cbrt(x) };
        // env functions
        add_builtin_fn! { __precision__(x) }
        add_builtin_fn! { __detail_depth__(x) }
        add_builtin_fn! { __max_stack_depth__(x) }
        add_builtin_fn! { __print_set_inst__(x) }
        // logic functions
        add_builtin_fn! { if(x) }
        add_builtin_fn! { else(x) }
        // special functions
        runtime.builtin.set_fun(
            "$",
            Fun {
                para_name: vec!["x"],
                data: vec![Inst::BultinFnCall("$")],
            },
        );
        runtime.builtin.set_fun(
            ".",
            Fun {
                para_name: vec!["x"],
                data: vec![Inst::BultinFnCall(".")],
            },
        );

        // add global scope
        let mut global = Scope::default();
        global.name = "__global__";
        global.stack_depth = 0;
        runtime.locals.push(global);

        runtime
    }
    pub fn execute(&mut self, source: &'input str) -> Result<&Vec<String>, GlobalError> {
        let mut compiler = Compiler::new(source);
        let ast = match compiler.compile() {
            Ok(ast) => ast,
            Err(ce) => return Err(GlobalError::CE(ce)),
        };
        for (idx, inst) in ast.iter().enumerate() {
            let to_print = if let &Inst::Set(_, _) = inst {
                unsafe { PRINT_SET_INST == 1 }
            } else {
                true
            };
            let output = match self.exec_inst(inst, idx) {
                Ok(val) => val,
                Err(ce) => return Err(GlobalError::RE(ce)),
            };
            let out_str = output.borrow().to_string();
            if to_print && !out_str.is_empty() {
                self.output.push(out_str);
            }
        }
        Ok(&self.output)
    }
    fn exec_fun(
        &mut self,
        fun: &Fun<'input>,
        line: usize,
    ) -> Result<Rc<RefCell<Var>>, RuntimeError> {
        unsafe {
            if self.locals.last().unwrap().stack_depth > MAX_STACK_DEPTH {
                let max_stack_depth = MAX_STACK_DEPTH;
                self.locals.last_mut().unwrap().stack_depth = 0;
                return Err(RuntimeError {
                    line,
                    msg: format!("reached maximum stack depth {}", max_stack_depth),
                });
            }
        }
        for inst in fun.data.iter() {
            let ret = self.exec_inst(inst, line)?;
            self.locals
                .last_mut()
                .unwrap()
                .add_ref_var("__return__", ret);
        }
        match self.locals.last().unwrap().get_var("__return__") {
            Some(val) => Ok(val),
            None => panic!("runtime internal error!"),
        }
    }
    fn exec_expr(
        &mut self,
        expr: &Expr<'input>,
        line: usize,
    ) -> Result<Rc<RefCell<Var>>, RuntimeError> {
        match expr {
            Expr::None => panic!("compiler internal error! (in runtime)"),
            Expr::Inst(sub_inst) => self.exec_inst(sub_inst.as_ref(), line),
            Expr::FunCall(name, args) => {
                let fun = {
                    let mut gfn = None;
                    for scope in self.locals.iter().rev() {
                        if let Some(efn) = scope.get_fun(name) {
                            gfn = Some(efn);
                            break;
                        }
                    }
                    if gfn.is_none() {
                        gfn = self.builtin.get_fun(name);
                    }
                    match gfn {
                        Some(gfn) => gfn,
                        None => {
                            return Err(RuntimeError {
                                line,
                                msg: format!(
                                    "{}() is an undefined function (could be variable)",
                                    name
                                ),
                            });
                        }
                    }
                };
                let fun_para_len = fun.borrow().para_name.len();
                let arg_len = args.len();
                let is_name_env = is_env(name);
                if is_name_env && arg_len > fun_para_len || !is_name_env && fun_para_len != arg_len
                {
                    return Err(RuntimeError {
                        line,
                        msg: format!(
                            "function {}() expect {} arguments got {}",
                            name,
                            fun.borrow().para_name.len(),
                            args.len()
                        ),
                    });
                }

                let mut sub_local = Scope::new();
                sub_local.name = name;
                sub_local.stack_depth = self.locals.last().unwrap().stack_depth + 1;
                for (pname, expr) in fun.borrow().para_name.iter().zip(args.iter()) {
                    let value = self.exec_expr(expr, line)?;
                    sub_local.add_ref_var(pname, value);
                }
                self.locals.push(sub_local);
                let rval = self.exec_fun(&fun.borrow(), line)?;
                self.locals.pop();
                Ok(rval)
            }
            Expr::Number(data) => {
                if let Some(evar) = self.builtin.get_var(data) {
                    Ok(evar)
                } else if let Some(num) = Var::new(data) {
                    self.builtin.add_var(data, num);
                    Ok(self.builtin.get_var(data).unwrap())
                } else {
                    return Err(RuntimeError {
                        line,
                        msg: format!("{} is invalid number (too large or too small)", data),
                    });
                }
            }
            Expr::Var(name) => {
                let mut var = None;
                let mut found = false;
                for scope in self.locals.iter_mut().rev() {
                    if found {
                        break;
                    }
                    if let Some(evar) = scope.get_var(name) {
                        found = true;
                        var = Some(evar);
                    }
                }
                if !found {
                    if let Some(bvar) = self.builtin.get_var(name) {
                        var = Some(bvar);
                    } else {
                        return Err(RuntimeError {
                            line,
                            msg: format!("{} is undefined variable", name),
                        });
                    }
                }
                Ok(var.expect("runtime internal error!"))
            }
        }
    }
    fn exec_inst(
        &mut self,
        inst: &Inst<'input>,
        line: usize,
    ) -> Result<Rc<RefCell<Var>>, RuntimeError> {
        macro_rules! check_none {
            ($x:ident) => {{
                if $x.borrow().type_ == VarType::None {
                    return Err(RuntimeError {
                        line,
                        msg: format!("using None type for operand or argument"),
                    });
                }
            }};
            ($arg:ident $(, $args:ident),*) => {{
                check_none!($arg);
                check_none!($($args),*);
            }};
        }
        macro_rules! handle_binary {
            ($lhs:ident $op:tt $rhs:ident) => {{
                let lhs = self.exec_expr($lhs, line)?;
                let rhs = self.exec_expr($rhs, line)?;
                check_none!(lhs, rhs);
                let eval = Rc::new(RefCell::new(&*lhs.borrow() $op &*rhs.borrow()));
                Ok(eval)
            }};
        }
        match inst {
            Inst::None => panic!("compiler internal error (in runtime)"),
            Inst::Expr(expr) => self.exec_expr(expr, line),
            Inst::Add(lhs, rhs) => handle_binary!(lhs + rhs),
            Inst::Sub(lhs, rhs) => handle_binary!(lhs - rhs),
            Inst::Div(lhs, rhs) => handle_binary!(lhs / rhs),
            Inst::Mod(lhs, rhs) => handle_binary!(lhs % rhs),
            Inst::Mul(lhs, rhs) => {
                let lhs = self.exec_expr(lhs, line)?;
                check_none!(lhs);
                // check lhs is zero for logic function
                if lhs.borrow().type_ <= VarType::I64 {
                    let lvalue: i64 = (&*lhs.borrow()).into();
                    if lvalue == 0 {
                        return Ok(lhs);
                    }
                }
                let rhs = self.exec_expr(rhs, line)?;
                check_none!(rhs);
                let eval = Rc::new(RefCell::new(&*lhs.borrow() * &*rhs.borrow()));
                Ok(eval)
            }
            Inst::Set(lhs, rhs) => match lhs {
                Expr::None => panic!("compiler internal error (in runtime)"),
                Expr::Var(name) => {
                    let rhs = self.exec_expr(rhs, line)?;
                    if rhs.borrow().type_ == VarType::None {
                        return Err(RuntimeError {
                            line,
                            msg: format!(
                                "cannot assign None to a variable, usually caused by assigning function definition expression to a variable"
                            ),
                        });
                    }
                    if self.builtin.has_var(name) {
                        return Err(RuntimeError {
                            line,
                            msg: format!("overriding builtin constant {}", name),
                        });
                    }
                    for scope in self.locals.iter().rev() {
                        if let Some(var) = scope.get_var(name) {
                            *var.borrow_mut() = Var::clone(&rhs.borrow());
                            return Ok(var);
                        }
                    }
                    self.locals
                        .last_mut()
                        .unwrap()
                        .add_var(name, Var::clone(&rhs.borrow()));
                    Ok(self.locals.last().unwrap().get_var(name).unwrap())
                }
                Expr::Inst(_) => Err(RuntimeError {
                    line,
                    msg: format!("cannot assign value to an expression"),
                }),
                Expr::Number(_) => Err(RuntimeError {
                    line,
                    msg: format!("cannot assign value to numbers"),
                }),
                Expr::FunCall(name, param) => {
                    if self.builtin.has_fun(name) {
                        Err(RuntimeError {
                            line,
                            msg: format!("overriding builtin constant {}", name),
                        })
                    } else {
                        let mut para_name = vec![];
                        for p in param {
                            match p {
                                Expr::None => panic!("runtime internal error (in compiler)"),
                                Expr::Var(pname) => para_name.push(*pname),
                                _ => {
                                    return Err(RuntimeError {
                                        line,
                                        msg: format!("function paramter only accpets variable"),
                                    });
                                }
                            }
                        }
                        // TODO: performance issue here ?
                        self.locals.last_mut().unwrap().set_fun(
                            name,
                            Fun {
                                para_name,
                                data: vec![Inst::Expr(rhs.clone())],
                            },
                        );
                        Ok(Rc::new(RefCell::new(Var::from_string(unsafe {
                            // SAFE: no multiple threads
                            if DETAIL_DEPTH == 1 {
                                format!("<defined function {}>", name)
                            } else {
                                format!("")
                            }
                        }))))
                    }
                }
            },
            Inst::Neg(expr) => {
                let val = self.exec_expr(expr, line)?;
                Ok(Rc::new(RefCell::new(-&*val.borrow())))
            }
            Inst::Pow(lhs, rhs) => {
                let lhs = self.exec_expr(lhs, line)?;
                let rhs = self.exec_expr(rhs, line)?;
                if lhs.borrow().type_ <= VarType::I64 && rhs.borrow().type_ <= VarType::I64 {
                    let mut lhs = Rc::new(Var::clone(&lhs.borrow()));
                    let mut rhs: i64 = (&*rhs.borrow()).into();
                    let mut reci = false;
                    if rhs < 0 {
                        rhs = -rhs;
                        reci = true;
                    }
                    let mut ret = Var::from(1);
                    while rhs > 0 {
                        if rhs & 1 == 1 {
                            ret = &ret * &lhs;
                        }
                        *Rc::get_mut(&mut lhs).unwrap() = &*lhs * &*lhs;
                        rhs >>= 1;
                    }
                    if reci {
                        let one = Var::new("1").unwrap();
                        *Rc::get_mut(&mut lhs).unwrap() = &one / &lhs;
                    }
                    Ok(Rc::new(RefCell::new(ret)))
                } else {
                    let lhs: f64 = (&*lhs.borrow()).into();
                    let rhs: f64 = (&*rhs.borrow()).into();
                    let val = lhs.powf(rhs);
                    Ok(Rc::new(RefCell::new(Var::from(val))))
                }
                //TODO: add BigNum implemntation
            }
            Inst::BultinFnCall(name) => {
                macro_rules! handle_arg_1 {
                    ($rust_fn:ident $(, $default_args:expr),*) => {{
                        let scope = self.locals.last_mut().expect("runtime internal error!");
                        let Some(x) = scope.get_var("x") else {
                            panic!("runtime internal error!")
                        };
                        check_none!(x);
                        if x.borrow().type_ <= VarType::F64 {
                            let x: f64 = (&*x.borrow()).into();
                            Ok(Rc::new(RefCell::new(Var::from(x.$rust_fn($($default_args),*)))))
                        } else {
                            //TODO: add BigNum implemntation
                            todo!("add BigNum implementation")
                        }
                    }};
                }
                macro_rules! handle_integer_env_fun {
                    ($env_var:ident, $limit:expr) => {{
                        let scope = self.locals.last_mut().expect("runtime internal error!");
                        if let Some(x) = scope.get_var("x") {
                            check_none!(x);
                            if x.borrow().type_ >= VarType::F64 {
                                return Err(RuntimeError {
                                    line,
                                    msg: format!(
                                        "cannot config {} with float number",
                                        stringify!($env_var)
                                    ),
                                });
                            }
                            // NOTE: will break on 128 bit target?
                            let value: i32 = (&*x.borrow()).into();
                            let value: u32 = match value.try_into() {
                                Ok(v) => v,
                                Err(_) => {
                                    return Err(RuntimeError {
                                        line,
                                        msg: format!(
                                            "cannot config {} with negative number",
                                            stringify!($env_var)
                                        ),
                                    });
                                }
                            };
                            if value > $limit {
                                return Err(RuntimeError {
                                    line,
                                    msg: format!(
                                        "env({}) must less than {}",
                                        stringify!($env_var),
                                        $limit
                                    ),
                                });
                            }
                            unsafe {
                                $env_var = value;
                                Ok(Rc::new(RefCell::new(Var::from($env_var as i64))))
                            }
                        } else {
                            // NOTE: will break on 128 bit target?
                            unsafe { Ok(Rc::new(RefCell::new(Var::from($env_var as i64)))) }
                        }
                    }};
                }
                macro_rules! handle_logic_fun {
                    (x $logic_oper:tt $stand:literal) => {{
                        let scope = self.locals.last_mut().expect("runtime internal error!");
                        let Some(x) = scope.get_var("x") else {
                            panic!("runtime internal error!")
                        };
                        check_none!(x);
                        if x.borrow().type_ > VarType::I64 {
                            return Err(RuntimeError {
                                line,
                                msg: format!("logic functions only accepts integer as argument"),
                            });
                        }
                        let x: i64 = (&*x.borrow()).into();
                        let result = (x $logic_oper $stand) as i32;
                        Ok(Rc::new(RefCell::new(Var::from(result))))
                    }};
                }
                macro_rules! handle_special_fun {
                    ($value:expr) => {{ Ok(self.builtin.get_var(stringify!($value)).unwrap()) }};
                }
                match name {
                    &"sin" => handle_arg_1!(sin),
                    &"cos" => handle_arg_1!(cos),
                    &"tan" => handle_arg_1!(tan),
                    &"asin" => handle_arg_1!(asin),
                    &"acos" => handle_arg_1!(acos),
                    &"atan" => handle_arg_1!(atan),
                    &"abs" => handle_arg_1!(abs),
                    &"sqrt" => handle_arg_1!(sqrt),
                    &"ceil" => handle_arg_1!(ceil),
                    &"floor" => handle_arg_1!(floor),
                    &"round" => handle_arg_1!(round),
                    &"exp" => handle_arg_1!(exp),
                    &"log" => handle_arg_1!(log10),
                    &"log2" => handle_arg_1!(log2),
                    &"ln" => handle_arg_1!(log, E),
                    &"trunc" => handle_arg_1!(trunc),
                    &"cbrt" => handle_arg_1!(cbrt),
                    &"__precision__" => handle_integer_env_fun!(PRECISION, 15),
                    &"__detail_depth__" => handle_integer_env_fun!(DETAIL_DEPTH, 1),
                    &"__max_stack_depth__" => handle_integer_env_fun!(MAX_STACK_DEPTH, u32::MAX),
                    &"__print_set_inst__" => handle_integer_env_fun!(PRINT_SET_INST, 1),
                    &"if" => handle_logic_fun!(x == 0),
                    &"else" => handle_logic_fun!(x != 0),
                    &"$" => handle_special_fun!(None),
                    &"." => handle_special_fun!(1),
                    &"print" => {
                        let scope = self.locals.last_mut().expect("runtime internal error!");
                        let Some(x) = scope.get_var("x") else {
                            panic!("runtime internal error!")
                        };
                        self.output.push(x.borrow().to_string());
                        Ok(Rc::new(RefCell::new(Var::none())))
                    }
                    _ => panic!("runtime internal error!"),
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::Runtime;
    use crate::test::{examples, simple_expr};

    #[test]
    fn inst_mod_1() {
        let mut runtime = Runtime::new();
        runtime.execute("10 mod 2").unwrap();
        runtime.execute("10 mod 3").unwrap();
        runtime.execute("10 mod -3").unwrap();
        runtime.execute("-10 mod -3").unwrap();
        runtime.execute("-10 mod 3").unwrap();
        let output = &runtime.output;
        let correct = vec!["0", "1", "1", "2", "2"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        assert_eq!(output, &correct);
    }

    #[test]
    fn inst_mod_2() {
        let mut runtime = Runtime::new();
        runtime
            .execute("ans = ((1 * 2 + 3) * 4) + 5 * 6 + 7 mod 2")
            .unwrap();
        runtime
            .execute("ans = (1 mod 3) + (2 mod 3) mod 3")
            .unwrap();
        let output = &runtime.output;
        let correct = vec!["1", "0"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        assert_eq!(output, &correct);
    }

    #[test]
    fn simple_one_plus_one() {
        let mut runtime = Runtime::new();
        let output = runtime.execute("1 + 1").unwrap();
        let correct = vec!["2".to_owned()];
        assert_eq!(output, &correct);
    }

    #[test]
    fn simple_expr_1() {
        let mut runtime = Runtime::new();
        runtime.execute("i = 1").unwrap();
        let output = runtime.execute(simple_expr::expr_1()).unwrap();
        let correct = vec!["1", "2"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        assert_eq!(output, &correct);
    }

    #[test]
    fn simple_expr_2() {
        let mut runtime = Runtime::new();
        runtime.execute("i = 1").unwrap();
        runtime.execute("b = 2").unwrap();
        let output = runtime.execute(simple_expr::expr_2()).unwrap();
        let correct = vec!["1", "2", "2"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        assert_eq!(output, &correct);
    }

    #[test]
    fn simple_expr_3() {
        let mut runtime = Runtime::new();
        runtime.execute("i = 1").unwrap();
        runtime.execute("a = 2").unwrap();
        runtime.execute("b = 3").unwrap();
        let output = runtime.execute(simple_expr::expr_3()).unwrap();
        let correct = vec!["1", "2", "3", "1024.0370370"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        assert_eq!(output, &correct);
    }

    #[test]
    fn basic() {
        let source = examples::basic();
        let mut runtime = Runtime::new();
        let output = runtime.execute(&source).unwrap();
        let correct = vec!["57", "1235", "1235"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        assert_eq!(output, &correct);
    }

    #[test]
    fn cosine_law() {
        let source = examples::cosine_law();
        let mut runtime = Runtime::new();
        let output = runtime.execute(&source).unwrap();
        let correct = vec!["7", "7", "7", "0.5000000", "60.0000000", "60.0000000"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        assert_eq!(output, &correct);
    }

    #[test]
    fn fib() {
        let source = examples::fib();
        let mut runtime = Runtime::new();
        let output = runtime.execute(&source).unwrap();
        let correct = vec!["1", "1", "2", "3", "5", "55"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        assert_eq!(output, &correct);
    }

    #[test]
    fn bug_1() {
        let mut runtime = Runtime::new();
        runtime.execute("()()").unwrap();
        let output = &runtime.output;
        let correct = vec!["0"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        assert_eq!(output, &correct);
    }

    #[test]
    fn bug_2() {
        let mut runtime = Runtime::new();
        runtime.execute("sum(a, b) = a + b").unwrap();
        runtime.execute("sin(sum(1, -1))").unwrap();
        let output = &runtime.output;
        let correct = vec!["0.0000000"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        assert_eq!(output, &correct);
    }
}
