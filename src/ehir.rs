
use std::collections::HashMap;
use regex_syntax::hir::*;
use regex_syntax::Parser;

#[derive(Debug)]
pub enum NoMatch {
    Stop,
    Break,
}

#[derive(Debug)]
pub enum Range {
    Single(u32),
    Multiple(u32, u32),
}

#[derive(Debug)]
pub enum AnchorLocation {
    Start,
    End
}

#[derive(Debug)]
pub enum AnchorType {
    Regular,
    Newline
}

#[derive(Debug)]
pub enum WordBoundaryType {
    Byte,
    Character
}

#[derive(Debug)]
pub enum Decision {
    CharacterSet(Vec<Range>), //Should be true if the character is NOT within the character set
    Literal(u32), //Determine if the character is a particular literal
    LiteralString(String),
    CountEquals(u32), //
    CountLessThan(u32),
    Anchor(AnchorType, AnchorLocation),
    WordBoundary(WordBoundaryType),
    Middle //Determine if we are NOT at the end of the text
}

#[derive(Debug)]
pub enum CaptureType {
    Beginning,
    End
}

#[derive(Debug, PartialEq)]
pub enum Modifier {
    Is, // A
    Not // !A
}

#[derive(Debug)]
pub enum Token {
    If(Modifier, Decision, NoMatch),
    While(Decision, Box<Token>), //Loop to check repetition
    StartCount, //Set the counter to zero. Used in repetition to check bounds
    IncrementCount, //Increment the counter every time a repetition matches
    Advance, //Advance to the next character
    Capture(u32, Vec<Token>), //TOken representing a capturing group
    Block(Vec<Token>),
    Empty,
}

#[derive(Debug)]
pub struct Ehir<'r> {
    pub _regex: & 'r str,
    pub _tokens: Vec<Token>, //List of tokens representing the EHIR
    pub _capture_names: HashMap<String, u32>, //A hashmap of all named capture groups and their corresponding indices
    pub _capture_count: u32, //Total number of capture groups, (including the entire match)
}

impl<'r> Ehir<'r> {

    fn bounds_check() -> Token {
        Token::If(Modifier::Not, Decision::Middle, NoMatch::Stop)
    }

    fn stop_break(is_inner_loop: bool) -> NoMatch {
        if is_inner_loop { NoMatch::Break } else { NoMatch::Stop }
    }

    fn bounded_to_snippet(mut inner_code: Vec<Token>, in_inner_loop: bool, n: u32, m: u32) -> Vec<Token> {

        inner_code.push(Token::IncrementCount);
        inner_code.push(Token::If(Modifier::Is, Decision::CountEquals(m), NoMatch::Break));

        vec![Token::Block(vec![
            Token::StartCount,
            Token::While(Decision::Middle, Box::new(Token::Block(inner_code))),
            Token::If(Modifier::Is, Decision::CountLessThan(n), Ehir::stop_break(in_inner_loop))
        ])]
    }

    fn unbounded_to_snippet(mut inner_code: Vec<Token>, in_inner_loop: bool, n: u32) -> Vec<Token> {

        inner_code.push(Token::IncrementCount);

        vec![Token::Block(vec![
            Token::StartCount,
            Token::While(Decision::Middle, Box::new(Token::Block(inner_code))),
            Token::If(Modifier::Is, Decision::CountLessThan(n), Ehir::stop_break(in_inner_loop))
        ])]
    }

    fn capturing_to_snippet(ind: u32, snippet: Vec<Token>) -> Vec<Token> {

        vec![Token::Block(vec![Token::Capture(ind, snippet)])]
    }

    fn non_capturing_to_snippet(snippet: Vec<Token>) -> Vec<Token> {
        vec![Token::Block(snippet)]
    }

