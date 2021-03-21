use crate::parse::*;

fn bounds_check(n: usize) -> String {
    format!("if index + counter + ({} - 1) > text.len() {{ index += 1; continue 'main; }}", n)
}

fn envelope(inner_code: String, repeater: &RepeaterType, nomatch: & str) -> String {

    match repeater {
        RepeaterType::ExactlyOnce => {
            inner_code
        },
        RepeaterType::ZeroAndOne => {
        format!("{{
    let mut match_count = 0;

    for _ in &text[index + counter..] {{
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

    for _ in &text[index + counter..] {{
        {}
        found = true;
    }}

    if !found {{
        {}
    }}
}}\n\n", inner_code, nomatch)
        },
        RepeaterType::ZeroAndAbove => {
            format!("for _ in &text[index + counter..] {{
    {}
}}\n\n", inner_code)
        },
        RepeaterType::ExactlyN(n) => {
            format!("{{
    let mut match_count = 0;

    for _ in &text[index + counter..] {{
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

    for _ in &text[index + counter..] {{
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

    for _ in &text[index + counter..] {{
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
    if inforloop { "break;" } else { "index += 1; continue;" }
}

fn token_translate(token: & Token, capture_index: & mut usize, inforloop: bool) -> Result<(bool, String), String> {

    let nomatch = get_no_match(inforloop);

    match token {
        Token::LiteralSingle(character, repeater) => {
            let outernomatch = get_no_match(inforloop);

            let withinnomatch = get_no_match(*repeater != RepeaterType::ExactlyOnce || inforloop);
            Ok((false, envelope(format!("{}\n\nif text[index + counter] != {} {{ {} }}\n\ncounter += 1;\n\n", bounds_check(1), *character, withinnomatch), repeater, outernomatch)))
        },
        Token::LiteralList(list) => {
            let mut conditions = format!("text[index + counter] == {}", list[0]);
            for i in 1..list.len() {
                conditions = format!("{} && text[index + counter + {}] == {}", conditions, i, list[i])
            }
            Ok((false, format!("{}\n\nif !({}) {{ {} }}\n\ncounter += {};\n\n", bounds_check(list.len()), conditions, nomatch, list.len())))
        },
        Token::Anchor(anchor) => match anchor {
            AnchorType::Start => {
                Ok((false, String::from("if index+counter != 0 { return None; }")))
            },
            AnchorType::End => {
                Ok((false, String::from("if index+counter != text.len() { index += 1; continue; }")))
            },
            AnchorType::WordBorder => {
                //Err(String::from("Word borders are not supported. Please see readme for more information."))
                Ok((true, format!("if index+counter != 0 && index+counter != str_text.len() {{
    if (word_class(text[index+counter-1]) || !word_class(text[index+counter])) &&
        (!word_class(text[index+counter-1]) || word_class(text[index+counter])) {{

        {}
    }}
}} else {{

    if index+counter == 0 && !word_class(text[0]) || index+counter == str_text.len() - 1 && !word_class(text[str_text.len() - 1]) {{

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

            Ok((false, envelope(format!("{}\n\nif {} {{ {} }}\n\ncounter += 1;\n\n", bounds_check(1), set.code("text[index+counter]"), withinnomatch), repeater, outernomatch)))
        },
        Token::Group(ast, repeater, group, _name) => {

            //If the parent ast node is in a for loop, or the repeater of this capture group is a for loop
            let isinforloop = *repeater != RepeaterType::ExactlyOnce || inforloop;


            let (word_boundary, code)  = match group {
                GroupType::Capturing => {
                    let capture_start = format!("let capture_{}_start = index+counter;\n\n", *capture_index);
                    let capture_end = format!("captures[{}] = Some((capture_{}_start, index + counter));\n\n", *capture_index, *capture_index);

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
                code = format!("{}{}", code, token_code);
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
                Ok((word_boundary, tree_code)) => {

                    let (count, table) = ast.get_captures(1);

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
    fn regex_function(&self, str_text: &str, start: usize) -> Option<Vec<Option<(usize, usize)>>> {{


        let text = str_text.as_bytes();

        let mut index = start;

        let mut captures = vec![None; {}];

        {}

        'main: while index < text.len() {{

            //Start counter
            let mut counter = 0;

            let capture_0_start = index + counter;

            {}

            captures[0] = Some((capture_0_start, index+counter));

            return Some(captures);
        }}


        None
    }}


    fn capture_names(&self) -> std::collections::HashMap<& 'static str, usize> {{
        let {}name_map = std::collections::HashMap::new();

        {}

        name_map
    }}


}}
", struct_name, struct_name, struct_name, struct_name, regex, count,
                               if word_boundary { "let word_class = |ch: u8| { ch >= 48 && ch <= 57 || ch >= 65 && ch <= 90 || ch == 95 || ch >= 97 && ch <= 122 };"
} else {""}, tree_code, if table.is_empty() {""} else {"mut "}, hashmap_init_code))


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

