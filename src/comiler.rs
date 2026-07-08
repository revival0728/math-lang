use crate::error::CompilerError;
use crate::lexer::{Lexer, Token};
use lexgen_util::LexerErrorKind;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub enum Expr<'input> {
    #[default]
    None,
    Number(&'input str),
    Var(&'input str),
    FunCall(&'input str, Vec<Expr<'input>>),
    Inst(Box<Inst<'input>>),
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub enum Inst<'input> {
    #[default]
    None,
    Expr(Expr<'input>),
    Neg(Expr<'input>),
    Set(Expr<'input>, Expr<'input>),
    Add(Expr<'input>, Expr<'input>),
    Sub(Expr<'input>, Expr<'input>),
    Div(Expr<'input>, Expr<'input>),
    Mul(Expr<'input>, Expr<'input>),
    Pow(Expr<'input>, Expr<'input>),
    Mod(Expr<'input>, Expr<'input>),
    BultinFnCall(&'input str),
}

#[derive(Debug, Default, Clone)]
pub struct CompilerState {
    in_paren: usize,
    in_expr: usize,
    in_funcall: usize,
    expr_depth: usize,
}

#[derive(Debug, Default, Clone)]
pub struct Compiler<'input> {
    source: &'input str,
    state: CompilerState,
    ast: Vec<Inst<'input>>,
    oper_tk: Vec<(lexgen_util::Loc, Token<'input>)>,
    idnt_tk: Vec<(lexgen_util::Loc, Expr<'input>)>,
    fun_call: Vec<Vec<Vec<(lexgen_util::Loc, Token<'input>)>>>,
    expr_buf: Vec<(lexgen_util::Loc, Vec<(lexgen_util::Loc, Token<'input>)>)>,
}

impl<'input> Inst<'input> {
    pub fn from_binary_token(token: &Token<'input>, lhs: Expr<'input>, rhs: Expr<'input>) -> Self {
        match token {
            Token::Plus => Self::Add(lhs, rhs),
            Token::Minus => Self::Sub(lhs, rhs),
            Token::Star => Self::Mul(lhs, rhs),
            Token::Slash => Self::Div(lhs, rhs),
            Token::Eq => Self::Set(lhs, rhs),
            Token::Pow => Self::Pow(lhs, rhs),
            Token::Mod => Self::Mod(lhs, rhs),
            _ => panic!("compiler internal error!"),
        }
    }
    pub fn is_expr(&self) -> bool {
        if let Inst::Expr(_) = self {
            true
        } else {
            false
        }
    }
    pub fn unwrap_expr(self) -> Expr<'input> {
        if let Inst::Expr(expr) = self {
            expr
        } else {
            panic!("compiler internal error!")
        }
    }
    pub fn priority(&self) -> u8 {
        match self {
            Inst::None | Inst::Expr(_) | Inst::BultinFnCall(_) => {
                panic!("compiler internal error!")
            }
            Inst::Set(_, _) => 0,
            Inst::Mod(_, _) => 1,
            Inst::Add(_, _) => 2,
            Inst::Sub(_, _) => 2,
            Inst::Mul(_, _) => 3,
            Inst::Div(_, _) => 3,
            Inst::Pow(_, _) => 4,
            Inst::Neg(_) => 5,
        }
    }
    pub fn get_binary_exprs(&mut self) -> (&mut Expr<'input>, &mut Expr<'input>) {
        match self {
            Self::Add(lhs, rhs)
            | Self::Sub(lhs, rhs)
            | Self::Mul(lhs, rhs)
            | Self::Div(lhs, rhs)
            | Self::Set(lhs, rhs)
            | Self::Pow(lhs, rhs)
            | Self::Mod(lhs, rhs) => (lhs, rhs),
            _ => panic!("compiler internal error!"),
        }
    }
}

impl<'input> Compiler<'input> {
    pub fn new(source: &'input str) -> Self {
        let state = CompilerState::default();
        let ast = Vec::new();
        let oper_tk = Vec::new();
        let idnt_tk = Vec::new();
        let fun_call = Vec::new();
        let expr_buf = Vec::new();

        Compiler {
            source,
            state,
            ast,
            oper_tk,
            idnt_tk,
            fun_call,
            expr_buf,
        }
    }
}

impl<'input> Compiler<'input> {
    pub fn compile(&mut self) -> Result<&Vec<Inst<'input>>, CompilerError> {
        let lexer = Lexer::new(self.source);

        self.expr_buf.push((lexgen_util::Loc::default(), vec![]));

        let mut to_neg_sign = true;
        let mut to_add_mul = false;
        let mut to_make_func = false;
        let mut to_push_token;
        for token in lexer {
            to_push_token = true;
            if token.is_err() {
                let err = token.err().unwrap();
                let mut ce = CompilerError::new_with_loc(&err.location);
                match err.kind {
                    LexerErrorKind::InvalidToken => {
                        ce.msg = "invalid token".to_owned();
                    }
                    LexerErrorKind::Custom(_) => {}
                }
                return Err(ce);
            }

            let (lloc, token, _rloc) = token.unwrap();

            match token {
                Token::None => panic!("compiler internal error!"),
                Token::Newline => {
                    to_add_mul = false;
                    to_make_func = false;
                    to_neg_sign = false;
                    to_push_token = false;
                    self.expr_buf.push((lloc, vec![]));
                }
                Token::Comma
                | Token::Eq
                | Token::Plus
                | Token::Minus
                | Token::Star
                | Token::Slash
                | Token::Mod => {
                    to_add_mul = false;
                    to_make_func = false;
                    if token == Token::Minus && to_neg_sign {
                        to_neg_sign = false;
                        to_push_token = false;
                        self.expr_buf
                            .last_mut()
                            .unwrap()
                            .1
                            .push((lloc, Token::NegSign));
                    } else {
                        to_neg_sign = true;
                    }
                }
                Token::Number(_) => {
                    to_make_func = false;
                    to_neg_sign = false;
                    if to_add_mul {
                        return Err(CompilerError::new_with_literal(
                            &lloc,
                            "consecutive number literals or numbers behind variable are not acceptable",
                        ));
                    } else {
                        to_add_mul = true;
                    }
                }
                Token::Var(_) => {
                    to_neg_sign = false;
                    to_make_func = true;
                    if to_add_mul {
                        self.expr_buf
                            .last_mut()
                            .unwrap()
                            .1
                            .push((lloc, Token::Star));
                    } else {
                        to_add_mul = true;
                    }
                }
                Token::LParen => {
                    to_neg_sign = false;
                    if to_make_func {
                        to_make_func = false;
                        let (loc, var_tk) = self.expr_buf.last_mut().unwrap().1.pop().unwrap();
                        if let Token::Var(name) = var_tk {
                            to_push_token = false;
                            self.expr_buf
                                .last_mut()
                                .unwrap()
                                .1
                                .push((loc, Token::FunCall(name)));
                        } else {
                            panic!("compiler internal error!");
                        }
                    } else if to_add_mul {
                        self.expr_buf
                            .last_mut()
                            .unwrap()
                            .1
                            .push((lloc, Token::Star));
                    }
                    to_add_mul = false;
                }
                _ => {
                    to_add_mul = false;
                    to_make_func = false;
                    to_neg_sign = false;
                }
            };
            if to_push_token {
                self.expr_buf.last_mut().unwrap().1.push((lloc, token));
            }
        }

        self.expr_buf.reverse();

        for _ in 0..self.expr_buf.len() {
            let expr = self.parse_expr()?;
            if expr == Expr::None {
                continue;
            }
            match expr {
                Expr::Inst(inst) => self.ast.push(*inst),
                Expr::None => panic!("compiler internal error!"),
                _ => self.ast.push(Inst::Expr(expr)),
            };
        }

        if !self.idnt_tk.is_empty() {
            return Err(CompilerError::new_with_literal(
                &self.idnt_tk[0].0,
                "redundant identifier or expression (missing operator)",
            ));
        }

        Ok(&self.ast)
    }
    fn parse_expr(&mut self) -> Result<Expr<'input>, CompilerError> {
        if self.expr_buf.is_empty() {
            return Ok(Expr::None);
        }

        if self.expr_buf.last().unwrap().1.is_empty() {
            self.expr_buf.pop();
            return Ok(Expr::None);
        }

        // seperate expressions
        self.oper_tk
            .push((lexgen_util::Loc::default(), Token::None));

        macro_rules! handle_comma_err {
            ($lloc:ident) => {{}};
        }

        let (rloc, tokens) = self.expr_buf.pop().unwrap();
        for (lloc, token) in tokens {
            if self.state.in_paren > 0 {
                macro_rules! push_token {
                    ($token:expr) => {{
                        self.expr_buf.last_mut().unwrap().1.push((lloc, $token));
                    }};
                }
                match token {
                    Token::LParen => {
                        self.state.in_paren += 1;
                        push_token!(Token::LParen);
                    }
                    Token::FunCall(name) => {
                        self.state.in_paren += 1;
                        push_token!(Token::FunCall(name));
                    }
                    Token::RParen => {
                        self.state.in_paren -= 1;
                        if self.state.in_paren == 0 {
                            let expr = self.parse_expr()?;
                            self.idnt_tk.push((lloc, expr));
                        } else {
                            push_token!(Token::RParen);
                        }
                    }
                    Token::Comma
                    | Token::Plus
                    | Token::Minus
                    | Token::Star
                    | Token::Slash
                    | Token::NegSign
                    | Token::Pow
                    | Token::Mod
                    | Token::Eq
                    | Token::Var(_)
                    | Token::Number(_) => push_token!(token),
                    Token::None | Token::Newline => panic!("compiler internal error!"),
                }
            } else if self.state.in_funcall > 0 {
                macro_rules! push_token {
                    ($token:expr) => {{
                        self.fun_call
                            .last_mut()
                            .unwrap()
                            .last_mut()
                            .unwrap()
                            .push((lloc, $token));
                    }};
                }
                match token {
                    Token::FunCall(name) => {
                        self.state.in_funcall += 1;
                        push_token!(Token::FunCall(name));
                    }
                    Token::LParen => {
                        self.state.in_funcall += 1;
                        push_token!(Token::LParen);
                    }
                    Token::RParen => {
                        self.state.in_funcall -= 1;
                        if self.state.in_funcall == 0 {
                            let expr = self.parse_funcall()?;
                            self.idnt_tk.push((lloc, expr));
                        } else {
                            push_token!(Token::RParen);
                        }
                    }
                    Token::Comma => {
                        self.fun_call.push(vec![]);
                    }
                    Token::Plus
                    | Token::Minus
                    | Token::Star
                    | Token::Slash
                    | Token::NegSign
                    | Token::Pow
                    | Token::Mod
                    | Token::Eq
                    | Token::Var(_)
                    | Token::Number(_) => push_token!(token),
                    Token::None | Token::Newline => panic!("compiler internal error!"),
                }
            } else {
                match token {
                    Token::LParen => {
                        self.state.in_paren += 1;
                        self.expr_buf.push((lloc, vec![]));
                    }
                    Token::RParen => {
                        return Err(CompilerError::new_with_literal(&lloc, "missing left paren"));
                    }
                    Token::FunCall(name) => {
                        self.state.in_funcall += 1;
                        self.fun_call
                            .push(vec![vec![(lloc, Token::Var(name))], vec![]]);
                    }
                    Token::Comma => {
                        return Err(CompilerError::new_with_literal(
                            &lloc,
                            "comma found outside function call",
                        ));
                    }
                    Token::Plus
                    | Token::Minus
                    | Token::Star
                    | Token::Slash
                    | Token::Pow
                    | Token::Mod
                    | Token::NegSign
                    | Token::Eq => self.oper_tk.push((lloc, token)),
                    Token::Var(name) => self.idnt_tk.push((lloc, Expr::Var(name))),
                    Token::Number(data) => self.idnt_tk.push((lloc, Expr::Number(data))),
                    Token::None | Token::Newline => panic!("compiler internal error!"),
                };
            }
        }

        if self.state.in_paren > 0 {
            return Err(CompilerError::new_with_literal(
                &rloc,
                "missing right paren",
            ));
        } else if self.state.in_funcall > 0 {
            return Err(CompilerError::new_with_literal(
                &rloc,
                "incomplete function call",
            ));
        } else if self.state.in_expr > 0 {
            return Err(CompilerError::new_with_literal(&rloc, "missing identifier"));
        } else {
            let inst = self.parse_inst()?;
            return if let Inst::Expr(expr) = inst {
                Ok(expr)
            } else {
                Ok(Expr::Inst(Box::new(inst)))
            };
        }
    }
    fn parse_funcall(&mut self) -> Result<Expr<'input>, CompilerError> {
        let mut args = Vec::new();
        let mut cur_fun_call = self.fun_call.pop().unwrap();
        if !cur_fun_call.last().unwrap().is_empty() {
            for _ in 1..cur_fun_call.len() {
                let tokens = cur_fun_call.pop().unwrap();
                self.expr_buf.push((cur_fun_call[0][0].0, tokens));
                let expr = self.parse_expr()?;
                args.push(expr);
            }
            args.reverse();
        } else {
            cur_fun_call.pop();
        }

        let mut fn_name = cur_fun_call.pop().unwrap();
        if let Token::Var(name) = fn_name.pop().unwrap().1 {
            Ok(Expr::FunCall(name, args))
        } else {
            panic!("compiler internal error!");
        }
    }
    fn parse_inst(&mut self) -> Result<Inst<'input>, CompilerError> {
        self.state.expr_depth = 0; // reset state
        while let Some((lloc, oper)) = self.oper_tk.pop() {
            match oper {
                Token::None => break,
                Token::NegSign => {
                    let Some((_, idnt)) = self.idnt_tk.pop() else {
                        return Err(CompilerError::new_with_literal(
                            &lloc,
                            "expected identifier or expression",
                        ));
                    };
                    if let Expr::Inst(inst) = idnt {
                        let mut cur_inst = Inst::Neg(Expr::None);
                        let mut inst = *inst;
                        let mut prv_inst = &mut inst;
                        let expr_depth = self.state.expr_depth;
                        self.state.expr_depth = 1;
                        for _ in 1..expr_depth {
                            if prv_inst.priority() > cur_inst.priority() {
                                break;
                            }
                            self.state.expr_depth += 1;
                            let (sub_lhs, _sub_rhs) = prv_inst.get_binary_exprs();
                            let Expr::Inst(nxt_inst) = sub_lhs else {
                                panic!("compiler internal error!")
                            };
                            prv_inst = nxt_inst.as_mut();
                        }
                        self.state.expr_depth += 1;
                        let (sub_lhs, _sub_rhs) = prv_inst.get_binary_exprs();
                        cur_inst = Inst::Neg(sub_lhs.clone());
                        *sub_lhs = Expr::Inst(Box::new(cur_inst));
                        self.idnt_tk.push((lloc, Expr::Inst(Box::new(inst))));
                    } else {
                        let inst = Box::new(Inst::Neg(idnt));
                        self.idnt_tk.push((lloc, Expr::Inst(inst)));
                    }
                }
                Token::Plus
                | Token::Minus
                | Token::Star
                | Token::Slash
                | Token::Pow
                | Token::Eq
                | Token::Mod => {
                    let Some((_rhs_loc, rhs)) = self.idnt_tk.pop() else {
                        return Err(CompilerError::new_with_literal(
                            &lloc,
                            "expected identifier or expression (missing RHS)",
                        ));
                    };
                    let Some((lhs_loc, lhs)) = self.idnt_tk.pop() else {
                        return Err(CompilerError::new_with_literal(
                            &lloc,
                            "expected identifier or expression (missing LHS)",
                        ));
                    };

                    if self.state.expr_depth == 0 {
                        self.state.expr_depth += 1;
                        let inst = Box::new(Inst::from_binary_token(&oper, lhs, rhs));
                        self.idnt_tk.push((lhs_loc, Expr::Inst(inst)));
                        continue;
                    }

                    let Expr::Inst(rhs_inst) = rhs else {
                        panic!("compiler internal error!")
                    };
                    let mut nxt_inst = *rhs_inst;
                    let mut prv_inst = &mut nxt_inst;
                    let mut cur_inst = Inst::from_binary_token(&oper, Expr::None, Expr::None);
                    let expr_depth = self.state.expr_depth;
                    self.state.expr_depth = 1;
                    for _ in 1..expr_depth {
                        if prv_inst.priority() > cur_inst.priority() {
                            break;
                        }
                        self.state.expr_depth += 1;
                        let (sub_lhs, _sub_rhs) = prv_inst.get_binary_exprs();
                        let Expr::Inst(nxt_inst) = sub_lhs else {
                            panic!("compiler internal error!")
                        };
                        prv_inst = nxt_inst.as_mut();
                    }
                    if cur_inst.priority() >= prv_inst.priority() {
                        self.state.expr_depth += 1;
                        let (sub_lhs, _sub_rhs) = prv_inst.get_binary_exprs();
                        cur_inst = Inst::from_binary_token(&oper, lhs, sub_lhs.clone());
                        *sub_lhs = Expr::Inst(Box::new(cur_inst));
                    } else {
                        *prv_inst = Inst::from_binary_token(
                            &oper,
                            lhs,
                            Expr::Inst(Box::new(prv_inst.clone())),
                        );
                    }
                    self.idnt_tk.push((lhs_loc, Expr::Inst(Box::new(nxt_inst))));
                }
                _ => panic!("compiler internal error!"),
            }
        }

        let Some((_, expr)) = self.idnt_tk.pop() else {
            panic!("compiler internal error!")
        };
        match expr {
            Expr::None => panic!("compiler internal error!"),
            Expr::Inst(inst) => Ok(*inst),
            _ => Ok(Inst::Expr(expr)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Compiler, Expr, Inst};
    use crate::test::{examples, simple_expr};
    use pretty_assertions::assert_eq;

    macro_rules! expr_inst {
        ($inst:expr) => {
            Expr::Inst(Box::new($inst))
        };
    }

    #[test]
    fn simple_expr_1() {
        let mut compiler = Compiler::new(simple_expr::expr_1());
        let ast = compiler.compile().unwrap();
        let correct = vec![Inst::Set(
            Expr::Var("i"),
            Expr::Inst(Box::new(Inst::Add(Expr::Var("i"), Expr::Number("1")))),
        )];
        assert_eq!(ast, &correct);
    }

    #[test]
    fn simple_expr_2() {
        let mut compiler = Compiler::new(simple_expr::expr_2());
        let ast = compiler.compile().unwrap();
        let correct = vec![Inst::Set(
            Expr::Var("i"),
            expr_inst!(Inst::Sub(
                expr_inst!(Inst::Add(
                    Expr::Var("i"),
                    expr_inst!(Inst::Mul(Expr::Number("1"), Expr::Var("b")))
                )),
                Expr::Number("1")
            )),
        )];
        assert_eq!(ast, &correct);
    }

    #[test]
    fn simple_expr_3() {
        let mut compiler = Compiler::new(simple_expr::expr_3());
        let ast = compiler.compile().unwrap();
        let correct = vec![Inst::Set(
            Expr::Var("i"),
            expr_inst!(Inst::Add(
                expr_inst!(Inst::Div(
                    expr_inst!(Inst::Mul(
                        expr_inst!(Inst::Mul(
                            Expr::Var("i"),
                            expr_inst!(Inst::Add(Expr::Number("1"), Expr::Number("2")))
                        )),
                        Expr::Number("3")
                    )),
                    expr_inst!(Inst::Pow(Expr::Var("b"), Expr::Number("5")))
                )),
                expr_inst!(Inst::Pow(
                    Expr::Number("4"),
                    expr_inst!(Inst::Add(Expr::Var("a"), Expr::Number("3")))
                ))
            )),
        )];
        assert_eq!(ast, &correct);
    }

    #[test]
    fn basic() {
        let source = examples::basic();
        let mut compiler = Compiler::new(&source);
        let ast = compiler.compile().unwrap();
        let correct = vec![
            Inst::Set(
                Expr::Var("ans"),
                expr_inst!(Inst::Add(
                    expr_inst!(Inst::Add(
                        expr_inst!(Inst::Mul(
                            expr_inst!(Inst::Add(
                                expr_inst!(Inst::Mul(Expr::Number("1"), Expr::Number("2"))),
                                Expr::Number("3")
                            )),
                            Expr::Number("4")
                        )),
                        expr_inst!(Inst::Mul(Expr::Number("5"), Expr::Number("6"),))
                    )),
                    Expr::Number("7")
                )),
            ),
            Inst::Set(
                Expr::Var("ans"),
                expr_inst!(Inst::Mul(
                    expr_inst!(Inst::Add(Expr::Var("ans"), Expr::Number("8"))),
                    expr_inst!(Inst::Add(Expr::Number("9"), Expr::Number("10")))
                )),
            ),
            Inst::Expr(Expr::Var("ans")),
        ];
        assert_eq!(ast, &correct);
    }

    #[test]
    fn cosine_law() {
        let source = examples::cosine_law();
        let mut compiler = Compiler::new(&source);
        let ast = compiler.compile().unwrap();
        let correct = vec![
            Inst::Set(Expr::Var("a"), Expr::Number("7")),
            Inst::Set(Expr::Var("b"), Expr::Number("7")),
            Inst::Set(Expr::Var("c"), Expr::Number("7")),
            Inst::Set(
                Expr::Var("cosRad"),
                expr_inst!(Inst::Div(
                    expr_inst!(Inst::Sub(
                        expr_inst!(Inst::Add(
                            expr_inst!(Inst::Pow(Expr::Var("a"), Expr::Number("2"))),
                            expr_inst!(Inst::Pow(Expr::Var("b"), Expr::Number("2"))),
                        )),
                        expr_inst!(Inst::Pow(Expr::Var("c"), Expr::Number("2"))),
                    )),
                    expr_inst!(Inst::Mul(
                        expr_inst!(Inst::Mul(Expr::Number("2"), Expr::Var("a"))),
                        Expr::Var("b")
                    )),
                )),
            ),
            Inst::Set(
                Expr::Var("deg"),
                expr_inst!(Inst::Mul(
                    expr_inst!(Inst::Div(
                        Expr::FunCall("acos", vec![Expr::Var("cosRad")]),
                        Expr::Var("pi")
                    )),
                    Expr::Number("180")
                )),
            ),
            Inst::Expr(Expr::Var("deg")),
        ];
        assert_eq!(ast, &correct);
    }
}
