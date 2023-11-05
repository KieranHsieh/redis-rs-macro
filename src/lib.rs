use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use std::mem;
use syn::{parse_macro_input, parse_quote, Expr, LitStr};

/// State used by the internal redis command lexer
enum State {
    Word,               // Inside unquoted word
    DoubleQuote,        // Inside double quote
    SplitMarker,        // Whitespace (Not \r)
    EscapedDoubleQuote, // Inside double quote after backslash
    Braced,             // Inside brace
}

/// A single redis command argument.
#[derive(Default, PartialEq, Debug)]
struct CmdArg {
    data: String,
    is_quoted: bool,
    is_braced: bool,
}

/// Split an input string by whitespace, except if enclosed by "double quotes" or
/// {curly braces}
fn split_input(input: &str) -> Vec<CmdArg> {
    let mut chars = input.chars();
    let mut output: Vec<CmdArg> = vec![];
    let mut current_word = CmdArg::default();
    let mut state = State::SplitMarker;
    loop {
        let cur = chars.next();
        state = match state {
            State::Word => match cur {
                None => {
                    output.push(CmdArg {
                        data: mem::replace(&mut current_word.data, String::new()),
                        is_quoted: current_word.is_quoted,
                        is_braced: current_word.is_braced,
                    });
                    current_word.is_quoted = false;
                    current_word.is_braced = false;
                    break;
                }
                Some('\t') | Some(' ') | Some('\n') => {
                    output.push(CmdArg {
                        data: mem::replace(&mut current_word.data, String::new()),
                        is_quoted: current_word.is_quoted,
                        is_braced: current_word.is_braced,
                    });
                    current_word.is_quoted = false;
                    current_word.is_braced = false;
                    State::SplitMarker
                }
                Some(c) => {
                    current_word.data.push(c);
                    State::Word
                }
            },
            State::SplitMarker => match cur {
                Some('\t') | Some(' ') | Some('\n') => State::SplitMarker,
                Some('\"') => {
                    current_word.is_quoted = true;
                    State::DoubleQuote
                }
                Some('{') => {
                    current_word.is_braced = true;
                    State::Braced
                }
                Some(c) => {
                    current_word.data.push(c);
                    State::Word
                }
                _ => break,
            },
            State::DoubleQuote => match cur {
                // Shouldn't ever happen. Macro syntax is invalid if there is an unclosed double quote
                None => panic!("incomplete quoted value"),
                Some('"') => State::Word,
                Some('\\') => State::EscapedDoubleQuote,
                Some(c) => {
                    current_word.data.push(c);
                    State::DoubleQuote
                }
            },
            State::EscapedDoubleQuote => match cur {
                // Shouldn't ever happen. Macro syntax is invalid if there is nothing after the backslash
                None => panic!("invalid escape sequence"),
                Some(cur) => {
                    current_word.data.push('\\');
                    current_word.data.push(cur);
                    State::DoubleQuote
                }
            },
            State::Braced => match cur {
                // Shouldn't ever happen. Macro syntax is invalid if there is an unclosed brace
                None => panic!("unclosed brace"),
                Some('}') => State::Word,
                Some(cur) => {
                    current_word.data.push(cur);
                    State::Braced
                }
            },
        };
    }
    output
}

