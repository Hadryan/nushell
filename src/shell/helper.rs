use crate::shell::completer::NuCompleter;
use crate::parser::parse2::PipelineElement;
use crate::parser::parse2::token_tree::TokenNode;
use crate::parser::parse2::tokens::RawToken;
use crate::parser::parse2::span::Spanned;
use crate::parser::nom_input;
use crate::prelude::*;
use ansi_term::Color;
use rustyline::completion::{self, Completer, FilenameCompleter};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::{Hinter, HistoryHinter};
use std::borrow::Cow::{self, Owned};

crate struct Helper {
    completer: NuCompleter,
    hinter: HistoryHinter,
}

impl Helper {
    crate fn new(commands: indexmap::IndexMap<String, Arc<dyn Command>>) -> Helper {
        Helper {
            completer: NuCompleter {
                file_completer: FilenameCompleter::new(),
                commands,
            },
            hinter: HistoryHinter {},
        }
    }
}

impl Completer for Helper {
    type Candidate = completion::Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &rustyline::Context<'_>,
    ) -> Result<(usize, Vec<completion::Pair>), ReadlineError> {
        self.completer.complete(line, pos, ctx)
    }
}

impl Hinter for Helper {
    fn hint(&self, line: &str, pos: usize, ctx: &rustyline::Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for Helper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(&'s self, prompt: &'p str, _: bool) -> Cow<'b, str> {
        Owned("\x1b[32m".to_owned() + &prompt[0..prompt.len() - 2] + "\x1b[m> ")
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        let tokens = crate::parser::pipeline(nom_input(line));

        match tokens {
            Err(_) => Cow::Borrowed(line),
            Ok((_rest, v)) => {
                let mut out = String::new();
                let tokens = match v.as_pipeline() {
                    Err(_) => return Cow::Borrowed(line),
                    Ok(v) => v,
                };

                let mut iter = tokens.into_iter();

                // match iter.next() {
                //     None => return Cow::Owned(out),
                //     Some(v) => out.push_str(v.span().slice(line)),
                // };
                let mut first = true;

                loop {
                    match iter.next() {
                        None => return Cow::Owned(out),
                        Some(token) => {
                            let styled = paint_pipeline_element(&token, line);

                            if !first {
                                out.push_str("|");
                            } else {
                                first = false;
                            }

                            out.push_str(&styled.to_string());
                        }
                    }
                }
            }
        }
    }

    fn highlight_char(&self, _line: &str, _pos: usize) -> bool {
        true
    }
}

fn paint_token_node(token_node: &TokenNode, line: &str) -> String {
    let styled = match token_node {
        TokenNode::Call(..) => Color::Cyan.bold().paint(token_node.span().slice(line)),
        TokenNode::Whitespace(..) => Color::White.normal().paint(token_node.span().slice(line)),
        TokenNode::Flag(..) => Color::Black.bold().paint(token_node.span().slice(line)),
        TokenNode::Identifier(..) => Color::Yellow.bold().paint(token_node.span().slice(line)),
        TokenNode::Path(..) => Color::Green.bold().paint(token_node.span().slice(line)),
        TokenNode::Error(..) => Color::Red.bold().paint(token_node.span().slice(line)),
        TokenNode::Delimited(..) => Color::White.paint(token_node.span().slice(line)),
        TokenNode::Operator(..) => Color::Purple.bold().paint(token_node.span().slice(line)),
        TokenNode::Pipeline(..) => Color::Blue.normal().paint(token_node.span().slice(line)),
        TokenNode::Token(Spanned { item: RawToken::Integer(..), ..}) => Color::Purple.bold().paint(token_node.span().slice(line)),
        TokenNode::Token(Spanned { item: RawToken::Size(..), ..}) => Color::Purple.bold().paint(token_node.span().slice(line)),
        TokenNode::Token(Spanned { item: RawToken::String(..), ..}) => Color::Green.normal().paint(token_node.span().slice(line)),
        TokenNode::Token(Spanned { item: RawToken::Variable(..), ..}) => Color::Yellow.bold().paint(token_node.span().slice(line)),
        TokenNode::Token(Spanned { item: RawToken::Bare, ..}) => Color::Green.normal().paint(token_node.span().slice(line)),
    };

    styled.to_string()
}

fn paint_pipeline_element(pipeline_element: &PipelineElement, line: &str) -> String {
    let mut styled = String::new();

    if let Some(ws) = pipeline_element.pre_ws {
        styled.push_str(&Color::White.normal().paint(ws.slice(line)));
    }
    styled.push_str(&paint_token_node(pipeline_element.call().head(), line));

    if let Some(children) = pipeline_element.call().children() {
        for child in children {
            styled.push_str(&paint_token_node(child, line));
        }
    }
    if let Some(ws) = pipeline_element.post_ws {
        styled.push_str(&Color::White.normal().paint(ws.slice(line)));
    }

    styled.to_string()
}

impl rustyline::Helper for Helper {}
