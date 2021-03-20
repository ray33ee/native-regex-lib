
use crate::characterset::CharacterSet;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum RepeaterType {
    ExactlyOnce, //No repetition
    ZeroAndOne, //?
    ZeroAndAbove, //*
    OneAndAbove, //+
    ExactlyN(usize), //{N}
    NAndAbove(usize), //{N,}
    Range(usize, usize) //{N, M}
}

struct Repeater {
    repeater: RepeaterType,
    length: usize
}

#[derive(Debug)]
pub enum GroupType {
    Capturing, //()
    NonCapturing //(?:)
}

#[derive(Debug)]
pub enum AnchorType {
    Start, //^
    End, //$
    WordBorder, // \b
}

/*#[derive(Debug)]
pub struct CharacterSet {
    pub inverted: bool,
    pub set: Vec<(bool, RangeInclusive<u8>)>
}*/

#[derive(Debug)]
pub enum Token {
    CharacterClass(CharacterSet, RepeaterType), //Can be a traditional character class, [] or a shorthand character class
    Anchor(AnchorType),
    LiteralSingle(u8, RepeaterType),
    LiteralList(Vec<u8>),
    Group(NativeRegexAST, RepeaterType, GroupType, Option<String>),
    DotMatch(RepeaterType),
    Alternation,
}

#[derive(Debug)]
pub struct NativeRegexAST {
    pub tokens: Vec<Token>
}



impl From<& [u8]> for Repeater{

    fn from(slice: & [u8]) -> Self {


        let (repeater, length) = {
            if !slice.is_empty() {
                let first_char = slice[0];

                if first_char == '?' as u8 {
                    (RepeaterType::ZeroAndOne, 1)
                } else if first_char == '+' as u8 {
                    (RepeaterType::OneAndAbove, 1)
                } else if first_char == '*' as u8 {
                    (RepeaterType::ZeroAndAbove, 1)
                } else if first_char == '{' as u8 {
                    let mut close_index = 1;

                    for ch in &slice[1..] {
                        if *ch == '}' as u8 {
                            break;
                        }
                        close_index += 1;
                    }

                    let mut substrings = (&slice[1..close_index]).split(|ch| *ch == ',' as u8);

                    unsafe {
                        let first_number = std::str::from_utf8_unchecked(substrings.next().unwrap()).parse::<usize>().unwrap();


                        match substrings.next() {
                            Some(second_substring) => {
                                if second_substring.is_empty() {
                                    (RepeaterType::NAndAbove(first_number), close_index + 1)
                                } else {
                                    (RepeaterType::Range(first_number, std::str::from_utf8_unchecked(second_substring).parse::<usize>().unwrap()), close_index + 1)
                                }
                            }
                            None => {
                                (RepeaterType::ExactlyN(first_number), close_index + 1)
                            }
                        }
                    }
                } else {
                    (RepeaterType::ExactlyOnce, 0)
                }
            }
            else {
                (RepeaterType::ExactlyOnce, 0)
            }
        };

        Repeater {
            repeater,
            length,
        }
    }
}

