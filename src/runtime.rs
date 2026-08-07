use crate::comiler::{Compiler, Expr, Inst};
use crate::env::*;
use crate::error::{GlobalError, RuntimeError};
use crate::var::{Var, VarType};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::Into;
use std::path::PathBuf;
use std::rc::Rc;

// TODO: seperate RuntimeFnCall from BuiltinFnCall (import, print, println)

const PI: f64 = std::f64::consts::PI;
const E: f64 = std::f64::consts::E;

#[derive(Debug, Clone, Default)]
pub struct Fun<'input> {
    para_name: Vec<&'input str>,
    data: Vec<Inst<'input>>,
}

#[derive(Debug, Default, Clone)]
pub struct Scope<'input> {
    id: usize,
    name: &'input str,
    stack_depth: u32,
    heap: Vec<Rc<RefCell<Var>>>,
    var_table: HashMap<&'input str, Rc<RefCell<Var>>>,
    fun_table: HashMap<&'input str, Rc<RefCell<Fun<'input>>>>,
}

#[derive(Debug, Default, Clone)]
pub struct Runtime<'input> {
    builtin: Scope<'input>,
    locals: Vec<Scope<'input>>,
    output: Vec<String>,
    work_path: PathBuf,
    module: HashMap<&'input str, &'input str>,
    out_buffer: String,
    max_scope_id: usize,
}