/// Generate a redis::cmd object using syntax as if from redis-cli
///
/// # Examples
/// ## Writing a command
/// The most basic usage of the macro is as seen below.
/// ```rust
/// use redis_rs_macro::redis;
/// redis!(SET my_key my_value 1);
/// ```
/// In the above example, my_key, my_value, and 1, are all passed into the .arg function of
/// redis::cmd as if they were literal strings.
/// ## Expansion
/// ```rust
/// redis::cmd("SET").arg("my_key").arg("my_value").arg("1");
/// ```
/// ## Quoting
/// If any of the above arguments contain whitespace, but should be treated as a single argument,
/// use whitespace to capture the entire sequence.
/// ```rust
/// use redis_rs_macro::redis;
/// redis!(SET "my key" my_value 1);
/// ```
/// ## Expansion
/// ```rust
/// redis::cmd("SET").arg("my key").arg("my_value").arg("1");
/// ```
/// ## Substitution
/// You can also substitue Rust expressions into .arg or cmd constructor if
/// you have dynamic data. This is done by enclosing the expression in curly braces.
/// ```rust
/// use redis_rs_macro::redis;
/// let x = 1;
/// redis!(SET my_key my_value {x});
/// ```
/// ## Expansion
/// ```rust
/// let x = 1;
/// redis::cmd("SET").arg("my_key").arg("my_value").arg(x);
/// ```
#[proc_macro]
pub fn redis(tokens: TokenStream) -> TokenStream {
    let token_str = tokens.to_string();
    let split_input = split_input(token_str.as_str());
    if split_input.is_empty() {
        return TokenStream::new();
    }

    let mut args: Vec<Expr> = vec![];
    for arg in split_input.into_iter() {
        if arg.is_quoted {
            let data: proc_macro::TokenStream = arg.data.to_token_stream().into();
            let litstr = parse_macro_input!(data as LitStr);
            args.push(parse_quote!(#litstr));
        } else {
            if arg.is_braced {
                let expr: Expr = match syn::parse_str::<Expr>(&arg.data) {
                    Ok(expr) => expr,
                    Err(err) => {
                        return TokenStream::from(err.to_compile_error());
                    }
                };
                args.push(expr);
            } else {
                let strm = arg.data.to_token_stream();
                args.push(parse_quote!(#strm));
            }
        }
    }
    let cmd = &args[0];
    let additional_args = &args[1..];
    quote! {
        redis::cmd(#cmd)#(.arg(#additional_args))*
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split_(cases: &[(&str, &[CmdArg])]) {
        for &(input, expected) in cases {
            let output: Vec<CmdArg> = split_input(input);
            assert!(
                expected == output.as_slice(),
                "Input: {:?}\nExpected: {:?}\nBut found: {:?}",
                input,
                expected,
                output
            );
        }
    }

    #[test]
    fn split_empty() {
        split_(&[("", &[])]);
    }

    #[test]
    fn split_leading_ws() {
        split_(&[
            (
                " abcd",
                &[CmdArg {
                    data: "abcd".into(),
                    is_quoted: false,
                    is_braced: false,
                }],
            ),
            (
                "\tabcd",
                &[CmdArg {
                    data: "abcd".into(),
                    is_quoted: false,
                    is_braced: false,
                }],
            ),
            (
                "\nabcd",
                &[CmdArg {
                    data: "abcd".into(),
                    is_quoted: false,
                    is_braced: false,
                }],
            ),
            (
                " \n\tabcd",
                &[CmdArg {
                    data: "abcd".into(),
                    is_quoted: false,
                    is_braced: false,
                }],
            ),
        ]);
    }

    #[test]
    fn split_normal() {
        split_(&[(
            "abcd 123",
            &[
                CmdArg{
                    data: "abcd".into(),
                    is_quoted: false,
                    is_braced: false
                },
                CmdArg{
                    data: "123".into(),
                    is_quoted: false,
                    is_braced: false
                }
            ]
        )])
    }

    #[test]
    fn split_dquotes() {
        split_(&[(
            "\"abcd 123\" abcd",
            &[
                CmdArg {
                    data: "abcd 123".into(),
                    is_quoted: true,
                    is_braced: false,
                },
                CmdArg{
                    data: "abcd".into(),
                    is_quoted: false,
                    is_braced: false
                }
            ],
        )]);
    }

    #[test]
    fn split_brackets() {
        split_(&[(
            "{abcd 123} abcd",
            &[
                CmdArg {
                    data: "abcd 123".into(),
                    is_quoted: false,
                    is_braced: true,
                },
                CmdArg{
                    data: "abcd".into(),
                    is_quoted: false,
                    is_braced: false
                },
            ],
        )]);
    }
}
