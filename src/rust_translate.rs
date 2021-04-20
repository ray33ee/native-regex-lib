

use crate::ehir::{Ehir, Token, Decision, NoMatch, Modifier, AnchorLocation, AnchorType, WordBoundaryType, Range};
use std::collections::HashMap;
use std::borrow::Borrow;


fn range_to_snippet(range: & Range, code: & mut String) {

    match range {
        Range::Single(n) => {
            code.push_str("(character.current().unwrap() as u32) == ");
            code.push_str(format!("{}", n).as_str());
        }
        Range::Multiple(n, m) => {

            code.push_str("((character.current().unwrap() as u32) >= ");
            code.push_str(format!("{}", n).as_str());
            code.push_str(" && (character.current().unwrap() as u32) <= ");
            code.push_str(format!("{}", m).as_str());

            code.push_str(")");
        }
    }
}

//Take a single token and convert it into a snippet of Rust code
fn translate_token(token: & Token, code: & mut String) -> Result<(), String> {
    match token {
        Token::If(modifier, decision, stop_or_break) => {
            code.push_str("if ");

            //Decision
            match decision {
                Decision::CharacterSet(range_list) => {
                    let mut range_list = range_list.into_iter();

                    range_to_snippet(range_list.next().unwrap(), code);

                    for range in range_list {
                        code.push_str(" || ");
                        range_to_snippet(range, code);
                    }
                }
                Decision::Literal(character) => {
                    code.push_str("(character.current().unwrap() as u32) == ");
                    code.push_str(format!("{}", character).as_str());

                }
                Decision::LiteralString(_) => {
                    return Err(String::from("Literal strings not supported yet."));
                }
                Decision::CountEquals(n) => {
                    code.push_str("match_count == ");
                    code.push_str(format!("{}", n).as_str());
                }
                Decision::CountLessThan(n) => {
                    code.push_str("match_count < ");
                    code.push_str(format!("{}", n).as_str());
                }
                Decision::Anchor(anchor_type, anchor_location) => {
                    match anchor_location {
                        AnchorLocation::Start => {
                            match anchor_type {
                                AnchorType::Regular => {
                                    code.push_str("character.previous() == native_regex_lib::native_regex::character::Previous::Start");
                                }
                                AnchorType::Newline => {
                                    code.push_str("character.previous() == native_regex_lib::native_regex::character::Previous::Character('\\n') || character.previous() == native_regex_lib::native_regex::character::Previous::Start");
                                }
                            }
                        }
                        AnchorLocation::End => {
                            match anchor_type {
                                AnchorType::Regular => {
                                    code.push_str("character.current().is_none()");
                                }
                                AnchorType::Newline => {
                                    code.push_str("{ if character.current().is_some() { if character.current().unwrap() != '\\n' { false } else { true } } else { true } }");
                                }
                            }
                        }
                    }
                }
                Decision::WordBoundary(boundary_type) => {
                    match boundary_type {
                        WordBoundaryType::Byte => {
                            code.push_str("{ if character.previous() != native_regex_lib::native_regex::character::Previous::Start && character.current().is_some() {
    if (Self::is_word_byte(character.previous().unwrap()) || !Self::is_word_byte(character.current().unwrap())) &&
        (!Self::is_word_byte(character.previous().unwrap()) || Self::is_word_byte(character.current().unwrap())) {
        false
    } else {
        true
    }
} else {
    if character.previous() == native_regex_lib::native_regex::character::Previous::Start && !Self::is_word_byte(character.current().unwrap()) || character.current().is_none() && !Self::is_word_byte(character.previous().unwrap()) {
        false
    } else {
        true
    }
} }");
                        }
                        WordBoundaryType::Character => {
                            code.push_str("{ if character.previous() != native_regex_lib::native_regex::character::Previous::Start && character.current().is_some() {
    if (Self::is_word_character(character.previous().unwrap()) || !Self::is_word_character(character.current().unwrap())) &&
        (!Self::is_word_character(character.previous().unwrap()) || Self::is_word_character(character.current().unwrap())) {
        false
    } else {
        true
    }
} else {
    if character.previous() == native_regex_lib::native_regex::character::Previous::Start && !Self::is_word_character(character.current().unwrap()) || character.current().is_none() && !Self::is_word_character(character.previous().unwrap()) {
        false
    } else {
        true
    }
} }");
                        }
                    }
                }
                Decision::Middle => {
                    code.push_str("character.current().is_some()");
                }
            }

            //Cheaty invert the logic by using else
            if *modifier == Modifier::Not {
                code.push_str(" {  } else ");
            }

            //Body
            code.push_str(" { ");
            code.push_str(match stop_or_break {
                NoMatch::Stop => {
                    "return None;"
                }
                NoMatch::Break => {
                    "break;"
                }
            });
            code.push_str(" }\n\n");
        }
        Token::While(decision, block) => {

            code.push_str("while ");

            match decision {
                Decision::Middle => {
                    code.push_str("character.current().is_some() ")
                }
                _ => { unreachable!() }
            }

            translate_token(block.as_ref(), code)?;
        }
        Token::StartCount => {
            code.push_str("let mut match_count = 0;\n\n");
        }
        Token::IncrementCount => {
            code.push_str("match_count += 1;\n\n");
        }
        Token::Advance => {
            code.push_str("character = chars.advance();\n\n");
        }
        Token::Capture(index, token_list) => {
            let index = format!("{}", index);

            //Start of capture
            code.push_str("let capture_");
            code.push_str(index.as_str());
            code.push_str("_start = character.index();\n\n");

            //Capture body
            for element in token_list {
                translate_token(element, code)?;
            }

            //End of capture
            code.push_str("captures.insert(");
            code.push_str(index.as_str());
            code.push_str(", (capture_");
            code.push_str(index.as_str());
            code.push_str("_start, character.index()));\n\n");
        }
        Token::Block(token_list) => {
            code.push_str("{\n\n");

            for element in token_list {
                translate_token(element, code)?;
            }

            code.push_str("}\n\n")
        }
        Token::Empty => {}
    }
    Ok(())
}

