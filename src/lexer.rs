use lexgen::lexer;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub enum Token<'input> {
    #[default]
    None,
    Pow,
    Plus,
    Minus,
    Star,
    Slash,
    Mod,
    Eq,
    LParen,
    RParen,
    Comma,
    Newline,
    Var(&'input str),
    Number(&'input str),

    // special tokens
    NegSign,
    FunCall(&'input str),
}

#[derive(Debug, Default, Clone)]
pub struct LexerState {}

lexer! {
    pub Lexer(LexerState) -> Token<'input>;

    let whitespace = [' ' '\t'];
    let newline = '\n' | "\r\n";

    rule Init {
        $whitespace,

        "+" = Token::Plus,
        "-" = Token::Minus,
        "*" = Token::Star,
        "/" = Token::Slash,
        "^" = Token::Pow,
        "=" = Token::Eq,
        "(" = Token::LParen,
        ")" = Token::RParen,
        "," = Token::Comma,
        "mod" = Token::Mod,
        $newline = Token::Newline,

        let var_init = ['a'-'z' 'A'-'Z' '_'];
        let var_subseq = $var_init | ['0'-'9'];
        $var_init $var_subseq* => |lexer| {
            let match_ = lexer.match_();
            lexer.return_(Token::Var(match_))
        },

        let digit = ['0'-'9'];
        $digit+ (('.' $digit+)? (('e' | 'E') ('+' | '-')? $digit+)?)? => |lexer| {
            let match_ = lexer.match_();
            lexer.return_(Token::Number(match_))
        },
    }
}

#[cfg(test)]
mod test {
    use crate::lexer::{Lexer, Token};
    use crate::test::examples;

    #[test]
    fn simple_digit() {
        let lexer = Lexer::new("1 1.2 1e2 1.2e2 1.2e-2");

        let tokens: Vec<Token> = lexer.into_iter().map(|e| e.unwrap().1).collect();
        let correct: Vec<Token> = vec![
            Token::Number("1"),
            Token::Number("1.2"),
            Token::Number("1e2"),
            Token::Number("1.2e2"),
            Token::Number("1.2e-2"),
        ];
        assert_eq!(tokens, correct);
    }

    #[test]
    fn basic() {
        let example = examples::basic();
        let lexer = Lexer::new(&example);

        let tokens: Vec<Token> = lexer.into_iter().map(|e| e.unwrap().1).collect();
        let correct: Vec<Token> = vec![
            Token::Var("ans"),
            Token::Eq,
            Token::LParen,
            Token::LParen,
            Token::Number("1"),
            Token::Star,
            Token::Number("2"),
            Token::Plus,
            Token::Number("3"),
            Token::RParen,
            Token::Star,
            Token::Number("4"),
            Token::RParen,
            Token::Plus,
            Token::Number("5"),
            Token::Star,
            Token::Number("6"),
            Token::Plus,
            Token::Number("7"),
            Token::Newline,
            Token::Var("ans"),
            Token::Eq,
            Token::LParen,
            Token::Var("ans"),
            Token::Plus,
            Token::Number("8"),
            Token::RParen,
            Token::Star,
            Token::LParen,
            Token::Number("9"),
            Token::Plus,
            Token::Number("10"),
            Token::RParen,
            Token::Newline,
            Token::Newline,
            Token::Var("ans"),
        ];
        assert_eq!(tokens, correct, "example basic test failed.");
    }

    #[test]
    fn cosine_law() {
        let example = examples::cosine_law();
        let lexer = Lexer::new(&example);

        let tokens: Vec<Token> = lexer.into_iter().map(|e| e.unwrap().1).collect();
        let correct: Vec<Token> = vec![
            Token::Var("a"),
            Token::Eq,
            Token::Number("7"),
            Token::Newline,
            Token::Var("b"),
            Token::Eq,
            Token::Number("7"),
            Token::Newline,
            Token::Var("c"),
            Token::Eq,
            Token::Number("7"),
            Token::Newline,
            Token::Newline,
            Token::Var("cosRad"),
            Token::Eq,
            Token::LParen,
            Token::Var("a"),
            Token::Pow,
            Token::Number("2"),
            Token::Plus,
            Token::Var("b"),
            Token::Pow,
            Token::Number("2"),
            Token::Minus,
            Token::Var("c"),
            Token::Pow,
            Token::Number("2"),
            Token::RParen,
            Token::Slash,
            Token::LParen,
            Token::Number("2"),
            Token::Var("a"),
            Token::Var("b"),
            Token::RParen,
            Token::Newline,
            Token::Var("deg"),
            Token::Eq,
            Token::Var("acos"),
            Token::LParen,
            Token::Var("cosRad"),
            Token::RParen,
            Token::Slash,
            Token::Var("pi"),
            Token::Star,
            Token::Number("180"),
            Token::Newline,
            Token::Newline,
            Token::Var("deg"),
        ];
        assert_eq!(tokens, correct, "example cosine-law test failed.");
    }
}
