use crate::parse::*;

fn bounds_check(n: usize) -> String {
    format!("if index + offset + ({} - 1) > text.len() {{ return None; }}", n)
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
    if (self.word_class(text[index+offset-1]) || !self.word_class(text[index+offset])) &&
        (!self.word_class(text[index+offset-1]) || self.word_class(text[index+offset])) {{

        {}
    }}
}} else {{

    if index+offset == 0 && !self.word_class(text[0]) || index+offset == text.len() - 1 && !self.word_class(text[text.len() - 1]) {{

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
                        hashmap_init_code.push_str(&format!("\t\tname_map.insert(\"{}\", {});\n", name, index))
                    }

                    Ok(format!("
pub struct {};

impl {} {{
    pub fn new() -> Self {{
        {}
    }}
}}

impl native_regex_lib::native_regex::NativeRegex for {} {{

    // Function to match regex '{}'
    #[allow(unused_parens)]
    fn step(&self, captures: & mut Vec<Option<(usize, usize)>>, text: & [u8], index: usize) -> Option<()> {{

        let mut offset = 0;

        let capture_0_start = index+offset;

        {}

        captures.insert(0, Some((capture_0_start, index+offset)));

        return Some(());
    }}

    fn capture_names(&self) -> std::collections::HashMap<& 'static str, usize> {{
        let {}name_map = std::collections::HashMap::new();

        {}

        name_map
    }}


}}
", struct_name, struct_name, struct_name, struct_name, regex, tree_code, if table.is_empty() {""} else {"mut "}, hashmap_init_code))


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

