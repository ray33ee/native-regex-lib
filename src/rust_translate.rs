
use std::collections::HashMap;
use std::ops::RangeInclusive;
use regex_syntax::Parser;
use regex_syntax::hir::*;

fn no_match_break(in_inner_loop: bool) -> & 'static str {
    if in_inner_loop { "break;" } else { "return None;" }
}

fn advance() -> & 'static str {
    "current = chars.next();\n\n"
}

fn bounds_check() -> & 'static str {
    "if current.is_none() { return None; }\n\n"
}

fn empty() -> String {
    String::new()
}

fn literal_to_snippet(ch: u32, in_inner_loop: bool) -> String {
    format!("{}if (current.unwrap().1 as u32) != {} {{ {} }}\n\n {}", bounds_check(), ch, no_match_break(in_inner_loop), advance())
}

fn range_to_snippet(range: RangeInclusive<u32>, is_first_and_only: bool) -> String {
    let (start, end) = range.into_inner();

    if start == end {
        format!("(current.unwrap().1 as u32) != {}", start)
    } else {
        if is_first_and_only {
            format!("(current.unwrap().1 as u32) < {} || (current.unwrap().1 as u32) > {}", start, end)
        } else {
            format!("((current.unwrap().1 as u32) < {} || (current.unwrap().1 as u32) > {})", start, end)
        }

    }
}

fn class_to_snippet<I>(ranges: I, in_inner_loop: bool) -> String
    where I: IntoIterator<Item = RangeInclusive<u32>> {

    let mut ranges = ranges.into_iter();

    let mut str_ranges = range_to_snippet(ranges.next().unwrap(), false);

    for range in ranges {
        str_ranges.push_str(" && ");
        str_ranges.push_str(&range_to_snippet(range, false));
    }

    format!("{}if {} {{\n {} \n}}\n\n{}", bounds_check(), str_ranges, no_match_break(in_inner_loop), advance())
}

fn noncapturing_to_snippet(snippet: String) -> String {
    format!("{{\n\n{}}}\n\n", snippet)
}

fn capturing_to_snippet(ind: u32, snippet: String) -> String {
    let capture_start = format!("let capture_{}_start = current.unwrap().0 + offset;\n\n", ind);
    let capture_end = format!("captures[{}] = Some((capture_{}_start, if current.is_some() {{ current.unwrap().0 + offset }} else {{ length }}));\n\n", ind, ind);

    format!("{{\n\n{}{}{}}}\n\n", capture_start, snippet, capture_end)
}

fn zero_and_one_to_snippet(inner_code: String) -> String {
    format!("{{
    let mut match_count = 0;

    while current.is_some() {{
        {}

        match_count += 1;

        if match_count == 1 {{
            break;
        }}
    }}

}}\n\n", inner_code)
}

fn zero_or_more_to_snippet(inner_code: String) -> String {
    format!("{{
    while current.is_some() {{
        {}
    }}

}}\n\n", inner_code)
}



fn one_or_more_to_snippet(inner_code: String, in_inner_loop: bool) -> String {
    format!("{{
    let mut found = false;

    while current.is_some() {{
        {}

        found = true;
    }}

    if !found {{
        {}
    }}

}}\n\n", inner_code, no_match_break(in_inner_loop))
}

fn exactly_to_snippet(inner_code: String, in_inner_loop: bool, n: usize) -> String {
    format!("{{
    let mut match_count = 0;

    while current.is_some() {{
        {}

        match_count += 1;

        if match_count == {} {{
            break;
        }}
    }}

    if match_count < {} {{
        {}
    }}
}}\n\n", inner_code, n, n, no_match_break(in_inner_loop))
}

fn bound_to_snippet(inner_code: String, in_inner_loop: bool, n: usize, m: usize) -> String {
    format!("{{
    let mut match_count = 0;

    while current.is_some() {{
        {}

        match_count += 1;

        if match_count == {} {{
            break;
        }}
    }}

    if match_count < {} {{
        {}
    }}
}}\n\n", inner_code, m, n, no_match_break(in_inner_loop))
}

fn above_to_snippet(inner_code: String, in_inner_loop: bool, n: usize) -> String {
    format!("{{
    let mut match_count = 0;

    while current.is_some() {{
        {}

        match_count += 1;
    }}

    if match_count < {} {{
        {}
    }}
}}\n\n", inner_code, n, no_match_break(in_inner_loop))
}

