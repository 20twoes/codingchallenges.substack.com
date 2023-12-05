// JSON parser
// Reference:  https://www.json.org/json-en.html
// My answer to:  https://codingchallenges.substack.com/p/coding-challenge-2
use clap::{CommandFactory, Parser};
use is_terminal::IsTerminal as _;
use std::{
    fs::File,
    io::{stdin, BufRead, BufReader},
    path::PathBuf,
};

#[derive(Parser)]
#[command(arg_required_else_help = true)]
struct Cli {
    /// The path to the file to read, use - to read from stdin (must not be a tty)
    file: PathBuf,
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    LeftBrace,
    RightBrace,
    Colon,
    Comma,
    String(String),
    True,
    False,
    Null,
    Number(String),
    LeftBracket,
    RightBracket,
    EOF,
}

#[derive(Debug, PartialEq)]
struct TokenizeError;

#[derive(Debug, PartialEq)]
struct ParseError;

fn main() {
    let args = Cli::parse();
    let mut file = args.file;

    // Read input from file or stdin
    let buffer: Box<dyn BufRead> = if file == PathBuf::from("-") {
        if stdin().is_terminal() {
            Cli::command().print_help().unwrap();
            std::process::exit(2);
        }
        file = PathBuf::from("<stdin>");
        println!("Using {}", file.display());
        Box::new(BufReader::new(stdin().lock()))
    } else {
        Box::new(BufReader::new(File::open(&file).unwrap()))
    };

    // Perform lexical analysis to get a stream of valid tokens
    let tokens = match tokenize(buffer) {
        Ok(t) => t,
        Err(_) => {
            eprintln!("illegal character found");
            std::process::exit(1)
        },
    };

    // Check for empty string
    if tokens.len() == 0 {
        eprintln!("Did not find anything to parse");
        std::process::exit(1)
    }

    // Parse token stream according to JSON rules
    match parse_tokens(&tokens[..]) {
        Ok(_) => {
            println!("Parse successful");
            std::process::exit(0)
        },
        Err(_) => {
            println!("Parse failed");
            std::process::exit(1)
        },
    }
}

fn tokenize(buf_reader: impl BufRead) -> Result<Vec<Token>, TokenizeError> {
    let mut tokens = Vec::new();

    for line in buf_reader.lines() {
        let l = line.unwrap();
        let mut iter = l.chars().peekable();

        while let Some(ch) = iter.next() {
            let token_value = match ch {
                '{' => Some(Token::LeftBrace),
                '}' => Some(Token::RightBrace),
                '[' => Some(Token::LeftBracket),
                ']' => Some(Token::RightBracket),
                ':' => Some(Token::Colon),
                ',' => Some(Token::Comma),
                '"' => {
                    let mut string = ch.to_string();
                    // Consume line until we reach the terminal quotation mark
                    // TODO: Support escaped quotes
                    while let Some(i) = iter.next() {
                        match i {
                            '"' => {
                                string.push_str(&i.to_string());
                                break;
                            },
                            _ => string.push_str(&i.to_string()),
                        }
                    }

                    Some(Token::String(string))
                },
                't' => {
                    let word = [iter.next(), iter.next(), iter.next()].map(|i| i.unwrap());
                    if word == ['r', 'u', 'e'] {
                        Some(Token::True)
                    } else {
                        return Err(TokenizeError);
                    }
                },
                'f' => {
                    let word = [
                        iter.next(),
                        iter.next(),
                        iter.next(),
                        iter.next(),
                    ].map(|i| i.unwrap());
                    if word == ['a', 'l', 's', 'e'] {
                        Some(Token::False)
                    } else {
                        return Err(TokenizeError);
                    }
                },
                'n' => {
                    let word = [
                        iter.next(),
                        iter.next(),
                        iter.next(),
                    ].map(|i| i.unwrap());
                    if word == ['u', 'l', 'l'] {
                        Some(Token::Null)
                    } else {
                        return Err(TokenizeError);
                    }
                },
                digit if digit.is_ascii_digit() => {
                    let mut value = digit.to_string();
                    while let Some(i) = iter.peek() {
                        match i {
                            i if i.is_ascii_digit() => {
                                value.push_str(&i.to_string());
                                // We only go forward if we're still in a number
                                iter.next();
                            }
                            _ => break,
                        }
                    }
                    Some(Token::Number(value))
                },
                ' ' => None, // Ignore whitespace
                _ => return Err(TokenizeError), // Any other character is not valid in this context
            };

            if let Some(t) = token_value {
                tokens.push(t);
            }
        }
    }

    Ok(tokens)
}

