#[derive(Debug)]
pub enum Token {
    Keyword(Keyword),
    Number(i64),
    Ident(String),
    Semicolon,
    Operator(Operator),
}

#[derive(Debug)]
pub enum Operator {
    LeftParenthesis,
    RightParenthesis,
    LeftCurlyBracket,
    RightCurlyBracket,
    Equality,
    Plus,
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
}

impl Keyword {
    pub fn parse(value: impl AsRef<str>) -> Option<Self> {
        let keyword = match value.as_ref() {
            "fn" => Keyword::Function,
            "return" => Keyword::Return,
            "let" => Keyword::Let,
            _ => return None,
        };

        Some(keyword)
    }
}