fn base_code(inner: String, capture_count: usize, struct_name: & str, regex: & str) -> String {
    format!("
pub struct {} {{
    named_groups: std::collections::HashMap<& 'static str, usize>
}}

impl {} {{
    pub fn new() -> Self {{
        let named_groups = std::collections::HashMap::new();



        {} {{
            named_groups
        }}
    }}
}}

impl Into<native_regex_lib::native_regex::Engine> for {} {{

    fn into(self) -> native_regex_lib::native_regex::Engine {{
        self.engine()
    }}

}}

impl native_regex_lib::native_regex::NativeRegex for {} {{

    // Function to match regex '{}'
    #[allow(unused_parens)]
    fn step(mut chars: std::str::CharIndices, offset: usize, length: usize) -> Option<Vec<Option<(usize, usize)>>> {{

        let mut captures = vec![None; {}];

        //Advance to first character & bounds check
        let mut current = chars.next();

        if current.is_none() {{ return None; }}

        //Zero capture
        let capture_0_first = current.unwrap().0 + offset;

        {}

        captures[0] = Some((capture_0_first, if current.is_some() {{ current.unwrap().0 + offset }} else {{ length }}));

        return Some(captures)
    }}

    fn capture_names(&self) -> &std::collections::HashMap<& 'static str, usize> {{
        &self.named_groups
    }}


}}
", struct_name, struct_name, struct_name, struct_name, struct_name, regex, capture_count, inner)
}