//Not a token. Just return a bunch of (index, & str) pairs
fn map_to_snippet(map: & HashMap<String, u32>, code: & mut String) {
    for (name, index) in map {
        code.push_str(format!("named_groups.insert(\"{}\", {});\n", name, index).as_str());
    }
}

fn translate_ehir(ehir: & Ehir, struct_name: & str) -> Result<String, String> {
    let mut code = String::new();

    code.push_str("pub struct ");
    code.push_str(struct_name);
    code.push_str(" {
    named_groups: std::collections::HashMap<& 'static str, usize>
}

impl ");
    code.push_str(struct_name);
    code.push_str(" {
    pub fn new() -> Self {
        let ");
    if !ehir._capture_names.is_empty() {
        code.push_str("mut ")
    }
    code.push_str("named_groups = std::collections::HashMap::new();

        ");
    map_to_snippet( & ehir._capture_names, &mut  code);
    code.push_str("

        ");
    code.push_str(struct_name);
    code.push_str(" {
            named_groups
        }
    }
}

impl Into<native_regex_lib::native_regex::Engine> for ");
    code.push_str(struct_name);
    code.push_str(" {

    fn into(self) -> native_regex_lib::native_regex::Engine {
        self.engine()
    }

}

impl native_regex_lib::native_regex::NativeRegex for ");
    code.push_str(struct_name);
    code.push_str(" {

    // Function to match regex '");
    code.push_str(ehir._regex);
    code.push_str("'
    #[allow(unused_parens, unused_comparisons)]
    #[inline(always)]
    fn step(mut chars: native_regex_lib::native_regex::character::Advancer, captures: & mut native_regex_lib::vectormap::VectorMap<(usize, usize)>) -> Option<()> {

        //Advance to first character & bounds check
        let mut character = chars.advance();

        ");

    for element in ehir._tokens.iter() {
        translate_token(element, & mut code)?;
    }

    code.push_str("

        Some(())
    }

    fn capture_names(&self) -> &std::collections::HashMap<& 'static str, usize> {
        &self.named_groups
    }

    fn capture_count(&self) -> usize { ");
    code.push_str(format!("{}", ehir._capture_count).as_str());
    code.push_str(" }

}");


    Ok(code)
}

pub fn translate(regex: & str, identifier_name: & str) -> Result<String, String> {
    translate_ehir(Ehir::translate(regex)?.borrow(), identifier_name)
}