#[derive(Debug, PartialEq)]
pub enum Meta<'a> {
    LeftBrace,
    LeftSquare,
    RightBrace,
    RightSquare,
    Comma,
    Colon,
    StringLiteral(&'a str),
    BoolLiteral(bool),
    NumberLiteral(f64),
    NullLiteral,
    Eof,
    Error(&'static str),
}

impl<'a> Meta<'a> {
    pub fn meta_type(&self) -> &'static str {
        match self {
            Meta::BoolLiteral(_) => "bool",
            Meta::StringLiteral(_) => "string",
            Meta::NumberLiteral(_) => "number",
            Meta::Error(_) => "Error",
            Meta::LeftBrace => "'{'", Meta::RightBrace => "'}'", Meta::LeftSquare => "'['", Meta::RightSquare => "']'",
            Meta::Colon => "':'",
            Meta::NullLiteral => "null",
            Meta::Comma => ",",
            Meta::Eof => "EOF"
        }
    }
}



#[derive(Debug)]
pub struct Token<'a> {
    pub lexeme: Meta<'a>,
    pub literal: &'a str,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug)]
pub struct Lexer<'a> {
    start: &'a str,
    line: usize,
    column: usize,
    done: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Lexer {
            start: source,
            line: 1,
            column: 1,
            done: false,
        }
    }

    pub fn next_token(&mut self) -> Option<Token<'a>> {
        if self.done {
            return None;
        }

        self.skip_whitespaces();
        let Some(c) = self.start.chars().next() else {
            return Some(self.mk_eof());
        };

        Some(match c {
            '[' => self.mk_token(Meta::LeftSquare, "["),
            ']' => self.mk_token(Meta::RightSquare, "]"),
            '{' => self.mk_token(Meta::LeftBrace, "{"),
            '}' => self.mk_token(Meta::RightBrace, "}"),
            ',' => self.mk_token(Meta::Comma, ","),
            ':' => self.mk_token(Meta::Colon, ":"),
            _ if c.is_digit(10) || c == '-' => self.number(),
            '"' => self.string(),
            _ if c.is_alphabetic() => self.identifier(),
            _ => self.mk_error("Unexpected character", &self.start[0..c.len_utf8()]),
        })
    }

    fn identifier(&mut self) -> Token<'a> {
        let idx = self
            .start
            .chars()
            .take_while(|c| c.is_ascii_alphabetic())
            .take(5)
            .count();
        let id = &self.start[..idx];
        match id {
            "true" => self.mk_token(Meta::BoolLiteral(true), id),
            "false" => self.mk_token(Meta::BoolLiteral(false), id),
            "null" => self.mk_token(Meta::NullLiteral, id),
            _ => self.mk_error("Unexpected identifier", id),
        }
    }

    fn string(&mut self) -> Token<'a> {
        let mut chars = self.start.chars().peekable();
        let mut idx = 1;
        chars.next();

        let mut skip_next = false;
        while let Some(&c) = chars.peek() {
            if c == '\n' {
                return self.mk_error("Unexpected newline", &self.start[..idx]);
            }

            idx += c.len_utf8();
            chars.next();
            if skip_next {
                skip_next = false;
                continue;
            }
            match c {
                '\\' => {
                    skip_next = true;
                }
                '"' => {
                    if skip_next {
                        skip_next = false;
                        continue;
                    }
                    return self.mk_token(
                        Meta::StringLiteral(&self.start[1..idx - 1]),
                        &self.start[..idx],
                    );
                }
                _ => (),
            };
        }
        return self.mk_error("Undetermined string literal", &self.start[..idx]);
    }

    fn number(&mut self) -> Token<'a> {
        let mut chars = self.start.chars().peekable();
        let mut idx = 0;

        // 匹配-，检查其后是否是数字
        if chars.peek() == Some(&'-') {
            idx += 1;
            chars.next();
            match chars.peek() {
                None => {
                    return self.mk_error("Unexpected EOF after '-'", &self.start[..idx]);
                }
                Some(c) if !c.is_digit(10) => {
                    return self.mk_error("Expect numeric literal after '-'", &self.start[..idx]);
                }
                _ => (),
            };
        }

        // 匹配.前的数字
        while let Some(&c) = chars.peek() {
            if !c.is_digit(10) {
                break;
            }
            idx += 1;
            chars.next();
        }

        // 如果没有.，直接返回
        if chars.peek() != Some(&'.') {
            let literal = &self.start[..idx];
            // should not fail?
            return self.mk_token(Meta::NumberLiteral(literal.parse().unwrap()), literal);
        }

        idx += 1;
        chars.next();

        match chars.peek() {
            None => {
                return self.mk_error("Unexpected EOF after '.'", &self.start[..idx]);
            }
            Some(c) if !c.is_digit(10) => {
                return self.mk_error("Expect numeric literal after '.'", &self.start[..idx]);
            }
            _ => (),
        };

        while let Some(&c) = chars.peek() {
            if !c.is_digit(10) {
                break;
            }
            idx += 1;
            chars.next();
        }
        let literal = &self.start[..idx];
        // should not fail?
        return self.mk_token(Meta::NumberLiteral(literal.parse().unwrap()), literal);
    }

    fn manipulate_states(&mut self, literal: &'a str) {
        if literal.is_empty() {
            return;
        }
        self.start = &self.start[literal.len()..];
        let mut line_offset = 0;
        let mut next_column = self.column;
        for c in literal.chars() {
            match c {
                '\n' => {
                    line_offset += 1;
                    next_column = 1;
                }
                _ => {
                    next_column += 1;
                }
            }
        }
        self.line += line_offset;
        self.column = next_column;
    }

    fn mk_token(&mut self, meta: Meta<'a>, literal: &'a str) -> Token<'a> {
        let &mut Lexer { line, column, .. } = self;
        let res: Token<'a> = Token {
            line,
            column,
            lexeme: meta,
            literal,
        };
        self.manipulate_states(literal);
        res
    }

    fn mk_eof(&mut self) -> Token<'a> {
        self.done = true;
        self.mk_token(Meta::Eof, "")
    }

    fn mk_error(&mut self, msg: &'static str, literal: &'a str) -> Token<'a> {
        self.done = true;
        self.mk_token(Meta::Error(msg), literal)
    }

    fn skip_whitespaces(&mut self) {
        if self.done {
            return;
        }
        let space_counts = self.start.chars().take_while(|c| c.is_whitespace()).count();
        if space_counts != 0 {
            self.mk_token(Meta::Eof, &self.start[0..space_counts]);
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

#[cfg(test)]
mod tests {
    use super::Lexer;
    use crate::lexer::{self, Meta, Token};
    use indoc::indoc;

    fn test() {
        let mut token;
        {
            let mut lexer = Lexer::new("ss");
            token = lexer.next_token().unwrap();
        }
        token;
        ()
    }


    fn assert_token<'a>(token: Option<Token<'a>>, expect: (Meta<'a>, usize, usize, &'a str)) {
        if token.is_none() {}
        let Some(Token {
            lexeme,
            line,
            column,
            literal,
        }) = token
        else {
            panic!("Expect Some")
        };
        assert_eq!((lexeme, line, column, literal), expect)
    }

    fn assert_token_seq<'a>(lexer: &'a mut Lexer, expect: Vec<(Meta<'a>, usize, usize, &'a str)>) {
        for ele in expect {
            assert_token(lexer.next(), ele);
        }
    }

    #[test]
    fn numbers() {
        for str in ["42", "42.0", "42.00"] {
            assert_token(
                Lexer::new(str).next(),
                (Meta::NumberLiteral(42.0), 1, 1, str),
            );
            let str = format!("-{str}");
            assert_token(
                Lexer::new(str.as_str()).next(),
                (Meta::NumberLiteral(-42.0), 1, 1, str.as_str()),
            )
        }

        let mut lexer = Lexer::new(" 42  ");
        assert_token(lexer.next(), (Meta::NumberLiteral(42.0), 1, 2, "42"));
        assert!(!lexer.done);
        assert_eq!(lexer.next().unwrap().lexeme, Meta::Eof);
        assert!(lexer.done);
        assert!(lexer.next().is_none());

        let mut lexer = Lexer::new("-");
        let Token {
            lexeme,
            line,
            column,
            literal,
        } = lexer.next().unwrap();
        let Meta::Error(_) = lexeme else { panic!() };

        assert_eq!((line, column, literal), (1, 1, "-"));
    }
    #[test]
    fn strings() {
        assert_token(
            Lexer::new(r#"  "泥嚎"  "#).next(),
            (Meta::StringLiteral("泥嚎"), 1, 3, r#""泥嚎""#),
        );
        let Token { lexeme, .. } = Lexer::new(r#"  "泥嚎  "#).next().unwrap();
        let Meta::Error(_) = lexeme else { panic!() };
    }

    #[test]
    fn it_works() {
        let mut lexer = Lexer::new("[12450]\n[-2.00] null true false");

        assert_token_seq(
            &mut lexer,
            vec![
                (Meta::LeftSquare, 1, 1, "["),
                (Meta::NumberLiteral(12450.0), 1, 2, "12450"),
                (Meta::RightSquare, 1, 7, "]"),
                (Meta::LeftSquare, 2, 1, "["),
                (Meta::NumberLiteral(-2.0), 2, 2, "-2.00"),
                (Meta::RightSquare, 2, 7, "]"),
                (Meta::NullLiteral, 2, 9, "null"),
                (Meta::BoolLiteral(true), 2, 14, "true"),
                (Meta::BoolLiteral(false), 2, 19, "false"),
                (Meta::Eof, 2, 24, ""),
            ],
        );
        let json = indoc!(
            r##"
        {
            "name": "Haruka",
            "age": 16,
            "friends": ["Chihaya", "Miki"]
        }
        "##
        );
        let mut lexer = Lexer::new(json);
        assert_token_seq(
            &mut lexer,
            vec![
                (Meta::LeftBrace, 1, 1, "{"),
                (Meta::StringLiteral("name"), 2, 5, "\"name\""),
                (Meta::Colon, 2, 11, ":"),
                (Meta::StringLiteral("Haruka"), 2, 13, "\"Haruka\""),
                (Meta::Comma, 2, 21, ","),
                (Meta::StringLiteral("age"), 3, 5, "\"age\""),
                (Meta::Colon, 3, 10, ":"),
                (Meta::NumberLiteral(16.0), 3, 12, "16"),
                (Meta::Comma, 3, 14, ","),
                (Meta::StringLiteral("friends"), 4, 5, "\"friends\""),
                (Meta::Colon, 4, 14, ":"),
                (Meta::LeftSquare, 4, 16, "["),
                (Meta::StringLiteral("Chihaya"), 4, 17, "\"Chihaya\""),
                (Meta::Comma, 4, 26, ","),
                (Meta::StringLiteral("Miki"), 4, 28, "\"Miki\""),
                (Meta::RightSquare, 4, 34, "]"),
                (Meta::RightBrace, 5, 1, "}"),
                (Meta::Eof, 6, 1, ""),
            ],
        );
    }
}