fn translate_hir(hir: & Hir, capture_names: & mut HashMap<String, usize>, in_inner_loop: bool) -> Result<(String, Option<usize>), String> {

    let mut snippet = String::new();

    let mut max: Option<usize> = None;

    match hir.kind() {
        HirKind::Empty => {
            snippet = empty();
        },
        HirKind::Literal(literal) => match literal {
            Literal::Byte(byte) => {
                snippet = literal_to_snippet(*byte as u32, in_inner_loop);
            },
            Literal::Unicode(ch) => {
                snippet = literal_to_snippet(*ch as u32, in_inner_loop);
            }
        },
        HirKind::Class(class) => match class {
            Class::Unicode(unicode) => {
                snippet = class_to_snippet(unicode.iter().map(|unicode_range| RangeInclusive::new(unicode_range.start() as u32, unicode_range.end() as u32)), in_inner_loop);
            },
            Class::Bytes(bytes) => {
                snippet = class_to_snippet(bytes.iter().map(|unicode_range| RangeInclusive::new(unicode_range.start() as u32, unicode_range.end() as u32)), in_inner_loop);
            }
        },
        HirKind::Anchor(anchor) => match anchor {
            Anchor::EndLine => {

            },
            Anchor::EndText => {

            },
            Anchor::StartLine => {

            },
            Anchor::StartText => {

            }
        },
        HirKind::WordBoundary(boundary) => match boundary {
            WordBoundary::Unicode => {

            }
            WordBoundary::Ascii => {

            }
            WordBoundary::AsciiNegate => {

            }
            WordBoundary::UnicodeNegate => {

            }
        },
        HirKind::Repetition(repeater) => {
            if !repeater.greedy {
                return Err(String::from("Non-greedy repetition is not supported since backtracking is not supported. Please see NativeRegex readme for more details."));
            }

            let (subset, m) = translate_hir(repeater.hir.as_ref(), capture_names, true)?;

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
                    zero_and_one_to_snippet(subset)
                },
                RepetitionKind::OneOrMore => {
                    one_or_more_to_snippet(subset, in_inner_loop)
                },
                RepetitionKind::ZeroOrMore => {
                    zero_or_more_to_snippet(subset)
                },
                RepetitionKind::Range(range) => match range {
                    RepetitionRange::AtLeast(n) => {
                        above_to_snippet(subset, in_inner_loop, n as usize)
                    },
                    RepetitionRange::Bounded(n, m) => {
                        bound_to_snippet(subset, in_inner_loop, n as usize, m as usize)
                    },
                    RepetitionRange::Exactly(n ) => {
                        exactly_to_snippet(subset, in_inner_loop, n as usize)
                    }
                },

            }

        },
        HirKind::Group(group) => {

            let (subset, m) = translate_hir(group.hir.as_ref(), capture_names, in_inner_loop)?;

            snippet = match group.kind.clone() {
                GroupKind::NonCapturing => {
                    noncapturing_to_snippet(subset)
                },
                GroupKind::CaptureIndex(index) => {
                    max = Some(index as usize);
                    capturing_to_snippet(index, subset)
                },
                GroupKind::CaptureName { name, index } => {
                    max = Some(index as usize);
                    capture_names.insert(name, index as usize);

                    capturing_to_snippet(index, subset)
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
                let (subset, m) = translate_hir(hir, capture_names, in_inner_loop)?;


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

pub fn translate( regex: & str, struct_name: & str) -> Result<String, String> {

    match Parser::new().parse(regex) {
        Ok(hir) => {
            let mut map = HashMap::new();
            let (inner, max) = translate_hir(&hir, & mut map, false)?;

            Ok(base_code(inner, if max.is_some() { max.unwrap() } else { 0 } + 1, struct_name, regex))
        }
        Err(e) => {
            Err(e.to_string())
        }
    }


}

/*use crate::parse::*;

fn bounds_check(n: usize) -> String {
    format!("if index + offset + ({} - 1) >= text.len() {{ return None; }}", n)
}

fn envelope(inner_code: String, repeater: &RepeaterType, nomatch: & str) -> String {

    match repeater {
        RepeaterType::ExactlyOnce => {
            inner_code
        },
        RepeaterType::ZeroAndOne => {
        format!("{{
    let mut match_count = 0;

    for _ in &text[index + offset..] {{
        {}

        match_count += 1;

        if match_count == 1 {{
            break;
        }}
    }}

}}\n\n", inner_code)
        },
        RepeaterType::OneAndAbove => {
            format!("{{
    let mut found = false;

    for _ in &text[index + offset..] {{
        {}
        found = true;
    }}

    if !found {{
        {}
    }}
}}\n\n", inner_code, nomatch)
        },
        RepeaterType::ZeroAndAbove => {
            format!("for _ in &text[index + offset..] {{
    {}
}}\n\n", inner_code)
        },
        RepeaterType::ExactlyN(n) => {
            format!("{{
    let mut match_count = 0;

    for _ in &text[index + offset..] {{
        {}

        match_count += 1;

        if match_count == {} {{
            break;
        }}
    }}

    if match_count < {} {{
        {}
    }}
}}\n\n", inner_code, n, n, nomatch)
        },
        RepeaterType::Range(n, m) => {
            format!("{{
    let mut match_count = 0;

    for _ in &text[index + offset..] {{
        {}

        match_count += 1;

        if match_count == {} {{
            break;
        }}
    }}

    if match_count < {} {{
        {}
    }}
}}\n\n", inner_code, m, n, nomatch)
        },
        RepeaterType::NAndAbove(n) => {
            format!("{{
    let mut match_count = 0;

    for _ in &text[index + offset..] {{
        {}

        match_count += 1;
    }}

    if match_count < {} {{
        {}
    }}
}}\n\n", inner_code, n, nomatch)
        }
    }
}

fn get_no_match(inforloop: bool) -> & 'static str {
    if inforloop { "break;" } else { "return None;" }
}

fn token_translate(token: & Token, capture_index: & mut usize, inforloop: bool) -> Result<(bool, String), String> {

    let nomatch = get_no_match(inforloop);

    match token {
        Token::LiteralSingle(character, repeater) => {
            let outernomatch = get_no_match(inforloop);

            let withinnomatch = get_no_match(*repeater != RepeaterType::ExactlyOnce || inforloop);
            Ok((false, envelope(format!("{}\n\nif text[index + offset] != {} {{ {} }}\n\noffset += 1;\n\n", bounds_check(1), *character, withinnomatch), repeater, outernomatch)))
        },
        Token::LiteralList(list) => {
            let mut conditions = format!("text[index + offset] == {}", list[0]);
            for i in 1..list.len() {
                conditions = format!("{} && text[index + offset + {}] == {}", conditions, i, list[i])
            }
            Ok((false, format!("{}\n\nif !({}) {{ {} }}\n\noffset += {};\n\n", bounds_check(list.len()), conditions, nomatch, list.len())))
        },
        Token::Anchor(anchor) => match anchor {
            AnchorType::Start => {
                Ok((false, String::from("if index+offset != 0 { return None; }")))
            },
            AnchorType::End => {
                Ok((false, String::from("if index+offset != text.len() { return None; }")))
            },
            AnchorType::WordBorder => {
                //Err(String::from("Word borders are not supported. Please see readme for more information."))
                Ok((true, format!("if index+offset != 0 && index+offset != text.len() {{
    if (Self::word_class(text[index+offset-1]) || !Self::word_class(text[index+offset])) &&
        (!Self::word_class(text[index+offset-1]) || Self::word_class(text[index+offset])) {{

        {}
    }}
}} else {{

    if index+offset == 0 && !Self::word_class(text[0]) || index+offset == text.len() - 1 && !Self::word_class(text[text.len() - 1]) {{

        {}
    }}
}}", nomatch, nomatch)))
            }
        },
        Token::Alternation => {
            Err(String::from("Alternation is not supported. Please see readme for more information."))
        },
        Token::CharacterClass(set, repeater) => {
            let outernomatch = get_no_match(inforloop);

            let withinnomatch = get_no_match(*repeater != RepeaterType::ExactlyOnce || inforloop);

            Ok((false, envelope(format!("{}\n\nif {} {{ {} }}\n\noffset += 1;\n\n", bounds_check(1), set.code("text[index+offset]"), withinnomatch), repeater, outernomatch)))
        },
        Token::Group(ast, repeater, group, _name) => {

            //If the parent ast node is in a for loop, or the repeater of this capture group is a for loop
            let isinforloop = *repeater != RepeaterType::ExactlyOnce || inforloop;


            let (word_boundary, code)  = match group {
                GroupType::Capturing => {
                    let capture_start = format!("let capture_{}_start = index+offset;\n\n", *capture_index);
                    let capture_end = format!("captures.push(Some((capture_{}_start, index + offset)));\n\n", *capture_index);

                    *capture_index += 1;

                    let (word_boundary, inner_code ) = translate_ast(ast, capture_index, isinforloop)?;

                    (word_boundary, envelope(format!("{{\n\n{}{}{}}}\n\n", capture_start, inner_code, capture_end), repeater, nomatch))
                },
                GroupType::NonCapturing => {
                    let (word_boundary, inner_code) = translate_ast(ast, capture_index, isinforloop)?;

                    (word_boundary, envelope(format!("{{\n\n{}}}\n\n", inner_code), repeater, nomatch))
                }
            };


            Ok((word_boundary, code))
        },
        Token::DotMatch(_) => {
            Err(String::from("Dot matching is not supported. Please see readme for more information."))
        }
    }
}

fn translate_ast(ast: & NativeRegexAST, capture_index: & mut usize, inforloop: bool) -> Result<(bool, String), String> {
    let mut code = String::new();

    let mut word_boundary = false;

    for token in ast.tokens.iter() {

        match token_translate(token, capture_index, inforloop) {
            Ok((wb, token_code)) => {
                //code = format!("{}{}", code, token_code);
                code.push_str(&token_code);
                word_boundary |= wb;
            }
            Err(e) => {
                return Err(e);
            }
        }

    }

    Ok((word_boundary, code))
}

pub fn translate(regex: & str, struct_name: & str) -> Result<String, String> {

    //Add the base code, including capture array/vector and custom function name

    match regex::Regex::new(regex) {
        Ok(_) => {
            let mut capture_index = 1;


            let ast = crate::parse::NativeRegexAST::from(regex.as_bytes());

            match translate_ast(&ast, & mut capture_index, false) {
                Ok((_word_boundary, tree_code)) => {

                    let (_, table) = ast.get_captures(1);

                    let mut hashmap_init_code = String::new();

                    for (name, index) in table.iter() {
                        hashmap_init_code.push_str(&format!("\t\tnamed_groups.insert(\"{}\", {});\n", name, index))
                    }

                    Ok(format!("
pub struct {} {{
    named_groups: std::collections::HashMap<& 'static str, usize>
}}

impl {} {{
    pub fn new() -> Self {{
        let named_groups = std::collections::HashMap::new();



        {} {{
            named_groups
        }}
    }}
}}

impl Into<native_regex_lib::native_regex::Engine> for {} {{

    fn into(self) -> native_regex_lib::native_regex::Engine {{
        self.engine()
    }}

}}

impl native_regex_lib::native_regex::NativeRegex for {} {{

    // Function to match regex '{}'
    #[allow(unused_parens)]
    fn step(mut chars: CharIndices, offset: usize, length: usize) -> Option<Vec<Option<(usize, usize)>>> {{

        let mut captures = vec![None; {}];

        //Advance to first character & bounds check
        let mut current = chars.next();

        if current.is_none() {{ return None; }}

        //Zero capture
        let capture_0_first = current.unwrap().0 + offset;

        {}

        captures[0] = Some((capture_0_first, if current.is_some() {{ current.unwrap().0 + offset }} else {{ length }}));

        return Some(captures)
    }}

    fn capture_names(&self) -> &std::collections::HashMap<& 'static str, usize> {{
        &self.named_groups
    }}


}}
", struct_name, struct_name, struct_name, struct_name, struct_name, regex, 3, tree_code))


                }
                Err(e) => {
                    Err(e)
                }
            }

        }
        Err(e) => {
            Err(format!("Invalid regex - {}", e))
        }
    }



}

*/