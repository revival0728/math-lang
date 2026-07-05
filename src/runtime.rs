use crate::comiler::Inst;

#[derive(Debug, Default, Clone)]
pub enum VarType {
    #[default]
    None,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
    BigInt,
}

#[derive(Debug, Default, Clone)]
pub struct Var {
    type_: VarType,
    data: Vec<u8>,
}

#[derive(Debug, Default, Clone)]
pub struct Function<'input> {
    data: Vec<Inst<'input>>,
}
