use std::{collections::HashMap, iter::Peekable};


use crate::{
    lexer::{Lexer, Meta, Token},
    Json,
};

/// Parse = Json EOF
///
/// Json = Object | Array | Primary
///
/// Object = '{' ( STRING ':' Json [',' STRING ':' Json]* )? '}'
///
/// Array = '[' (Json [',' Json] )? ']'
///
/// Primary = "true" | "false" | "null" | STRING | NUMBER

#[derive(Debug)]
pub struct Error {
    msg: String,
    line: usize,
    column: usize,
    // token: Token,
}



type ParseResult = Result<Json, Error>;

pub struct Parser<'a> {
    lexer: Peekable<Lexer<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Parser<'a> {
        Parser { lexer: lexer.peekable() }
    }
    pub fn parse(&mut self) -> ParseResult {
        let res = self.json()?;
        self.consume(&Meta::Eof)?;
        Ok(res)
    }

    fn advance_unchecked(&mut self) -> Token {
        let Some(res) = self.lexer.next() else {
            panic!("Impossible")
        };
        res
    }

    fn peek_unchecked(&mut self) -> &Token {
        let Some(res) = self.lexer.peek() else {
            panic!("Impossible")
        };
        res
    }

    fn consume(&mut self, expect_lexeme: &Meta<'a>) -> Result<Token, Error> {
        let Token { lexeme, line, column, .. } = self.peek_unchecked();
        if lexeme.meta_type() == expect_lexeme.meta_type() {
            return Ok(self.lexer.next().unwrap())
        }
        
        let line = *line;
        let column = *column;
        match lexeme {
            &Meta::Error(msg) => {
                Err(Error { msg: String::from(msg), line, column })
            }
            _ => {
                Err(Error { msg: format!("Expect {}, got {}", expect_lexeme.meta_type(), lexeme.meta_type()), line, column })
            }
        }
    }

    fn json(&mut self) -> ParseResult {
        let Token { lexeme, .. } = self.peek_unchecked();
        match lexeme {
            Meta::LeftBrace => self.object(),
            Meta::LeftSquare => self.array(),
            _ => self.primary()
        }
    }

    fn read_kv(&mut self) -> Result<(String, Json), Error> {
        let Token { lexeme: Meta::StringLiteral(key), .. } = self.consume(&Meta::StringLiteral(""))? else {
            panic!("Impossible")
        };
        let key = String::from(key);
        self.consume(&Meta::Colon)?;
        let value = self.json()?;
        Ok((key, value))
    }

    fn object(&mut self) -> ParseResult {
        self.consume(&Meta::LeftBrace)?;
        if self.peek_unchecked().lexeme == Meta::RightBrace {
            self.consume(&Meta::RightBrace)?;
            return Ok(Json::Object(HashMap::with_capacity(0)));
        } 

        let mut result: HashMap<String, Json> = HashMap::new();
        let (k, v) = self.read_kv()?;
        result.insert(k, v);

        while self.peek_unchecked().lexeme == Meta::Comma {
            self.advance_unchecked();
            let (k, v) = self.read_kv()?;
            result.insert(k, v);
        }
        self.consume(&Meta::RightBrace)?;
        Ok(Json::Object(result))
    }
    fn array(&mut self) -> ParseResult {
        self.consume(&Meta::LeftSquare)?;
        if self.peek_unchecked().lexeme == Meta::RightSquare {
            self.consume(&Meta::RightSquare)?;
            return Ok(Json::Array(Vec::with_capacity(0)));
        } 
        
        let mut result: Vec<Json> = Vec::new();
        let v = self.json()?;
        result.push(v);

        while self.peek_unchecked().lexeme == Meta::Comma {
            self.advance_unchecked();
            let v = self.json()?;
            result.push(v);
        }
        self.consume(&Meta::RightSquare)?;
        Ok(Json::Array(result))
    }
    fn primary(&mut self) -> ParseResult {
        let Token { lexeme, line, column, .. } = self.advance_unchecked();
        Ok(match lexeme {
            Meta::NullLiteral => Json::Null,
            Meta::BoolLiteral(bool) => Json::Bool(bool),
            Meta::StringLiteral(str) => Json::String(String::from(str)),
            Meta::NumberLiteral(num) => Json::Number(num),
            Meta::Error(msg) => {
                return Err(Error { msg: String::from(msg), line, column })
            }
            _ => {
                return Err(Error { msg: format!("Expect Primary, got {}", lexeme.meta_type()), line, column })
            }
        })
    }
}


#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use indoc::indoc;
    use super::Parser;

    #[test]
    fn literals() {
        let json = indoc!(
            r##"
        [
            {
                "name": "Haruka",
                "age": 16,
                "friends": ["Chihaya", "Miki"]
            },
            {
                "name": "Chihaya",
                "age": 16,
                "friends": ["Haruka", "Miki"]
            }
        ]
        "##
        );
        let mut parser = Parser::new(Lexer::new(json));
        dbg!(parser.parse());
    }
}