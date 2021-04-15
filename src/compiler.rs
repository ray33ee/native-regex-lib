
use std::collections::HashMap;
use std::ops::RangeInclusive;
use regex_syntax::Parser;
use regex_syntax::hir::*;

pub trait Compiler {

    fn no_match_break(in_inner_loop: bool) -> & 'static str;

    fn advance() -> & 'static str;

    fn bounds_check() -> & 'static str;

    fn empty() -> String;

    fn literal_to_snippet(ch: u32, in_inner_loop: bool) -> String;

    fn range_to_snippet(range: RangeInclusive<u32>, is_first_and_only: bool) -> String;

    fn class_to_snippet(ranges: Vec<RangeInclusive<u32>>, in_inner_loop: bool) -> String;

    fn non_capturing_to_snippet(snippet: String) -> String;

    fn capturing_to_snippet(ind: u32, snippet: String) -> String;

    fn map_to_snippet(map: HashMap<String, usize>) -> String;

    fn bounded_to_snippet(inner_code: String, in_inner_loop: bool, n: usize, m: usize) -> String;

    fn unbounded_to_snippet(inner_code: String, in_inner_loop: bool, n: usize) -> String;

    fn base_code(inner: String, capture_count: usize, struct_name: & str, regex: & str, map: HashMap<String, usize>) -> String ;

    fn name_identifier_validator(identifier: & str) -> Result<(), String>;

    fn start_text_snippet() -> String;

    fn end_text_snippet() -> String;

    fn start_line_snippet() -> String;

    fn end_line_snippet() -> String;

    fn wordboundary_unicode(in_inner_loop: bool) -> String;

    fn wordboundary_ascii(in_inner_loop: bool) -> String;

    fn negate_wordboundary_unicode(in_inner_loop: bool) -> String;

    fn negate_wordboundary_ascii(in_inner_loop: bool) -> String;


    fn translate_hir(hir: & Hir, capture_names: & mut HashMap<String, usize>, in_inner_loop: bool) -> Result<(String, Option<usize>), String> {

        let mut snippet = String::new();

        let mut max: Option<usize> = None;

        match hir.kind() {
            HirKind::Empty => {
                snippet = Self::empty();
            },
            HirKind::Literal(literal) => match literal {
                Literal::Byte(byte) => {
                    snippet = format!("{}{}{}", Self::bounds_check(), Self::literal_to_snippet(*byte as u32, in_inner_loop), Self::advance());
                },
                Literal::Unicode(ch) => {
                    snippet = format!("{}{}{}", Self::bounds_check(), Self::literal_to_snippet(*ch as u32, in_inner_loop), Self::advance());
                }
            },
            HirKind::Class(class) => match class {
                Class::Unicode(unicode) => {
                    snippet = format!("{}{}{}",
                                      Self::bounds_check(),
                                      Self::class_to_snippet(unicode.iter().map(|unicode_range| RangeInclusive::new(unicode_range.start() as u32, unicode_range.end() as u32)).collect::<Vec<_>>(), in_inner_loop),
                                      Self::advance());
                },
                Class::Bytes(bytes) => {
                    snippet = format!("{}{}{}",
                                      Self::bounds_check(),
                                      Self::class_to_snippet(bytes.iter().map(|unicode_range| RangeInclusive::new(unicode_range.start() as u32, unicode_range.end() as u32)).collect::<Vec<_>>(), in_inner_loop),
                                      Self::advance());
                }
            },
            HirKind::Anchor(anchor) => match anchor {
                Anchor::EndLine => {
                    snippet = Self::end_line_snippet();
                },
                Anchor::EndText => {
                    snippet = Self::end_text_snippet();
                },
                Anchor::StartLine => {
                    snippet = Self::start_line_snippet();
                },
                Anchor::StartText => {
                    snippet = Self::start_text_snippet();
                }
            },
            HirKind::WordBoundary(boundary) => match boundary {
                WordBoundary::Unicode => {
                    snippet =  Self::wordboundary_unicode(in_inner_loop);
                }
                WordBoundary::Ascii => {


                    snippet =  Self::wordboundary_ascii(in_inner_loop);
                }
                WordBoundary::AsciiNegate => {

                    snippet =  Self::negate_wordboundary_ascii(in_inner_loop);

                }
                WordBoundary::UnicodeNegate => {


                    snippet =  Self::negate_wordboundary_unicode(in_inner_loop);
                }
            },
            HirKind::Repetition(repeater) => {
                if !repeater.greedy {
                    return Err(String::from("Non-greedy repetition is not supported since backtracking is not supported. Please see NativeRegex readme for more details."));
                }

                let (subset, m) = Self::translate_hir(repeater.hir.as_ref(), capture_names, true)?;

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
                        Self::bounded_to_snippet(subset, in_inner_loop, 0, 1)
                    },
                    RepetitionKind::OneOrMore => {
                        Self::unbounded_to_snippet(subset, in_inner_loop, 1)
                    },
                    RepetitionKind::ZeroOrMore => {
                        Self::unbounded_to_snippet(subset, in_inner_loop, 0)
                    },
                    RepetitionKind::Range(range) => match range {
                        RepetitionRange::AtLeast(n) => {
                            Self::unbounded_to_snippet(subset, in_inner_loop, n as usize)
                        },
                        RepetitionRange::Bounded(n, m) => {
                            Self::bounded_to_snippet(subset, in_inner_loop, n as usize, m as usize)
                        },
                        RepetitionRange::Exactly(n ) => {
                            Self::bounded_to_snippet(subset, in_inner_loop, n as usize, n as usize)
                        }
                    },

                }

            },
            HirKind::Group(group) => {

                let (subset, m) = Self::translate_hir(group.hir.as_ref(), capture_names, in_inner_loop)?;

                snippet = match group.kind.clone() {
                    GroupKind::NonCapturing => {
                        Self::non_capturing_to_snippet(subset)
                    },
                    GroupKind::CaptureIndex(index) => {
                        max = Some(index as usize);
                        Self::capturing_to_snippet(index, subset)
                    },
                    GroupKind::CaptureName { name, index } => {
                        max = Some(index as usize);
                        capture_names.insert(name, index as usize);

                        Self::capturing_to_snippet(index, subset)
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
                    let (subset, m) = Self::translate_hir(hir, capture_names, in_inner_loop)?;


                    if m.is_some() {
                        if max.is_some() {
                            if m.unwrap() > max.unwrap() {
                                max = Some(m.unwrap())
                            }
                        } else {
                            max = m;
                        }
                    }

                    snippet.push_str(&subset);
                }
            },
            HirKind::Alternation(_) => {
                return Err(String::from("Alternation is not supported. Please see NativeRegex readme for more details."));
            }
        }

        Ok((snippet, max))
    }

    fn translate( regex: & str, identifier_name: & str) -> Result<String, String> {

        Self::name_identifier_validator(identifier_name)?;

        match Parser::new().parse(regex) {
            Ok(hir) => {
                let mut map = HashMap::new();
                let (inner, max) = Self::translate_hir(&hir, & mut map, false)?;

                Ok(Self::base_code(inner, if max.is_some() { max.unwrap() } else { 0 } + 1, identifier_name, regex, map))
            }
            Err(e) => {
                Err(e.to_string())
            }
        }


    }

}