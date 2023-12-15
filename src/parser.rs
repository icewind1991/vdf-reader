use logos::Logos;
use parse_display::Display;
use std::str;

/// Parser token.
#[derive(PartialEq, Debug, Logos, Display, Clone)]
#[logos(skip r"[ \t\n\f\r]+")] // whitespace
#[logos(skip r"//[^\n]*")] // comments
pub enum Token {
    /// A group is starting.
    #[token("{")]
    #[display("start of group")]
    GroupStart,
    /// A group is ending.
    #[token("}")]
    #[display("end of group")]
    GroupEnd,
    /// An enclosed or bare item.
    #[regex("(\"([^\"\\\\]|\\\\.)*\")|([^# \t\n{}\"][^ \"\t\n]*)", priority = 0)]
    #[display("item")]
    Item,
    /// An enclosed or bare statement.
    #[regex("(\"#([^\"\\\\]|\\\\.)*\")|(#[^ \"\t\n]+)")]
    #[display("statement")]
    Statement,
}

#[cfg(test)]
mod tests {
    use super::Token;
    use logos::Logos;

    fn get_token(input: &str) -> Option<Result<Token, <Token as Logos>::Error>> {
        let mut lex = Token::lexer(input);
        lex.next()
    }

    fn get_tokens(input: &str) -> Result<Vec<(Token, &str)>, <Token as Logos>::Error> {
        Token::lexer(input)
            .spanned()
            .map(|(res, span)| res.map(|token| (token, &input[span])))
            // .map(|res| dbg!(res))
            .collect()
    }

    #[test]
    fn next() {
        assert_eq!(get_token("test"), Some(Ok(Token::Item)));
        assert_eq!(get_token("\"test\""), Some(Ok(Token::Item)));
        assert_eq!(get_token("\"\""), Some(Ok(Token::Item)));
        assert_eq!(get_token("\"\" "), Some(Ok(Token::Item)));
        assert_eq!(get_token("#test"), Some(Ok(Token::Statement)));
        assert_eq!(get_token("\"#test\""), Some(Ok(Token::Statement)));
        assert_eq!(get_token("{"), Some(Ok(Token::GroupStart)));
        assert_eq!(get_token("}"), Some(Ok(Token::GroupEnd)));
        assert_eq!(get_token("//test more"), None);

        assert_eq!(get_token("test"), Some(Ok(Token::Item)));
        assert_eq!(get_token("#test"), Some(Ok(Token::Statement)));

        assert_eq!(get_token("lol wut"), Some(Ok(Token::Item)));
        assert_eq!(get_token("#lol wut"), Some(Ok(Token::Statement)));

        assert_eq!(get_token("lol{"), Some(Ok(Token::Item)));
        assert_eq!(get_token("#lol{"), Some(Ok(Token::Statement)));

        assert_eq!(get_token("lol}"), Some(Ok(Token::Item)));
        assert_eq!(get_token("#lol}"), Some(Ok(Token::Statement)));

        assert_eq!(get_token("\"test\""), Some(Ok(Token::Item)));
        assert_eq!(get_token("\"#test\""), Some(Ok(Token::Statement)));

        assert_eq!(get_token("\"te\\\"st\""), Some(Ok(Token::Item)));
        assert_eq!(get_token("\"te\\st\""), Some(Ok(Token::Item)));
        assert_eq!(get_token("\"#te\\\"st\""), Some(Ok(Token::Statement)));
    }

    #[test]
    fn tokenize() {
        assert_eq!(
            get_tokens(
                r#"foo { // eol comment
                "asd" "bar"
                // a comment
                #include other
                empty ""
            }"#
            ),
            Ok(vec![
                (Token::Item, "foo"),
                (Token::GroupStart, "{"),
                (Token::Item, r#""asd""#),
                (Token::Item, r#""bar""#),
                (Token::Statement, r#"#include"#),
                (Token::Item, r#"other"#),
                (Token::Item, r#"empty"#),
                (Token::Item, r#""""#),
                (Token::GroupEnd, "}")
            ])
        )
    }
}