    fn translate_hir(hir: & Hir, capture_names: & mut HashMap<String, u32>, in_inner_loop: bool) -> Result<(Vec<Token>, Option<u32>), String> {

        let mut snippet = vec![];

        let mut max: Option<u32> = None;

        match hir.kind() {
            HirKind::Empty => {
                snippet = vec![Token::Empty];
            },
            HirKind::Literal(literal) => match literal {
                Literal::Byte(byte) => {
                    snippet = vec![
                        Ehir::bounds_check(),
                        Token::If(Modifier::Not, Decision::Literal(*byte as u32), Ehir::stop_break(in_inner_loop)),
                        Token::Advance];
                },
                Literal::Unicode(ch) => {
                    snippet = vec![
                        Ehir::bounds_check(),
                        Token::If(Modifier::Not, Decision::Literal(*ch as u32), Ehir::stop_break(in_inner_loop)),
                        Token::Advance];
                }
            },
            HirKind::Class(class) => match class {
                Class::Unicode(unicode) => {

                    let range_set: Vec<_> = unicode.iter().map(|unicode_range| {

                        if unicode_range.start() == unicode_range.end() {
                            Range::Single(unicode_range.start() as u32)
                        } else {
                            Range::Multiple(unicode_range.start() as u32, unicode_range.end() as u32)
                        }
                    }).collect::<Vec<_>>();

                    snippet = vec![
                        Ehir::bounds_check(),
                        Token::If(Modifier::Not, Decision::CharacterSet(range_set), Ehir::stop_break(in_inner_loop)),
                        Token::Advance];
                },
                Class::Bytes(bytes) => {
                    let range_set: Vec<_> = bytes.iter().map(|unicode_range| {

                        if unicode_range.start() == unicode_range.end() {
                            Range::Single(unicode_range.start() as u32)
                        } else {
                            Range::Multiple(unicode_range.start() as u32, unicode_range.end() as u32)
                        }
                    }).collect::<Vec<_>>();

                    snippet = vec![
                        Ehir::bounds_check(),
                        Token::If(Modifier::Not, Decision::CharacterSet(range_set), Ehir::stop_break(in_inner_loop)),
                        Token::Advance];
                }
            },
            HirKind::Anchor(anchor) => match anchor {
                Anchor::EndLine => {
                    snippet = vec![Token::If(Modifier::Not, Decision::Anchor(AnchorType::Newline, AnchorLocation::End), Ehir::stop_break(in_inner_loop))];
                },
                Anchor::EndText => {
                    snippet = vec![Token::If(Modifier::Not, Decision::Anchor(AnchorType::Regular, AnchorLocation::End), Ehir::stop_break(in_inner_loop))];
                },
                Anchor::StartLine => {
                    snippet = vec![Token::If(Modifier::Not, Decision::Anchor(AnchorType::Newline, AnchorLocation::Start), Ehir::stop_break(in_inner_loop))];
                },
                Anchor::StartText => {
                    snippet = vec![Token::If(Modifier::Not, Decision::Anchor(AnchorType::Regular, AnchorLocation::Start), Ehir::stop_break(in_inner_loop))];
                }
            },
            HirKind::WordBoundary(boundary) => match boundary {
                WordBoundary::Unicode => {
                    //snippet =  Self::wordboundary_unicode(in_inner_loop);
                    snippet = vec![Token::If(Modifier::Not, Decision::WordBoundary(WordBoundaryType::Character), Ehir::stop_break(in_inner_loop))];
                }
                WordBoundary::Ascii => {
                    snippet = vec![Token::If(Modifier::Not, Decision::WordBoundary(WordBoundaryType::Byte), Ehir::stop_break(in_inner_loop))];
                }
                WordBoundary::AsciiNegate => {
                    snippet =vec![Token::If(Modifier::Is, Decision::WordBoundary(WordBoundaryType::Byte), Ehir::stop_break(in_inner_loop))];
                }
                WordBoundary::UnicodeNegate => {
                    snippet = vec![Token::If(Modifier::Is, Decision::WordBoundary(WordBoundaryType::Character), Ehir::stop_break(in_inner_loop))];
                }
            },
            HirKind::Repetition(repeater) => {
                if !repeater.greedy {
                    return Err(String::from("Non-greedy repetition is not supported since backtracking is not supported. Please see NativeRegex readme for more details."));
                }

                let (subset, m) = Ehir::translate_hir(repeater.hir.as_ref(), capture_names, true)?;

                if m.is_some() {
                    if max.is_some() {
                        if m.unwrap() > max.unwrap() {
                            max = Some(m.unwrap())
                        }
                    } else {
                        max = m;
                    }
                }

                snippet = match repeater.kind.clone() {
                    RepetitionKind::ZeroOrOne => {
                        Ehir::bounded_to_snippet(subset, in_inner_loop, 0, 1)
                    },
                    RepetitionKind::OneOrMore => {
                        Ehir::unbounded_to_snippet(subset, in_inner_loop, 1)
                    },
                    RepetitionKind::ZeroOrMore => {
                        Ehir::unbounded_to_snippet(subset, in_inner_loop, 0)
                    },
                    RepetitionKind::Range(range) => match range {
                        RepetitionRange::AtLeast(n) => {
                            Ehir::unbounded_to_snippet(subset, in_inner_loop, n)
                        },
                        RepetitionRange::Bounded(n, m) => {
                            Ehir::bounded_to_snippet(subset, in_inner_loop, n, m)
                        },
                        RepetitionRange::Exactly(n ) => {
                            Ehir::bounded_to_snippet(subset, in_inner_loop, n, n)
                        }
                    },

                }

            },
            HirKind::Group(group) => {

                let (subset, m) = Ehir::translate_hir(group.hir.as_ref(), capture_names, in_inner_loop)?;

                snippet = match group.kind.clone() {
                    GroupKind::NonCapturing => {
                        Ehir::non_capturing_to_snippet(subset)
                    },
                    GroupKind::CaptureIndex(index) => {
                        max = Some(index);
                        Ehir::capturing_to_snippet(index, subset)
                    },
                    GroupKind::CaptureName { name, index } => {
                        max = Some(index);
                        capture_names.insert(name, index);

                        Ehir::capturing_to_snippet(index, subset)
                    }
                };

                if m.is_some() {
                    if max.is_some() {
                        if m.unwrap() > max.unwrap() {
                            max = Some(m.unwrap())
                        }
                    } else {
                        max = m;
                    }
                }



            },
            HirKind::Concat(hirs) => {
                for hir in hirs {
                    let (mut subset, m) = Ehir::translate_hir(hir, capture_names, in_inner_loop)?;


                    if m.is_some() {
                        if max.is_some() {
                            if m.unwrap() > max.unwrap() {
                                max = Some(m.unwrap())
                            }
                        } else {
                            max = m;
                        }
                    }

                    snippet.append(& mut subset);
                }
            },
            HirKind::Alternation(_) => {
                return Err(String::from("Alternation is not supported. Please see NativeRegex readme for more details."));
            }
        }

        Ok((snippet, max))
    }

    pub fn translate(regex: & str) -> Result<Ehir, String> {

        match Parser::new().parse(regex) {
            Ok(hir) => {
                let mut map = HashMap::new();
                let mut ehir_code = Vec::new();
                let (inner, max) = Ehir::translate_hir(&hir, & mut map, false)?;

                ehir_code.push(Ehir::bounds_check());

                ehir_code.append(& mut Ehir::capturing_to_snippet(0, inner));

                Ok(Ehir {
                    _regex: regex,
                    _tokens: ehir_code,
                    _capture_names: map,
                    _capture_count: if max.is_some() { max.unwrap() } else { 0 } + 1
                })
            }
            Err(e) => {
                Err(e.to_string())
            }
        }


    }
}
