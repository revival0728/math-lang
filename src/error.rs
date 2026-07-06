pub trait Error {
    fn msg(&self) -> &String;
    fn line(&self) -> u32;
    fn col(&self) -> Option<u32>;
    fn err_type(&self) -> &'static str;

    fn col_to_string(&self) -> String {
        if let Some(col) = self.col() {
            format!(":{}: ", col)
        } else {
            format!(": ")
        }
    }
    fn all_info(&self) -> String {
        format!(
            "{} at line:{}{}{}",
            self.err_type(),
            self.line(),
            self.col_to_string(),
            self.msg()
        )
    }
    fn no_loc_info(&self) -> String {
        format!("{}: {}", self.err_type(), self.msg())
    }
}

#[derive(Debug, Default, Clone)]
pub struct CompilerError {
    pub line: u32,
    pub col: u32,
    pub byte_idx: usize,
    pub msg: String,
}

#[derive(Debug, Default, Clone)]
pub struct RuntimeError {
    pub line: usize,
    pub msg: String,
}

#[derive(Debug, Clone)]
pub enum GlobalError {
    RE(RuntimeError),
    CE(CompilerError),
}

macro_rules! impl_global_error {
    ($fname:ident) => {
        impl GlobalError {
            pub fn $fname(&self) -> String {
                match self {
                    Self::CE(err) => err.$fname(),
                    Self::RE(err) => err.$fname(),
                }
            }
        }
    };
}
impl_global_error!(all_info);
impl_global_error!(no_loc_info);

impl Error for CompilerError {
    fn col(&self) -> Option<u32> {
        Some(self.col)
    }
    fn line(&self) -> u32 {
        self.line
    }
    fn msg(&self) -> &String {
        &self.msg
    }
    fn err_type(&self) -> &'static str {
        "Compiler Error"
    }
}

impl Error for RuntimeError {
    fn col(&self) -> Option<u32> {
        None
    }
    fn line(&self) -> u32 {
        self.line as u32
    }
    fn msg(&self) -> &String {
        &self.msg
    }
    fn err_type(&self) -> &'static str {
        "Runtime Error"
    }
}

impl CompilerError {
    pub fn new(loc: &lexgen_util::Loc, msg: String) -> Self {
        CompilerError {
            line: loc.line,
            col: loc.col,
            byte_idx: loc.byte_idx,
            msg,
        }
    }
    pub fn new_with_loc(loc: &lexgen_util::Loc) -> Self {
        CompilerError {
            line: loc.line,
            col: loc.col,
            byte_idx: loc.byte_idx,
            msg: String::new(),
        }
    }
    pub fn new_with_literal(loc: &lexgen_util::Loc, msg: &'static str) -> Self {
        CompilerError {
            line: loc.line,
            col: loc.col,
            byte_idx: loc.byte_idx,
            msg: msg.to_string(),
        }
    }
}
