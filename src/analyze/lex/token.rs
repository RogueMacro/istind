#[derive(Debug)]
pub enum Token {
    Keyword(Keyword),
    Number(i64),
    Ident(String),
    Semicolon,
    Operator(Operator),
}

#[derive(Debug, Clone, Copy)]
pub enum Operator {
    LeftParenthesis,
    RightParenthesis,
    LeftCurlyBracket,
    RightCurlyBracket,
    Equality,
    Plus,
    Minus,
    Star,
    Slash,
}

impl Operator {
    pub fn parse(c: char) -> Option<Self> {
        let op = match c {
            '(' => Self::LeftParenthesis,
            ')' => Self::RightParenthesis,
            '{' => Self::LeftCurlyBracket,
            '}' => Self::RightCurlyBracket,
            '=' => Self::Equality,
            '+' => Self::Plus,
            '-' => Self::Minus,
            '*' => Self::Star,
            '/' => Self::Slash,
            _ => return None,
        };

        Some(op)
    }
}

#[derive(Debug)]
pub enum Keyword {
    Function,
    Return,
    Let,
    Int,
    Bool,
    Char,
}

impl Keyword {
    pub fn parse(value: impl AsRef<str>) -> Option<Self> {
        let keyword = match value.as_ref() {
            "fn" => Keyword::Function,
            "return" => Keyword::Return,
            "let" => Keyword::Let,
            "int" => Keyword::Int,
            "bool" => Keyword::Bool,
            "char" => Keyword::Char,
            _ => return None,
        };

        Some(keyword)
    }
}
