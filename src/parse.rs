
use std::ops::RangeInclusive;

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

#[derive(Debug)]
pub struct CharacterSet {
    pub inverted: bool,
    pub set: Vec<RangeInclusive<u8>>
}

#[derive(Debug)]
pub enum Token {
    CharacterClass(CharacterSet, RepeaterType), //Can be a traditional character class, [] or a shorthand character class
    Anchor(AnchorType),
    LiteralSingle(u8, RepeaterType),
    LiteralList(Vec<u8>),
    Group(NativeRegexAST, RepeaterType, GroupType),
    DotMatch(RepeaterType),
    Alternation,
}

#[derive(Debug)]
pub struct NativeRegexAST {
    pub tokens: Vec<Token>
}

impl std::fmt::Display for CharacterSet {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", if self.inverted { "inverted" } else { "not inverted" })?;

        for range in self.set.iter() {
            if *range.start() == *range.end() {
                write!(f, ", {}", *range.start() as char)?;
            } else {
                write!(f, ", {}-{}", *range.start() as char, *range.end() as char)?;
            }
        }




        Ok(())
    }

}

impl From<& [u8]> for CharacterSet {
    fn from(mut slice: & [u8]) -> Self {

        let mut set = Vec::new();

        //Look for a '^' and chop it off
        let inverted = if slice[0] == '^' as u8 {
            slice = &slice[1..];
            true
        } else {
            false
        };

        //Similarly if we have leading or trailing '-' add them and chop the string
        if slice[0] == '-' as u8 {
            slice = &slice[1..];
            set.push(RangeInclusive::new('-' as u8, '-' as u8));
        }

        if slice[slice.len() - 1] == '-' as u8 {
            slice = &slice[..slice.len()-1];
            set.push(RangeInclusive::new('-' as u8, '-' as u8));
        }

        let mut index = 0;

        while index < slice.len() {

            let mut counter = 0;

            //Get current token
            let ch = if slice[index] == '\\' as u8 {

                if slice[index+1] == 'd' as u8 {
                    set.push(RangeInclusive::new('0' as u8, '9' as u8));
                    index += 2;
                    continue;
                } else if slice[index+1] == 's' as u8 {
                    set.push(RangeInclusive::new(' ' as u8, ' ' as u8));
                    set.push(RangeInclusive::new('\t' as u8, '\t' as u8));
                    set.push(RangeInclusive::new('\n' as u8, '\n' as u8));
                    set.push(RangeInclusive::new('\r' as u8, '\r' as u8));
                    set.push(RangeInclusive::new(12, 12)); //Form feed
                    index += 2;
                    continue;
                } else if slice[index+1] == 'w' as u8 {
                    set.push(RangeInclusive::new('A' as u8, 'Z' as u8));
                    set.push(RangeInclusive::new('a' as u8, 'z' as u8));
                    set.push(RangeInclusive::new('0' as u8, '9' as u8));
                    set.push(RangeInclusive::new('_' as u8, '_' as u8));
                    index += 2;
                    continue;
                } else if slice[index+1] == 't' as u8 {
                    set.push(RangeInclusive::new('\t' as u8, '\t' as u8));
                    index += 2;
                    continue;
                } else if slice[index+1] == 'n' as u8 {
                    set.push(RangeInclusive::new('\n' as u8, '\n' as u8));
                    index += 2;
                    continue;
                } else if slice[index+1] == 'r' as u8 {
                    set.push(RangeInclusive::new('\r' as u8, '\r' as u8));
                    index += 2;
                    continue;
                }
                else {
                    counter += 2;
                    slice[index+1]
                }



            } else {
                counter += 1;
                slice[index]
            };

            if index + counter < slice.len() {

                //Get next token. If it's a '-' we have a range of characters
                if slice[index + counter] == '-' as u8 {
                    set.push(RangeInclusive::new(ch as u8, slice[index + counter + 1] as u8));
                    index += counter + 2;
                } else {
                    set.push(RangeInclusive::new(ch, ch));
                    index += counter;
                }
            } else {
                set.push(RangeInclusive::new(ch, ch));
                index += counter;
            }

        }

        CharacterSet {
            inverted,
            set
        }
    }
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

                if second_character == 's' as u8 || second_character == 'w' as u8 || second_character == 'd' as u8 {
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

            if slice[1] == '?' as u8 && slice[2] == ':' as u8 {
                (Token::Group(NativeRegexAST::from(&slice[3..close_index]),
                             repeater.repeater,
                             GroupType::NonCapturing), &slice[close_index+repeater.length+1..])
            } else {
                (Token::Group(NativeRegexAST::from(&slice[1..close_index]),
                             repeater.repeater,
                             GroupType::Capturing), &slice[close_index+repeater.length+1..])
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

    pub fn get_captures(& self) -> usize {
        let mut total = 0;

        for token in self.tokens.iter() {
            match token {
                Token::Group(ast, _, group) => match group {
                    GroupType::Capturing => {
                        total += ast.get_captures();
                    }
                    _ => {}
                }
                _ => {}
            }
        }

        total+1 //We add one because on match we capture the entire match
    }

    fn recur_tree(& self, level: usize) {

        let mut tabs = String::new();

        for _ in 0..level {
            tabs = format!("{}\t", tabs);
        }

        for token in self.tokens.iter() {
            match token {
                Token::Group(ast, repeater, group) => {
                    println!("{}Group {:?} {:?}\n", tabs, repeater, group);
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