// Parse JSON value.
// A value can be any of the following:
// - object
// - array
// - string
// - number
// - "true"
// - "false"
// - "null"
//
// Apparently there is some disparity between the JSON reference and the test suite
// The test suite says that the payload must be in an object or array,
// but that's not what I gathered from the spec.
// https://www.json.org/json-en.html
// http://www.json.org/JSON_checker/test.zip

fn is_simple_value(token: &Token) -> bool {
    match token {
        Token::String(_) | Token::True | Token::False | Token::Null | Token::Number(_) => true,
        _ => false,
    }
}

#[derive(Debug)]
struct JsonParser<'a> {
    iter: core::iter::Peekable<core::slice::Iter<'a, Token>>,
}

impl<'a> JsonParser<'a> {
    fn new(tokens: &[Token]) -> JsonParser {
        JsonParser {
            iter: tokens.iter().peekable(),
        }
    }

    fn read(&mut self) -> &Token {
        if let Some(t) = self.iter.next() {
            t
        } else {
            &Token::EOF
        }
    }

    fn peek(&mut self) -> &Token {
        if let Some(t) = self.iter.peek() {
            t
        } else {
            &Token::EOF
        }
    }

    fn is_eof(&mut self) -> bool {
        if let None = self.iter.peek() {
            true
        } else {
            false
        }
    }

    fn read_left_brace(&mut self) -> bool {
        let token = self.read();
        if let Token::LeftBrace = token {
            true
        } else {
            false
        }
    }

    fn read_right_brace(&mut self) -> bool {
        let token = self.read();
        if let Token::RightBrace = token {
            true
        } else {
            false
        }
    }

    fn read_object_key(&mut self) -> bool {
        let token = self.read();
        if let Token::String(_) = token {
            true
        } else {
            false
        }
    }

    fn read_colon(&mut self) -> bool {
        let token = self.read();
        if let Token::Colon = token {
            true
        } else {
            false
        }
    }

    fn read_left_bracket(&mut self) -> bool {
        let token = self.read();
        if let Token::LeftBracket = token {
            true
        } else {
            false
        }
    }

    fn read_right_bracket(&mut self) -> bool {
        let token = self.read();
        if let Token::RightBracket = token {
            true
        } else {
            false
        }
    }

}


fn parse_tokens(tokens: &[Token]) -> Result<(), ParseError> {
    let mut parser = JsonParser::new(tokens);

    let token = parser.peek();

    let valid = match token {
        t if is_simple_value(t) => {
            parser.read();
            true
        },
        Token::LeftBrace => {
            new_parse_object(&mut parser)
        }
        Token::LeftBracket => {
            new_parse_array(&mut parser)
        },
        _ => false,
    };

    if valid && parser.is_eof() {
        Ok(())
    } else {
        Err(ParseError)
    }
}

fn new_parse_object(parser: &mut JsonParser) -> bool {
    parser.read_left_brace();

    match parser.peek() {
        Token::RightBrace => true, // Empty object
        Token::String(_) => new_parse_object_member(parser),
        _ => false,
    };

    parser.read_right_brace()
}

fn new_parse_object_member(parser: &mut JsonParser) -> bool {
    let mut valid = parser.read_object_key();
    if !valid {
        return false;
    }

    valid = parser.read_colon();
    if !valid {
        return false;
    }

    valid = new_parse_object_value(parser);
    if !valid {
        return false;
    }

    let token = parser.peek();

    match token {
        Token::Comma => {
            parser.read();
            new_parse_object_member(parser)
        },
        Token::RightBrace => true,
        _ => false,
    }
}

