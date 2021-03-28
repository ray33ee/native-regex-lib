
use std::collections::HashMap;
use std::ops::RangeInclusive;

pub struct RustCompiler;

impl crate::compiler::Compiler for RustCompiler {

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
        format!("if (current.unwrap().current() as u32) != {} {{ {} }}\n\n", ch, Self::no_match_break(in_inner_loop))
    }

    fn range_to_snippet(range: RangeInclusive<u32>, is_first_and_only: bool) -> String {
        let (start, end) = range.into_inner();

        if start == end {
            format!("(current.unwrap().current() as u32) != {}", start)
        } else {
            if is_first_and_only {
                format!("(current.unwrap().current() as u32) < {} || (current.unwrap().current() as u32) > {}", start, end)
            } else {
                format!("((current.unwrap().current() as u32) < {} || (current.unwrap().current() as u32) > {})", start, end)
            }

        }
    }

    fn class_to_snippet(ranges: Vec<RangeInclusive<u32>>, in_inner_loop: bool) -> String {

        let mut ranges = ranges.into_iter();

        let mut str_ranges = Self::range_to_snippet(ranges.next().unwrap(), false);

        for range in ranges {
            str_ranges.push_str(" && ");
            str_ranges.push_str(&Self::range_to_snippet(range, false));
        }

        format!("if {} {{\n {} \n}}\n\n", str_ranges, Self::no_match_break(in_inner_loop))
    }

    fn non_capturing_to_snippet(snippet: String) -> String {
        format!("{{\n\n{}}}\n\n", snippet)
    }

    fn capturing_to_snippet(ind: u32, snippet: String) -> String {
        let capture_start = format!("let capture_{}_start = current.unwrap().index();\n\n", ind);
        let capture_end = format!("captures[{}] = Some((capture_{}_start, if current.is_some() {{ current.unwrap().index() }} else {{ length }}));\n\n", ind, ind);

        format!("{{\n\n{}{}{}}}\n\n", capture_start, snippet, capture_end)
    }

    fn map_to_snippet(map: HashMap<String, usize>) -> String {
        let mut snippet = String::new();

        //named_groups.insert("name", 23);

        for (name, index) in map {
            snippet.push_str(&format!("named_groups.insert(\"{}\", {});\n", name, index));
        }

        snippet
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

}}\n\n", inner_code, Self::no_match_break(in_inner_loop))
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
}}\n\n", inner_code, n, n, Self::no_match_break(in_inner_loop))
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
}}\n\n", inner_code, m, n, Self::no_match_break(in_inner_loop))
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
    fn step(mut chars: native_regex_lib::native_regex::CharOffsetIndices, length: usize) -> Option<Vec<Option<(usize, usize)>>> {{

        let mut captures = vec![None; {}];

        //Advance to first character & bounds check
        let mut current = chars.next();

        if current.is_none() {{ return None; }}

        //Zero capture
        let capture_0_first = current.unwrap().index();

        {}

        captures[0] = Some((capture_0_first, if current.is_some() {{ current.unwrap().index() }} else {{ length }}));

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

    fn start_text_snippet() -> String {
        format!("if current.unwrap().previous() != native_regex_lib::native_regex::Previous::Start {{ return None; }}\n\n")
    }

    fn end_text_snippet() -> String {
        format!("if current.is_some() {{ return None; }}\n\n")
    }

    fn start_line_snippet() -> String {
        format!("if current.unwrap().previous() != native_regex_lib::native_regex::Previous::Character('\\n') && current.unwrap().previous() != native_regex_lib::native_regex::Previous::Start {{ return None; }}\n\n")
    }

    fn end_line_snippet() -> String {
        //format!("if current.unwrap().current() != native_regex_lib::native_regex::Previous::Character(\"\\n\") {{ return None; }}\n\n")
        format!("if current.unwrap().previous() != native_regex_lib::native_regex::Previous::Character('\\n') && {{ {} current.is_some() }} {{ return None; }}\n\n", Self::advance())
    }
}
