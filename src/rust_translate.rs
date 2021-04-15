
use std::collections::HashMap;
use std::ops::RangeInclusive;

pub struct RustCompiler;

impl crate::compiler::Compiler for RustCompiler {

    fn no_match_break(in_inner_loop: bool) -> & 'static str {
        if in_inner_loop { "break;" } else { "return None;" }
    }

    //Advance
    fn advance() -> & 'static str {
        "character = chars.advance();\n\n"
    }

    //If(Is, End, Stop)
    fn bounds_check() -> & 'static str {
        "if character.current().is_none() { return None; }\n\n"
    }

    //Empty
    fn empty() -> String {
        String::new()
    }

    //If(Not, Literal(_), Stop or Break)
    fn literal_to_snippet(ch: u32, in_inner_loop: bool) -> String {
        format!("if (character.current().unwrap() as u32) != {} {{ {} }}\n\n", ch, Self::no_match_break(in_inner_loop))
    }

    fn range_to_snippet(range: RangeInclusive<u32>, is_first_and_only: bool) -> String {
        let (start, end) = range.into_inner();

        if start == end {
            format!("(character.current().unwrap() as u32) != {}", start)
        } else {
            if is_first_and_only {
                format!("(character.current().unwrap() as u32) < {} || (character.current().unwrap() as u32) > {}", start, end)
            } else {
                format!("((character.current().unwrap() as u32) < {} || (character.current().unwrap() as u32) > {})", start, end)
            }

        }
    }

    //If(Not, CharacterSet(_), Stop or Break)
    fn class_to_snippet(ranges: Vec<RangeInclusive<u32>>, in_inner_loop: bool) -> String {

        let mut ranges = ranges.into_iter();

        let mut str_ranges = Self::range_to_snippet(ranges.next().unwrap(), false);

        for range in ranges {
            str_ranges.push_str(" && ");
            str_ranges.push_str(&Self::range_to_snippet(range, false));
        }

        format!("if {} {{\n {} \n}}\n\n", str_ranges, Self::no_match_break(in_inner_loop))
    }

    //Block(...)
    fn non_capturing_to_snippet(snippet: String) -> String {
        format!("{{\n\n{}}}\n\n", snippet)
    }

    //Block(CaptureBeginning, ..., CaptureEnd)
    fn capturing_to_snippet(ind: u32, snippet: String) -> String {
        let capture_start = format!("let capture_{}_start = character.index();\n\n", ind);
        let capture_end = format!("captures[{}] = Some((capture_{}_start, character.index()));\n\n", ind, ind);

        format!("{{\n\n{}{}{}}}\n\n", capture_start, snippet, capture_end)
    }

    //Not a token. Just return a bunch of (index, & str) pairs
    fn map_to_snippet(map: HashMap<String, usize>) -> String {
        let mut snippet = String::new();

        //named_groups.insert("name", 23);

        for (name, index) in map {
            snippet.push_str(&format!("named_groups.insert(\"{}\", {});\n", name, index));
        }

        snippet
    }


    //Block(StartCount, While(..., IncrementCount, If(Is, CountEquals(m), Break)), If(Is, CountLessThen(n), Stop or Break))
    fn bounded_to_snippet(inner_code: String, in_inner_loop: bool, n: usize, m: usize) -> String {
        format!("{{
    let mut match_count = 0;

    while character.current().is_some() {{
        {}

        match_count += 1;

        if match_count == {} {{
            break;
        }}
    }}

    if match_count < {} {{
        {}
    }}
}}\n\n", inner_code, m, n, Self::no_match_break(in_inner_loop))
    }

    //Block(StartCount, While(..., IncrementCount), If(Is, CountLessThen(n), Stop or Break))
    fn unbounded_to_snippet(inner_code: String, in_inner_loop: bool, n: usize) -> String {
        format!("{{
    let mut match_count = 0;

    while character.current().is_some() {{
        {}

        match_count += 1;
    }}

    if match_count < {} {{
        {}
    }}
}}\n\n", inner_code, n, Self::no_match_break(in_inner_loop))
    }

    fn base_code(inner: String, capture_count: usize, struct_name: & str, regex: & str, map: HashMap<String, usize>) -> String {



        format!("
pub struct {} {{
    named_groups: std::collections::HashMap<& 'static str, usize>
}}

impl {} {{
    pub fn new() -> Self {{
        let {}named_groups = std::collections::HashMap::new();

        {}

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
    #[allow(unused_parens, unused_comparisons)]
    fn step(mut chars: native_regex_lib::native_regex::character::CharOffsetIndices) -> Option<Vec<Option<(usize, usize)>>> {{

        let mut captures = vec![None; {}];

        //Advance to first character & bounds check
        let mut character = chars.advance();

        if character.current().is_none() {{ return None; }}

        //Zero capture
        let capture_0_first = character.index();

        {}

        captures[0] = Some((capture_0_first, character.index()));

        return Some(captures)
    }}

    fn capture_names(&self) -> &std::collections::HashMap<& 'static str, usize> {{
        &self.named_groups
    }}


}}
", struct_name, struct_name, if map.is_empty() {""} else {"mut "}, Self::map_to_snippet(map), struct_name, struct_name, struct_name, regex, capture_count, inner)
    }

    fn name_identifier_validator(_identifier: & str) -> Result<(), String> {
        Ok(())
    }

    //If(Not, Anchor(Regular, Start), Stop)
    fn start_text_snippet() -> String {
        format!("if character.previous() != native_regex_lib::native_regex::character::Previous::Start {{ return None; }}\n\n")
    }

    //If(Not, Anchor(Regular, End), Stop)
    fn end_text_snippet() -> String {
        format!("if character.current().is_some() {{ return None; }}\n\n")
    }

    //If(Not, Anchor(Newline, Start), Stop)
    fn start_line_snippet() -> String {
        format!("if character.previous() != native_regex_lib::native_regex::character::Previous::Character('\\n') && character.previous() != native_regex_lib::native_regex::character::Previous::Start {{ return None; }}\n\n")
    }

    //If(Not, Anchor(Newline, End), Stop)
    fn end_line_snippet() -> String {
        format!("if character.current().is_some() {{ if character.current().unwrap() != '\\n' {{ return None; }} }}\n\n")

    }

    fn wordboundary_unicode(in_inner_loop: bool) -> String {
        format!("
if character.previous() != native_regex_lib::native_regex::character::Previous::Start && character.current().is_some() {{
    if (Self::is_word_character(character.previous().unwrap()) || !Self::is_word_character(character.current().unwrap())) &&
        (!Self::is_word_character(character.previous().unwrap()) || Self::is_word_character(character.current().unwrap())) {{
        {}
    }}
}} else {{
    if character.previous() == native_regex_lib::native_regex::character::Previous::Start && !Self::is_word_character(character.current().unwrap()) || character.current().is_none() && !Self::is_word_character(character.previous().unwrap()) {{
        {}
    }}
}}", Self::no_match_break(in_inner_loop), Self::no_match_break(in_inner_loop))
    }

    fn wordboundary_ascii(in_inner_loop: bool) -> String {
        format!("
if character.previous() != native_regex_lib::native_regex::character::Previous::Start && character.current().is_some() {{
    if (Self::is_word_byte(character.previous().unwrap()) || !Self::is_word_byte(character.current().unwrap())) &&
        (!Self::is_word_byte(character.previous().unwrap()) || Self::is_word_byte(character.current().unwrap())) {{
        {}
    }}
}} else {{
    if character.previous() == native_regex_lib::native_regex::character::Previous::Start && !Self::is_word_byte(character.current().unwrap()) || character.current().is_none() && !Self::is_word_byte(character.previous().unwrap()) {{
        {}
    }}
}}", Self::no_match_break(in_inner_loop), Self::no_match_break(in_inner_loop))
    }


    fn negate_wordboundary_unicode(in_inner_loop: bool) -> String {
        format!("
if character.previous() != native_regex_lib::native_regex::character::Previous::Start && character.current().is_some() {{
    if (Self::is_word_character(character.previous().unwrap()) || !Self::is_word_character(character.current().unwrap())) &&
        (!Self::is_word_character(character.previous().unwrap()) || Self::is_word_character(character.current().unwrap())) {{

    }} else {{ {} }}
}} else {{
    if character.previous() == native_regex_lib::native_regex::character::Previous::Start && !Self::is_word_character(character.current().unwrap()) || character.current().is_none() && !Self::is_word_character(character.previous().unwrap()) {{

    }} else {{ {} }}
}}", Self::no_match_break(in_inner_loop), Self::no_match_break(in_inner_loop))
    }

    fn negate_wordboundary_ascii(in_inner_loop: bool) -> String {
        format!("
if character.previous() != native_regex_lib::native_regex::character::Previous::Start && character.current().is_some() {{
    if (Self::is_word_byte(character.previous().unwrap()) || !Self::is_word_byte(character.current().unwrap())) &&
        (!Self::is_word_byte(character.previous().unwrap()) || Self::is_word_byte(character.current().unwrap())) {{

    }} else {{ {} }}
}} else {{
    if character.previous() == native_regex_lib::native_regex::character::Previous::Start && !Self::is_word_byte(character.current().unwrap()) || character.current().is_none() && !Self::is_word_byte(character.previous().unwrap()) {{

    }} else {{ {} }}
}}", Self::no_match_break(in_inner_loop), Self::no_match_break(in_inner_loop))
    }
}