fn new_parse_object_value(parser: &mut JsonParser) -> bool {
    match parser.peek() {
        t if is_simple_value(t) => {
            parser.read();
            true
        },
        Token::LeftBrace => new_parse_object(parser),
        Token::LeftBracket => new_parse_array(parser),
        _ => false,
    }
}

fn new_parse_array(parser: &mut JsonParser) -> bool {
    parser.read_left_bracket();

    let valid = match parser.peek() {
        Token::RightBracket => true, // Empty array
        _ => new_parse_array_element(parser),
    };

    if valid {
        parser.read_right_bracket()
    } else {
        false
    }
}

fn new_parse_array_element(parser: &mut JsonParser) -> bool {
    let valid = match parser.peek() {
        t if is_simple_value(t) => {
            parser.read();
            true
        },
        Token::LeftBrace => new_parse_object(parser),
        Token::LeftBracket => new_parse_array(parser),
        _ => false,
    };

    if !valid {
        return false;
    }

    match parser.peek() {
        Token::Comma => {
            parser.read();
            new_parse_array_element(parser)
        },
        // No more elements
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use assert_cmd::prelude::*;
    use std::io::Cursor;
    use super::*;

    #[test]
    fn check_tokenize_empty_string() {
        let result = tokenize(Cursor::new(b"")).unwrap();
        assert_eq!(result, []);
    }

    #[test]
    fn check_tokenize_empty_object() {
        let result = tokenize(Cursor::new(b"{}")).unwrap();
        assert_eq!(result, [
            Token::LeftBrace,
            Token::RightBrace,
        ]);
    }

    #[test]
    fn check_tokenize_object() {
        let result = tokenize(Cursor::new(b"{\"key\": \"value\"}")).unwrap();
        assert_eq!(result, [
            Token::LeftBrace,
            Token::String("\"key\"".to_string()),
            Token::Colon,
            Token::String("\"value\"".to_string()),
            Token::RightBrace,
        ])
    }

    #[test]
    fn check_tokenize_object_multiline() {
        let result = tokenize(Cursor::new(b"{\n  \"key\": \"value\",\n  \"key2\": \"value\"\n}")).unwrap();
        assert_eq!(result, [
            Token::LeftBrace,
            Token::String("\"key\"".to_string()),
            Token::Colon,
            Token::String("\"value\"".to_string()),
            Token::Comma,
            Token::String("\"key2\"".to_string()),
            Token::Colon,
            Token::String("\"value\"".to_string()),
            Token::RightBrace,
        ])
    }

    #[test]
    fn check_tokenize_fails_for_unquoted_key() {
        let result = tokenize(Cursor::new(b"{key: \"value\"}")).unwrap_err();
        assert_eq!(result, TokenizeError)
    }

    #[test]
    fn check_tokenize_true() {
        let result = tokenize(Cursor::new(b"{\"key\": true}")).unwrap();
        assert_eq!(result, [
            Token::LeftBrace,
            Token::String("\"key\"".to_string()),
            Token::Colon,
            Token::True,
            Token::RightBrace,
        ])
    }

    #[test]
    fn check_tokenize_false() {
        let result = tokenize(Cursor::new(b"{\"key\": false}")).unwrap();
        assert_eq!(result, [
            Token::LeftBrace,
            Token::String("\"key\"".to_string()),
            Token::Colon,
            Token::False,
            Token::RightBrace,
        ])
    }

    #[test]
    fn check_tokenize_null() {
        let result = tokenize(Cursor::new(b"{\"key\": null}")).unwrap();
        assert_eq!(result, [
            Token::LeftBrace,
            Token::String("\"key\"".to_string()),
            Token::Colon,
            Token::Null,
            Token::RightBrace,
        ])
    }

    #[test]
    fn check_tokenize_number() {
        let result = tokenize(Cursor::new(b"{\"key\": 101}")).unwrap();
        assert_eq!(result, [
            Token::LeftBrace,
            Token::String("\"key\"".to_string()),
            Token::Colon,
            Token::Number("101".to_string()),
            Token::RightBrace,
        ])
    }

    #[test]
    fn check_tokenize_empty_array() {
        let result = tokenize(Cursor::new(b"{\"key\": []}")).unwrap();
        assert_eq!(result, [
            Token::LeftBrace,
            Token::String("\"key\"".to_string()),
            Token::Colon,
            Token::LeftBracket,
            Token::RightBracket,
            Token::RightBrace,
        ])
    }

    #[test]
    fn check_tokenize_array() {
        let result = tokenize(Cursor::new(b"{\"key\": [\"list value\"]}")).unwrap();
        assert_eq!(result, [
            Token::LeftBrace,
            Token::String("\"key\"".to_string()),
            Token::Colon,
            Token::LeftBracket,
            Token::String("\"list value\"".to_string()),
            Token::RightBracket,
            Token::RightBrace,
        ])
    }

    #[test]
    fn check_parse_tokens_string() {
        let tokens = [
            Token::String("\"key\"".to_string()),
        ];
        let result = parse_tokens(&tokens).unwrap();
        assert_eq!(result, ())
    }

    #[test]
    fn check_parse_tokens_empty_object() {
        let tokens = [
            Token::LeftBrace,
            Token::RightBrace,
        ];
        let result = parse_tokens(&tokens).unwrap();
        assert_eq!(result, ())
    }

    #[test]
    fn check_parse_tokens_object() {
        let tokens = [
            Token::LeftBrace,
            Token::String("\"key\"".to_string()),
            Token::Colon,
            Token::String("\"value\"".to_string()),
            Token::RightBrace,
        ];
        let result = parse_tokens(&tokens).unwrap();
        assert_eq!(result, ())
    }

    #[test]
    fn check_parse_tokens_object_trailing_comma() {
        let tokens = [
            Token::LeftBrace,
            Token::String("\"key\"".to_string()),
            Token::Colon,
            Token::String("\"value\"".to_string()),
            Token::Comma,
            Token::RightBrace,
        ];
        let result = parse_tokens(&tokens).unwrap_err();
        assert_eq!(result, ParseError)
    }

    #[test]
    fn check_parse_tokens_object_multiple_keys() {
        let tokens = [
            Token::LeftBrace,
            Token::String("\"key\"".to_string()),
            Token::Colon,
            Token::String("\"value\"".to_string()),
            Token::Comma,
            Token::String("\"key2\"".to_string()),
            Token::Colon,
            Token::String("\"value\"".to_string()),
            Token::RightBrace,
        ];
        let result = parse_tokens(&tokens).unwrap();
        assert_eq!(result, ())
    }

    #[test]
    fn check_parse_tokens_nested_object() {
        let tokens = [
            Token::LeftBrace,
            Token::String("\"key\"".to_string()),
            Token::Colon,
            Token::LeftBrace,
            Token::String("\"key2\"".to_string()),
            Token::Colon,
            Token::String("\"list value\"".to_string()),
            Token::RightBrace,
            Token::RightBrace,
        ];
        let result = parse_tokens(&tokens).unwrap();
        assert_eq!(result, ())
    }

    #[test]
    fn check_parse_tokens_true() {
        let tokens = [
            Token::LeftBrace,
            Token::String("\"key\"".to_string()),
            Token::Colon,
            Token::True,
            Token::RightBrace,
        ];
        let result = parse_tokens(&tokens).unwrap();
        assert_eq!(result, ())
    }

    #[test]
    fn check_parse_tokens_false() {
        let tokens = [
            Token::LeftBrace,
            Token::String("\"key\"".to_string()),
            Token::Colon,
            Token::False,
            Token::RightBrace,
        ];
        let result = parse_tokens(&tokens).unwrap();
        assert_eq!(result, ())
    }

    #[test]
    fn check_parse_tokens_empty_object_as_value() {
        let tokens = [
            Token::LeftBrace,
            Token::String("\"key\"".to_string()),
            Token::Colon,
            Token::LeftBrace,
            Token::RightBrace,
            Token::RightBrace,
        ];
        let result = parse_tokens(&tokens).unwrap();
        assert_eq!(result, ())
    }

    #[test]
    fn check_parse_tokens_inner_array() {
        let tokens = [
            Token::LeftBrace,
            Token::String("\"key\"".to_string()),
            Token::Colon,
            Token::LeftBracket,
            Token::String("\"list value\"".to_string()),
            Token::RightBracket,
            Token::RightBrace,
        ];
        let result = parse_tokens(&tokens).unwrap();
        assert_eq!(result, ())
    }

    #[test]
    fn check_parse_tokens_empty_array() {
        let tokens = [
            Token::LeftBracket,
            Token::RightBracket,
        ];
        let result = parse_tokens(&tokens).unwrap();
        assert_eq!(result, ())
    }

    #[test]
    fn check_parse_tokens_array() {
        let tokens = [
            Token::LeftBracket,
            Token::String("\"value\"".to_string()),
            Token::Comma,
            Token::String("\"value 2\"".to_string()),
            Token::RightBracket,
        ];
        let result = parse_tokens(&tokens).unwrap();
        assert_eq!(result, ())
    }

    #[test]
    fn check_parse_tokens_array_trailing_comma() {
        let tokens = [
            Token::LeftBracket,
            Token::String("\"value\"".to_string()),
            Token::Comma,
            Token::String("\"value 2\"".to_string()),
            Token::Comma,
            Token::RightBracket,
        ];
        let result = parse_tokens(&tokens).unwrap_err();
        assert_eq!(result, ParseError)
    }

    fn build_cmd_assert(file: &str) -> Result<assert_cmd::assert::Assert, Box<dyn std::error::Error>> {
        let mut cmd = std::process::Command::cargo_bin("cc2jsonparser")?;
        Ok(cmd.arg(PathBuf::from(format!("testinputs/{}", file))).assert())
    }

    #[test]
    fn run_cmd_step1_valid() {
        if let Ok(assert) = build_cmd_assert("step1/valid.json") {
            assert.success().code(0);
        }
    }

    #[test]
    fn run_cmd_step1_invalid() {
        if let Ok(assert) = build_cmd_assert("step1/invalid.json") {
            assert.failure().code(1);
        }
    }

    #[test]
    fn run_cmd_step2_valid() {
        if let Ok(assert) = build_cmd_assert("step2/valid.json") {
            assert.success().code(0);
        }
    }

    #[test]
    fn run_cmd_step2_valid2() {
        if let Ok(assert) = build_cmd_assert("step2/valid2.json") {
            assert.success().code(0);
        }
    }

    #[test]
    fn run_cmd_step2_invalid() {
        if let Ok(assert) = build_cmd_assert("step2/invalid.json") {
            assert.failure().code(1);
        }
    }

    #[test]
    fn run_cmd_step2_invalid2() {
        if let Ok(assert) = build_cmd_assert("step2/invalid2.json") {
            assert.failure().code(1);
        }
    }

    #[test]
    fn run_cmd_step3_valid() {
        if let Ok(assert) = build_cmd_assert("step3/valid.json") {
            assert.success().code(0);
        }
    }

    #[test]
    fn run_cmd_step3_invalid() {
        if let Ok(assert) = build_cmd_assert("step3/invalid.json") {
            assert.failure().code(1);
        }
    }

    #[test]
    fn run_cmd_step4_valid() {
        if let Ok(assert) = build_cmd_assert("step4/valid.json") {
            assert.success().code(0);
        }
    }

    #[test]
    fn run_cmd_step4_valid2() {
        if let Ok(assert) = build_cmd_assert("step4/valid2.json") {
            assert.success().code(0);
        }
    }

    #[test]
    fn run_cmd_step4_invalid() {
        if let Ok(assert) = build_cmd_assert("step4/invalid.json") {
            assert.failure().code(1);
        }
    }
}