impl<'input> Scope<'input> {
    pub fn new(name: &'input str, id: usize) -> Self {
        let mut new = Self::default();
        new.name = name;
        new.id = id;
        new
    }
    pub fn get_name(&self) -> &'input str {
        self.name
    }
    pub fn get_id(&self) -> usize {
        self.id
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
    pub fn add_ref_fun(&mut self, name: &'input str, ref_fun: Rc<RefCell<Fun<'input>>>) {
        self.fun_table.insert(name, ref_fun);
    }
    pub fn get_heap(&self, index: usize) -> Rc<RefCell<Var>> {
        Rc::clone(&self.heap[index])
    }
    pub fn add_arr(&mut self, len: usize) -> (usize, usize) {
        // return [start, end)
        let mem: Vec<Rc<RefCell<Var>>> = (0..len)
            .map(|_| Rc::new(RefCell::new(Var::from(0))))
            .collect();
        let start = self.heap.len();
        let end = start + len;
        self.heap.extend(mem.into_iter());
        (start, end)
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

        // pre allocate locals
        runtime.locals = Vec::with_capacity(unsafe { MAX_STACK_DEPTH + 1 } as usize);

        runtime.builtin.name = "__builtin__";
        // add builtin constant
        runtime.builtin.add_var("pi", Var::from(PI));
        runtime.builtin.add_var("e", Var::from(E));
        runtime.builtin.add_var("true", Var::from(0));
        runtime.builtin.add_var("false", Var::from(1));
        runtime.builtin.add_var("0", Var::from(0));
        runtime.builtin.add_var("1", Var::from(1));
        runtime
            .builtin
            .add_var("None", Var::from_string(String::new()));

        // add builtin functions
        // NOTE: need to handle exec_inst()::BuiltinFnCall(_)
        macro_rules! group_to_literal {
            ( $($arg:ident),* $(,)? ) => {
                vec![ $( stringify!($arg) ),* ]
            };
        }
        macro_rules! add_builtin_fn {
            ($name:ident($($para:ident),*)) => {
                runtime.builtin.set_fun(
                    stringify!($name),
                    Fun {
                        para_name: group_to_literal!($($para),*),
                        data: vec![Inst::BuiltinFnCall(stringify!($name))],
                    },
                )
            };
        }
        // output functions
        add_builtin_fn! { print(x) };
        add_builtin_fn! { println(x) };
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
        add_builtin_fn! { __index_base__(x) }
        // logic functions
        add_builtin_fn! { if(x) }
        add_builtin_fn! { else(x) }
        add_builtin_fn! { sign(x) }
        // type functions
        add_builtin_fn! { int32(x) }
        // sequence relative functions
        add_builtin_fn! { Sequence(len) }
        add_builtin_fn! { len(seq) }
        // module relative functions
        add_builtin_fn! { import(__module__) }
        // control functions
        add_builtin_fn! { abort(msg) }
        add_builtin_fn! { assert_eq(lhs, rhs, msg) }
        add_builtin_fn! { assert_ne(lhs, rhs, msg) }
        // special functions
        runtime.builtin.set_fun(
            "$",
            Fun {
                para_name: vec!["x"],
                data: vec![Inst::BuiltinFnCall("$")],
            },
        );
        runtime.builtin.set_fun(
            ".",
            Fun {
                para_name: vec!["x"],
                data: vec![Inst::BuiltinFnCall(".")],
            },
        );

        // add global scope
        let mut global = Scope::new("__global__", runtime.next_scope_id());
        global.stack_depth = 0;
        runtime.locals.push(global);

        runtime
    }
    pub fn set_work_path(&mut self, path: PathBuf) {
        self.work_path = path;
    }
    pub fn execute(&mut self, source: &'input str) -> Result<&Vec<String>, GlobalError> {
        let mut compiler = Compiler::new(source);
        let ast = match compiler.compile() {
            Ok(ast) => ast,
            Err(ce) => return Err(GlobalError::CE(ce)),
        };
        match self.exec_ast(ast).err() {
            Some(err) => return Err(err),
            None => {}
        };
        if !self.out_buffer.is_empty() {
            let output = self.out_buffer.drain(..).collect();
            self.output.push(output);
            self.out_buffer.clear();
        }
        Ok(&self.output)
    }
    pub fn exec_ast(&mut self, ast: &Vec<Inst<'input>>) -> Result<(), GlobalError> {
        for (idx, inst) in ast.iter().enumerate() {
            let to_print = if let &Inst::Set(_, _) = inst {
                unsafe { PRINT_SET_INST == 1 }
            } else {
                true
            };
            let output = match self.exec_inst(inst, idx) {
                Ok(val) => val,
                Err(ce) => {
                    while self.locals.last().unwrap().name != "__global__" {
                        self.locals.pop();
                    }
                    return Err(GlobalError::RE(ce));
                }
            };
            let out_str = output.borrow().to_string();
            if to_print && !out_str.is_empty() {
                self.output.push(out_str);
            }
        }
        Ok(())
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
    fn next_scope_id(&mut self) -> usize {
        let id = self.max_scope_id;
        self.max_scope_id += 1;
        id
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

                let mut sub_local = Scope::new(name, self.next_scope_id());
                sub_local.stack_depth = self.locals.last().unwrap().stack_depth + 1;
                for (pname, expr) in fun.borrow().para_name.iter().zip(args.iter()) {
                    let value = self.exec_expr(expr, line)?;
                    let value = (*value.borrow()).clone();
                    sub_local.add_var(pname, value);
                }
                self.locals.push(sub_local);
                let rval = self.exec_fun(&fun.borrow(), line)?;
                let fun_scope = self.locals.pop().unwrap();
                let cur_scope_index = self.locals.len() - 1;
                let cur_scope = self.locals.last_mut().unwrap();
                if rval.borrow().type_ == VarType::Sequence {
                    Runtime::sequence_move_data(&fun_scope, cur_scope, cur_scope_index, &rval);
                }
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
                let mut last_name = "";
                for scope in self.locals.iter_mut().rev() {
                    if last_name == scope.name {
                        continue;
                    }
                    if found {
                        break;
                    }
                    if let Some(evar) = scope.get_var(name) {
                        found = true;
                        var = Some(evar);
                    }
                    last_name = scope.name;
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
            Expr::String(str) => {
                let var = Var::from_string(str.to_string());
                Ok(Rc::new(RefCell::new(var)))
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
        macro_rules! check_is_num {
            ($x:ident) => {{
                if !$x.borrow().is_num() {
                    return Err(RuntimeError {
                        line,
                        msg: format!("variable or expression is not number, cannot be an operand or a argument of math functions"),
                    });
                }
            }};
            ($arg:ident $(, $args:ident),*) => {{
                check_is_num!($arg);
                check_is_num!($($args),*);
            }};
        }
        macro_rules! handle_binary {
            ($lhs:ident $op:tt $rhs:ident) => {{
                let lhs = self.exec_expr($lhs, line)?;
                let rhs = self.exec_expr($rhs, line)?;
                check_none!(lhs, rhs);
                check_is_num!(lhs, rhs);
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
                check_is_num!(lhs);
                // check lhs is zero for logic function
                if lhs.borrow().type_ <= VarType::I64 {
                    let lvalue: i64 = (&*lhs.borrow()).into();
                    if lvalue == 0 {
                        return Ok(lhs);
                    }
                }
                let rhs = self.exec_expr(rhs, line)?;
                if lhs.borrow().type_ <= VarType::I64 {
                    let lhs_val: i64 = (&*lhs.borrow()).into();
                    if lhs_val == 1 {
                        return Ok(rhs);
                    }
                }
                check_none!(rhs);
                check_is_num!(rhs);
                let eval = Rc::new(RefCell::new(&*lhs.borrow() * &*rhs.borrow()));
                Ok(eval)
            }
            Inst::Idx(lhs, rhs) => {
                let lhs = self.exec_expr(lhs, line)?;
                check_none!(lhs);
                if lhs.borrow().type_ != VarType::Sequence {
                    return Err(RuntimeError {
                        line,
                        msg: format!("only a sequence can be indexed"),
                    });
                }
                let rhs = self.exec_expr(rhs, line)?;
                check_none!(rhs);
                check_is_num!(rhs);
                let (scope_index, scope_id) = lhs.borrow().get_scope();
                let (start, end) = lhs.borrow().get_boundary();
                if rhs.borrow().type_ >= VarType::F64 {
                    return Err(RuntimeError {
                        line,
                        msg: format!("only integers can be used for indexing"),
                    });
                }
                let start = start as i64;
                let end = end as i64;
                let index: i64 = (&*rhs.borrow()).into();
                let index_base = unsafe { INDEX_BASE } as i64;
                let index = if index >= 0 {
                    start + index - index_base
                } else {
                    end + index
                };
                if index < 0 || index >= end {
                    return Err(RuntimeError {
                        line,
                        msg: format!(
                            "sequence index out of range, length is {} but got index {} ({}-base)",
                            end - start,
                            index - start + index_base,
                            index_base
                        ),
                    });
                }
                let index: usize = match index.try_into() {
                    Ok(index) => index,
                    Err(_) => {
                        return Err(RuntimeError {
                            line,
                            msg: format!(
                                "cannot index using type {} due to computer architecture",
                                rhs.borrow().type_
                            ),
                        });
                    }
                };
                let val = match self.locals.get(scope_index) {
                    Some(scope) => {
                        if scope.id != scope_id {
                            return Err(RuntimeError {
                                line,
                                msg: format!("sequence data out of scope before indexing"),
                            });
                        }
                        scope.get_heap(index)
                    }
                    None => {
                        return Err(RuntimeError {
                            line,
                            msg: format!("sequence data out of scope before indexing"),
                        });
                    }
                };
                Ok(val)
            }
            Inst::Set(lhs, rhs) => match lhs {
                Expr::None => panic!("compiler internal error (in runtime)"),
                Expr::Var(name) => {
                    let rhs = self.exec_expr(rhs, line)?;
                    // if rhs.borrow().type_ == VarType::None {
                    //     return Err(RuntimeError {
                    //         line,
                    //         msg: format!(
                    //             "cannot assign None to a variable, usually caused by assigning function definition expression to a variable"
                    //         ),
                    //     });
                    // }
                    if self.builtin.has_var(name) {
                        return Err(RuntimeError {
                            line,
                            msg: format!("overriding builtin constant {}", name),
                        });
                    }
                    let mut last_name = "";
                    for scope in self.locals.iter().rev() {
                        if last_name == scope.name {
                            continue;
                        }
                        if let Some(var) = scope.get_var(name) {
                            *var.borrow_mut() = Var::clone(&rhs.borrow());
                            return Ok(var);
                        }
                        last_name = scope.name;
                    }
                    self.locals
                        .last_mut()
                        .unwrap()
                        .add_var(name, Var::clone(&rhs.borrow()));
                    Ok(self.locals.last().unwrap().get_var(name).unwrap())
                }
                Expr::Inst(_) => {
                    let rhs = self.exec_expr(rhs, line)?;
                    // if rhs.borrow().type_ == VarType::None {
                    //     return Err(RuntimeError {
                    //         line,
                    //         msg: format!(
                    //             "cannot assign None to a variable, usually caused by assigning function definition expression to a variable"
                    //         ),
                    //     });
                    // }
                    let lhs = self.exec_expr(lhs, line)?;
                    let val = Var::clone(&rhs.borrow());
                    *lhs.borrow_mut() = val;
                    Ok(lhs)
                }
                Expr::Number(_) => Err(RuntimeError {
                    line,
                    msg: format!("cannot assign value to numbers"),
                }),
                Expr::FunCall(name, param) => {
                    if self.builtin.has_fun(name) {
                        Err(RuntimeError {
                            line,
                            msg: format!("overriding builtin function {}", name),
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
                Expr::String(_) => Err(RuntimeError {
                    line,
                    msg: format!("cannot assign value to string literal"),
                }),
            },
            Inst::Neg(expr) => {
                let val = self.exec_expr(expr, line)?;
                check_none!(val);
                check_is_num!(val);
                Ok(Rc::new(RefCell::new(-&*val.borrow())))
            }
            Inst::Pow(lhs, rhs) => {
                let lhs = self.exec_expr(lhs, line)?;
                let rhs = self.exec_expr(rhs, line)?;
                check_none!(lhs, rhs);
                check_is_num!(lhs, rhs);
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
            Inst::BuiltinFnCall(name) => {
                macro_rules! handle_arg_1 {
                    ($rust_fn:ident $(, $default_args:expr),*) => {{
                        let scope = self.locals.last_mut().expect("runtime internal error!");
                        let Some(x) = scope.get_var("x") else {
                            panic!("runtime internal error!")
                        };
                        check_none!(x);
                        check_is_num!(x);
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
                            check_is_num!(x);
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
                        check_is_num!(x);
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
                    &"__index_base__" => handle_integer_env_fun!(INDEX_BASE, 1),
                    &"if" => handle_logic_fun!(x == 0),
                    &"else" => handle_logic_fun!(x != 0),
                    &"sign" => {
                        let scope = self.locals.last_mut().expect("runtime internal error!");
                        let Some(x) = scope.get_var("x") else {
                            panic!("runtime internal error!")
                        };
                        check_none!(x);
                        let x: f64 = (&*x.borrow()).into();
                        let result = match x.total_cmp(&0.0) {
                            Ordering::Equal => 0,
                            Ordering::Greater => 1,
                            Ordering::Less => -1,
                        };
                        Ok(Rc::new(RefCell::new(Var::from(result))))
                    }
                    &"$" => handle_special_fun!(None),
                    &"." => handle_special_fun!(1),
                    &"int32" => {
                        let scope = self.locals.last_mut().expect("runtime internal error!");
                        let Some(x) = scope.get_var("x") else {
                            panic!("runtime internal error!")
                        };
                        check_none!(x);
                        check_is_num!(x);
                        if x.borrow().type_ <= VarType::F64 {
                            let x: f64 = (&*x.borrow()).into();
                            let trunc_float = x.trunc();
                            let trunc_int = trunc_float as i32;
                            if f64::from(trunc_int) != trunc_float {
                                return Err(RuntimeError {
                                    line,
                                    msg: format!("too large or small to truncate to I32"),
                                });
                            }
                            Ok(Rc::new(RefCell::new(Var::from(trunc_int))))
                        } else {
                            //TODO: add BigNum implemntation
                            todo!("add BigNum implementation")
                        }
                    }
                    &"print" => {
                        let scope = self.locals.last_mut().expect("runtime internal error!");
                        let Some(x) = scope.get_var("x") else {
                            panic!("runtime internal error!")
                        };
                        let out = match x.borrow().type_ {
                            VarType::Sequence => self.sequence_to_string(Rc::clone(&x), line)?,
                            _ => x.borrow().to_string(),
                        };
                        self.out_buffer.push_str(&out);
                        Ok(Rc::new(RefCell::new(Var::none())))
                    }
                    &"println" => {
                        let scope = self.locals.last_mut().expect("runtime internal error!");
                        let Some(x) = scope.get_var("x") else {
                            panic!("runtime internal error!")
                        };
                        let out = match x.borrow().type_ {
                            VarType::Sequence => self.sequence_to_string(Rc::clone(&x), line)?,
                            _ => x.borrow().to_string(),
                        };
                        self.out_buffer.push_str(&out);
                        let output = self.out_buffer.drain(..).collect();
                        self.output.push(output);
                        self.out_buffer.clear();
                        Ok(Rc::new(RefCell::new(Var::none())))
                    }
                    &"Sequence" => {
                        let scope = self.locals.last_mut().expect("runtime internal error!");
                        let Some(len) = scope.get_var("len") else {
                            panic!("runtime internal error!")
                        };
                        check_none!(len);
                        let len_type = len.borrow().type_;
                        if len_type >= VarType::F64 {
                            return Err(RuntimeError {
                                line,
                                msg: format!(
                                    "cannot init an sequence with type {}, only accept integer",
                                    len_type
                                ),
                            });
                        }
                        let len: i64 = (&*len.borrow()).into();
                        let len: usize = match len.try_into() {
                            Ok(len) => len,
                            Err(_) => {
                                return Err(RuntimeError {
                                    line,
                                    msg: format!(
                                        "cannot index using type {} due to computer architecture",
                                        len_type
                                    ),
                                });
                            }
                        };
                        let cur_local = self.locals.len() - 1;
                        let ptr = self.locals[cur_local].add_arr(len);
                        let cur_local_id = self.locals[cur_local].id;
                        let arr = Var::new_sequence(ptr, (cur_local, cur_local_id));
                        Ok(Rc::new(RefCell::new(arr)))
                    }
                    &"len" => {
                        let scope = self.locals.last_mut().expect("runtime internal error!");
                        let Some(seq) = scope.get_var("seq") else {
                            panic!("runtime internal error!")
                        };
                        check_none!(seq);
                        if seq.borrow().type_ != VarType::Sequence {
                            return Err(RuntimeError {
                                line,
                                msg: format!(
                                    "builtin function len() could only get the length of Sequence"
                                ),
                            });
                        }
                        let (start, end) = seq.borrow().get_boundary();
                        // FIXME: break on 128bit?
                        let value = (end - start) as i64;
                        Ok(Rc::new(RefCell::new(Var::from(value))))
                    }
                    &"import" => {
                        let scope = self.locals.last_mut().expect("runtime internal error!");
                        let Some(module) = scope.get_var("__module__") else {
                            panic!("runtime internal error!")
                        };
                        if module.borrow().type_ != VarType::None {
                            return Err(RuntimeError {
                                line,
                                msg: format!(
                                    "import() function only accpets literal string but got type {}",
                                    module.borrow().type_
                                ),
                            });
                        }
                        let path_str = module.borrow().to_string();
                        if path_str.is_empty() {
                            return Err(RuntimeError {
                                line,
                                msg: format!(
                                    "import() function only accpets literal string but got None",
                                ),
                            });
                        }
                        let mut path = std::path::PathBuf::new();
                        path = path.join(self.work_path.clone());
                        for sub_path in path_str.split('/') {
                            path = path.join(sub_path);
                        }
                        let source = match std::fs::read_to_string(path) {
                            Ok(s) => s,
                            Err(_) => {
                                return Err(RuntimeError {
                                    line,
                                    msg: format!("module {} does not exist", path_str),
                                });
                            }
                        };
                        let mod_name = path_str.leak();
                        if self.module.contains_key(mod_name) {
                            return Ok(Rc::new(RefCell::new(Var::none())));
                        }
                        let source = source.leak();
                        self.module.insert(mod_name, source);
                        let last_output = self.output.len();
                        match self.execute(source) {
                            Ok(_) => {
                                // do actual import
                                let split_index = self.locals.len() - 1;
                                let prv_scope_index = self.locals.len() - 2;
                                let (prv_scope, mod_scope) = self.locals.split_at_mut(split_index);
                                let prv_scope = prv_scope.last_mut().unwrap();
                                let mod_scope = mod_scope.last_mut().unwrap();
                                // do variable
                                for (var_name, var_value) in mod_scope.var_table.iter() {
                                    if var_name == &"__module__" {
                                        continue;
                                    }
                                    // FIXME: check will not work, module overwite value while executing
                                    if prv_scope.has_var(var_name) {
                                        return Err(RuntimeError {
                                            line,
                                            msg: format!(
                                                "duplicate variable {} in module {}",
                                                var_name, mod_name
                                            ),
                                        });
                                    }
                                    let var_type = var_value.borrow().type_;
                                    match var_type {
                                        VarType::Sequence => {
                                            Runtime::sequence_move_data(
                                                mod_scope,
                                                prv_scope,
                                                prv_scope_index,
                                                var_value,
                                            );
                                            prv_scope.add_ref_var(var_name, Rc::clone(var_value));
                                        }
                                        _ => {
                                            prv_scope.add_ref_var(var_name, Rc::clone(var_value));
                                        }
                                    }
                                }
                                // do function
                                for (fun_name, fun_value) in mod_scope.fun_table.iter() {
                                    if prv_scope.has_fun(fun_name) {
                                        return Err(RuntimeError {
                                            line,
                                            msg: format!(
                                                "duplicate function {} in module {}",
                                                fun_name, mod_name
                                            ),
                                        });
                                    }
                                    prv_scope.add_ref_fun(fun_name, Rc::clone(fun_value));
                                }
                                // clear module output
                                self.output.drain(last_output..);
                                Ok(Rc::new(RefCell::new(Var::from_string(unsafe {
                                    // SAFE: no multiple threads
                                    if DETAIL_DEPTH == 1 {
                                        format!("<import module {}>", mod_name)
                                    } else {
                                        format!("")
                                    }
                                }))))
                            }
                            Err(err) => Err(RuntimeError {
                                line,
                                msg: format!("\n\tImport Error:\n\t\t{}", err.no_loc_info()),
                            }),
                        }
                    }
                    &"abort" => {
                        let scope = self.locals.last_mut().expect("runtime internal error!");
                        let Some(msg) = scope.get_var("msg") else {
                            panic!("runtime internal error!")
                        };
                        Err(RuntimeError {
                            line,
                            msg: format!("abort(\"{}\")", msg.borrow().to_string()),
                        })
                    }
                    &"assert_eq" => {
                        let scope = self.locals.last_mut().expect("runtime internal error!");
                        let Some(lhs) = scope.get_var("lhs") else {
                            panic!("runtime internal error!")
                        };
                        let Some(rhs) = scope.get_var("rhs") else {
                            panic!("runtime internal error!")
                        };
                        let Some(msg) = scope.get_var("msg") else {
                            panic!("runtime internal error!")
                        };
                        if lhs.borrow().type_ == rhs.borrow().type_
                            && lhs.borrow().type_ == VarType::Sequence
                        {
                            if self.is_sequence_equal(&lhs, &rhs) {
                                Ok(Rc::new(RefCell::new(Var::none())))
                            } else {
                                Err(RuntimeError {
                                    line,
                                    msg: format!(
                                        "assert_eq(lhs, rhs, \"{}\")",
                                        msg.borrow().to_string()
                                    ),
                                })
                            }
                        } else if (*lhs.borrow()) == (*rhs.borrow()) {
                            Ok(Rc::new(RefCell::new(Var::none())))
                        } else {
                            Err(RuntimeError {
                                line,
                                msg: format!(
                                    "assert_eq(lhs, rhs, \"{}\")",
                                    msg.borrow().to_string()
                                ),
                            })
                        }
                    }
                    &"assert_ne" => {
                        let scope = self.locals.last_mut().expect("runtime internal error!");
                        let Some(lhs) = scope.get_var("lhs") else {
                            panic!("runtime internal error!")
                        };
                        let Some(rhs) = scope.get_var("rhs") else {
                            panic!("runtime internal error!")
                        };
                        let Some(msg) = scope.get_var("msg") else {
                            panic!("runtime internal error!")
                        };
                        if lhs.borrow().type_ == rhs.borrow().type_
                            && lhs.borrow().type_ == VarType::Sequence
                        {
                            if !self.is_sequence_equal(&lhs, &rhs) {
                                Ok(Rc::new(RefCell::new(Var::none())))
                            } else {
                                Err(RuntimeError {
                                    line,
                                    msg: format!(
                                        "assert_eq(lhs, rhs, \"{}\")",
                                        msg.borrow().to_string()
                                    ),
                                })
                            }
                        } else if (*lhs.borrow()) != (*rhs.borrow()) {
                            Ok(Rc::new(RefCell::new(Var::none())))
                        } else {
                            Err(RuntimeError {
                                line,
                                msg: format!(
                                    "assert_ne(lhs, rhs, \"{}\")",
                                    msg.borrow().to_string()
                                ),
                            })
                        }
                    }
                    _ => panic!("runtime internal error!"),
                }
            }
        }
    }
    fn is_sequence_equal(&self, lhs: &Rc<RefCell<Var>>, rhs: &Rc<RefCell<Var>>) -> bool {
        let lhs_ptr = lhs.borrow().get_boundary();
        let (lhs_scope, _) = lhs.borrow().get_scope();
        let rhs_ptr = rhs.borrow().get_boundary();
        let (rhs_scope, _) = rhs.borrow().get_scope();
        for (li, ri) in (lhs_ptr.0..lhs_ptr.1).zip(rhs_ptr.0..rhs_ptr.1) {
            let l = Rc::clone(&self.locals[lhs_scope].heap[li]);
            let r = Rc::clone(&self.locals[rhs_scope].heap[ri]);
            if l.borrow().type_ == r.borrow().type_ && l.borrow().type_ == VarType::Sequence {
                if !self.is_sequence_equal(&l, &r) {
                    return false;
                }
            } else {
                if (*l.borrow()) != (*r.borrow()) {
                    return false;
                }
            }
        }
        true
    }
    fn sequence_move_data(
        from: &Scope<'input>,
        to: &mut Scope<'input>,
        to_index: usize,
        var_value: &Rc<RefCell<Var>>,
    ) {
        let from_ptr = var_value.borrow().get_boundary();
        let to_ptr = to.add_arr(from_ptr.1 - from_ptr.0);
        // copy heap data recursively
        for (fi, ti) in (from_ptr.0..from_ptr.1).zip(to_ptr.0..to_ptr.1) {
            to.heap[ti] = Rc::clone(&from.heap[fi]);
            if from.heap[fi].borrow().type_ == VarType::Sequence {
                Runtime::sequence_move_data(from, to, to_index, &from.heap[fi]);
            }
        }
        let to_id = to.id;
        *var_value.borrow_mut() = Var::new_sequence(to_ptr, (to_index, to_id));
    }
    fn sequence_to_string(
        &mut self,
        seq: Rc<RefCell<Var>>,
        line: usize,
    ) -> Result<String, RuntimeError> {
        let (scope_index, scope_id) = seq.borrow().get_scope();
        let (start, end) = seq.borrow().get_boundary();
        let ele: Result<Vec<String>, RuntimeError> = (start..end)
            .map(|i| match self.locals.get(scope_index) {
                Some(scope) => Ok({
                    if scope.id != scope_id {
                        return Err(RuntimeError {
                            line,
                            msg: format!("sequence data out of scope before indexing"),
                        });
                    }
                    let e = scope.get_heap(i);
                    match e.borrow().type_ {
                        VarType::Sequence => self.sequence_to_string(Rc::clone(&e), line)?,
                        _ => e.borrow().to_string(),
                    }
                }),
                None => Err(RuntimeError {
                    line,
                    msg: format!("sequence data out of scope before indexing"),
                }),
            })
            .collect();
        Ok(format!("[{}]", ele?.join(", ")))
    }
}

#[cfg(test)]
mod test {
    use super::Runtime;
    use crate::test::{examples, simple_expr};

    #[test]
    fn module() {
        let source = examples::module();
        let mut runtime = Runtime::new();
        runtime.work_path = std::path::PathBuf::new().join("examples");
        let output = runtime.execute(&source).unwrap();
        let correct = vec![
            "hello, world!",
            "hello, Chisato!",
            "hello, Bob!",
            "hello, Alice!",
            "hello, [Bob, Alice]!",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
        assert_eq!(output, &correct);
    }

    #[test]
    fn print_sequence() {
        let mut runtime = Runtime::new();
        runtime.execute("seq = Sequence(3)").unwrap();
        runtime.execute("seq:0 = Sequence(3)").unwrap();
        runtime.execute("print(seq)").unwrap();
        let output = &runtime.output;
        let correct = vec![
            "<Sequence of Scope 0 with length 3 at 0x0000000000000000>",
            "<Sequence of Scope 0 with length 3 at 0x0000000000000003>",
            "[[0, 0, 0], 0, 0]",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
        assert_eq!(output, &correct);
    }

    #[test]
    fn sequence() {
        let source = examples::sequence();
        let mut runtime = Runtime::new();
        let output = runtime.execute(&source).unwrap();
        let correct = vec![
            "<Sequence of Scope 0 with length 3 at 0x0000000000000000>",
            "0",
            "1",
            "1",
            "fib(10) = 55",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
        assert_eq!(output, &correct);
    }

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