impl Token {
    fn from(slice: & [u8]) -> (Self, & [u8]) {

        let first_char = slice[0];

        if first_char == '[' as u8 {

            let mut close_index = 1;

            for ch in &slice[1..] {
                if *ch == ']' as u8 {
                    break;
                }
                close_index += 1;
            }

            let repeater = Repeater::from(&slice[close_index+1..]);

            (Token::CharacterClass(CharacterSet::from(&slice[1..close_index]), repeater.repeater), &slice[close_index+repeater.length+1..])

        } else if first_char == '|' as u8 {
            (Token::Alternation, &slice[1..])
        } else if first_char == '$' as u8 {
            (Token::Anchor(AnchorType::End), &slice[1..])
        } else if first_char == '^' as u8 {
            (Token::Anchor(AnchorType::Start), &slice[1..])
        } else if first_char == '\\' as u8{

            //If the next character is the character of a shorthand class, return a character class for that type.
            //Otherwise return the escaped character

            let second_character = slice[1];

            if second_character == 'b' as u8 {
                (Token::Anchor(AnchorType::WordBorder), &slice[2..])
            }
            else {
                let repeater = Repeater::from(&slice[2..]);

                if second_character == 's' as u8 || second_character == 'w' as u8 || second_character == 'd' as u8 || second_character == 'D' as u8 || second_character == 'S' as u8 || second_character == 'W' as u8 {
                    (Token::CharacterClass(CharacterSet::from(&slice[0..2]), repeater.repeater), &slice[repeater.length+2..])
                } else if second_character == 'n' as u8 {
                    (Token::LiteralSingle('\n' as u8, repeater.repeater), &slice[repeater.length+2..])
                } else if second_character == 't' as u8 {
                    (Token::LiteralSingle('\t' as u8, repeater.repeater), &slice[repeater.length+2..])
                } else if second_character == 'r' as u8 {
                    (Token::LiteralSingle('\r' as u8, repeater.repeater), &slice[repeater.length+2..])
                }
                else {
                    (Token::LiteralSingle(second_character, repeater.repeater), &slice[repeater.length+2..])
                }
            }



        } else if first_char == '.' as u8 {
            let repeater = Repeater::from(&slice[1..]);

            (Token::DotMatch(repeater.repeater), &slice[repeater.length+1..])

        } else if first_char == '(' as u8 {
            let mut nest_depth = 1;
            let mut close_index = 1;

            for ch in &slice[1..] {
                if *ch == '(' as u8 {
                    nest_depth += 1;
                } else if *ch == ')' as u8 {
                    nest_depth -= 1;

                    if nest_depth == 0 {
                        break;
                    }
                }
                close_index += 1;
            }

            let repeater = Repeater::from(&slice[close_index+1..]);

            if slice[1] == '?' as u8 {
                if slice[2] == ':' as u8 {
                    (Token::Group(NativeRegexAST::from(&slice[3..close_index]),
                                  repeater.repeater,
                                  GroupType::NonCapturing,
                                  None), &slice[close_index+repeater.length+1..])
                } else if slice[2] == 'P' as u8 && slice[3] == '<' as u8 {

                    let mut position = 5;

                    for (i, ch) in (&slice[4..]).iter().enumerate() {
                        position = i + 4;
                        if *ch == '>' as u8 {
                            break;
                        }
                    }

                    unsafe {
                        (Token::Group(NativeRegexAST::from(&slice[position + 1..close_index]),
                                      repeater.repeater,
                                      GroupType::Capturing,
                                      Some(String::from(std::str::from_utf8_unchecked(&slice[4..position])))), &slice[close_index + repeater.length + 1..])
                    }

                } else {
                    panic!(format!("Group modifier {} not supported.", slice[2]))
                }

            } else {
                (Token::Group(NativeRegexAST::from(&slice[1..close_index]),
                              repeater.repeater,
                              GroupType::Capturing,
                              None), &slice[close_index+repeater.length+1..])
            }



        } else {
            //Match as literal?

            /*

              - Keep searching until we find a character that's not a literal character
              - If the terminating character is a repeater then stop before the last character and ship as LiteralList
              - Otherwise ship as a LiteralSingle
              - If we have a single charcter with no repeater, ship as LiteralSingle.
              - This one might take a bit more thought...
              - Don't forget escaped characters like \+

             */

            let mut finish_index = 1;

            for ch in &slice[1..] {

                if *ch == '+' as u8 || *ch == '*' as u8 || *ch == '?' as u8 || *ch == '{' as u8 {
                    return if finish_index == 1 {
                        let repeater = Repeater::from(&slice[1..]);
                        (Token::LiteralSingle(first_char, repeater.repeater), &slice[repeater.length + 1..])
                    } else {
                        (Token::LiteralList(Vec::from(&slice[0..finish_index - 1])), &slice[finish_index - 1..])
                    }


                } else if *ch == '^' as u8 || *ch == '$' as u8 || *ch == '.' as u8 || *ch == '|' as u8 || *ch == '(' as u8 || *ch == ')' as u8 || *ch == '[' as u8 || *ch == '{' as u8 || *ch == '\\' as u8 {
                    return if finish_index == 1 {
                        (Token::LiteralSingle(first_char, RepeaterType::ExactlyOnce), &slice[1..])
                    } else {
                        (Token::LiteralList(Vec::from(&slice[0..finish_index])), &slice[finish_index..])
                    }
                } else {

                }

                finish_index += 1;
            }

            (Token::LiteralList(Vec::from(&slice[0..finish_index])), &slice[finish_index..])

           // panic!(format!("Character '{}' not yet implemented - Match anything else", first_char));
        }

    }
}

impl NativeRegexAST {

    pub fn get_captures(& self, start: usize) -> (usize, HashMap<String, usize>) {
        let mut total = start; //We add one because on match we capture the entire match

        let mut hash = HashMap::new();

        for token in self.tokens.iter() {
            match token {
                Token::Group(ast, _, group, name_option) => {

                    if name_option.is_some() {
                        hash.insert(name_option.as_ref().unwrap().clone(), total);
                    }

                    match group {
                        GroupType::Capturing => {
                            total += 1;
                            let (count, table) = ast.get_captures(total);
                            total = count;
                            hash.extend(table);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        (total, hash)
    }

    fn recur_tree(& self, level: usize) {

        let mut tabs = String::new();

        for _ in 0..level {
            tabs = format!("{}\t", tabs);
        }

        for token in self.tokens.iter() {
            match token {
                Token::Group(ast, repeater, group, name) => {
                    println!("{}Group {:?} {:?} {:?}\n", tabs, repeater, group, name);
                    ast.recur_tree(level+1);
                },
                Token::LiteralList(list) => {
                    print!("{}LiteralList '", tabs);
                    for ch in list {
                        print!("{}", *ch as char);
                    }
                    print!("'\n");
                },
                Token::LiteralSingle(ch, repeater) => {
                    println!("{}LiteralSingle '{}' {:?}", tabs, *ch as char, repeater);
                }
                Token::CharacterClass(set, repeeater) => {
                    println!("{}{:?} {}\n", tabs, repeeater, set);
                },
                _ => {
                    println!("{}{:?}\n", tabs, token);
                }
            }

        }



    }

    pub fn tree(& self) {
        self.recur_tree(0);
    }

}

impl From<& [u8]> for NativeRegexAST {
    fn from(regex: & [u8]) -> Self {

        let mut tokens = Vec::new();

        let mut remainder = regex;

        while !remainder.is_empty() {

            let (tok, rem) = Token::from(remainder);

            remainder = rem;

            tokens.push(tok);


        }

        NativeRegexAST {
            tokens
        }
    }
}

