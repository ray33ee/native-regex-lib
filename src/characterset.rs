use std::ops::RangeInclusive;

#[derive(Debug)]
pub enum CharacterRange {
    Single(u8),
    Multiple(RangeInclusive<u8>)
}

#[derive(Debug)]
pub struct SubSet {
    ranges: Vec<CharacterRange>,
    inverted: bool
}

#[derive(Debug)]
pub struct CharacterSet {
    subsets: Vec<SubSet>,
    inverted: bool
}

impl CharacterRange {

    pub fn code(&self, invert: bool, char_identifier: & str) -> String {
        match self {
            Self::Single(ch) => {
                if invert {
                    format!("{} != {}", char_identifier, *ch)
                } else {
                    format!("{} == {}", char_identifier, *ch)
                }
            }
            Self::Multiple(range ) => {
                if invert {
                    format!("({} < {} || {} > {})", char_identifier, range.start(), char_identifier, range.end())
                } else {
                    format!("({} >= {} && {} <= {})", char_identifier, range.start(), char_identifier, range.end())
                }
            }
        }
    }

}

impl SubSet {

    pub fn new(inverted: bool, ranges: Vec<CharacterRange>) -> Self {
        SubSet {
            inverted,
            ranges
        }
    }

    pub fn code(&self, invert: bool, char_identifier: & str) -> String {
        //We assume that the user will never call SubSet::code if self.ranges is empty
        /*
        if self.subsets.is_empty() {
            //Error
        }
        */
        let invert = invert ^ self.inverted;

        let mut res = self.ranges.get(0).unwrap().code(invert, char_identifier);

        for i in 1..self.ranges.len() {
            res.push_str(&format!(" {} {}", if invert {"&&"} else {"||"}, self.ranges.get(i).unwrap().code(invert, char_identifier)));
        }

        res
    }

}

impl CharacterSet {

    pub fn new() -> Self {
        CharacterSet {
            subsets: Vec::new(),
            inverted: false
        }
    }

    pub fn invert(& mut self, invert: bool) {
        self.inverted = invert;
    }

    pub fn add_subsets(& mut self, subset: SubSet) {
        self.subsets.push(subset);
    }

    pub fn add_subset(& mut self, invert: bool, range: CharacterRange) {
        self.subsets.push(SubSet::new(invert, vec![range]))
    }

    pub fn code(&self, char_identifier: & str) -> String {

        //We assume that the user will never call CharacterSet::code if self.subsets is empty
        /*
        if self.subsets.is_empty() {
            //Error
        }
        */

        let mut res = if self.subsets.len() == 1 {
            format!("{}", self.subsets.get(0).unwrap().code(self.inverted, char_identifier))
        } else {
            format!("({})", self.subsets.get(0).unwrap().code(self.inverted, char_identifier))
        };

        for i in 1..self.subsets.len() {
            res.push_str(&format!(" {} ({})", if self.inverted {"&&"} else {"||"}, self.subsets.get(i).unwrap().code(self.inverted, char_identifier)));
        }

        res
    }

}

impl From<& [u8]> for CharacterSet {
    fn from(mut slice: & [u8]) -> Self {

        let mut set = CharacterSet::new();

        //Look for a '^' and chop it off
        set.inverted = if slice[0] == '^' as u8 {
            slice = &slice[1..];
            false
        } else {
            true
        };

        //Similarly if we have leading or trailing '-' add them and chop the string
        if slice[0] == '-' as u8 {
            slice = &slice[1..];
            //set.push((false, RangeInclusive::new('-' as u8, '-' as u8)));

            set.add_subset(false, CharacterRange::Single('-' as u8));
        }

        if slice[slice.len() - 1] == '-' as u8 {
            slice = &slice[..slice.len()-1];
            //set.push((false, RangeInclusive::new('-' as u8, '-' as u8)));

            set.add_subset(false, CharacterRange::Single('-' as u8));
        }

        let mut index = 0;

        while index < slice.len() {

            let mut counter = 0;

            //Get current token
            let ch = if slice[index] == '\\' as u8 {

                if slice[index+1] == 'd' as u8 {
                    //set.push((false, RangeInclusive::new('0' as u8, '9' as u8)));

                    set.add_subset(false, CharacterRange::Multiple(RangeInclusive::new('0' as u8, '9' as u8)));
                    index += 2;
                    continue;
                } else if slice[index+1] == 'D' as u8 {
                    //set.push((false, RangeInclusive::new(0, '0' as u8 - 1)));
                    //set.push((false, RangeInclusive::new('9' as u8 + 1, 255)));

                    set.add_subset(true, CharacterRange::Multiple(RangeInclusive::new('0' as u8, '9' as u8)));
                    index += 2;
                    continue;
                } else if slice[index+1] == 's' as u8 {



                    /*set.push((false, RangeInclusive::new(' ' as u8, ' ' as u8)));
                    set.push((false, RangeInclusive::new('\t' as u8, '\t' as u8)));
                    set.push((false, RangeInclusive::new('\n' as u8, '\n' as u8)));
                    set.push((false, RangeInclusive::new('\r' as u8, '\r' as u8)));
                    set.push((false, RangeInclusive::new(12, 12)));*/ //Form feed

                    set.add_subsets(SubSet::new(false, vec![
                        CharacterRange::Single(' ' as u8),
                        CharacterRange::Single('\t' as u8),
                        CharacterRange::Single('\n' as u8),
                        CharacterRange::Single('\r' as u8),
                        CharacterRange::Single(12)
                    ]));

                    index += 2;
                    continue;
                } else if slice[index+1] == 'S' as u8 {
                    /*set.push((false, RangeInclusive::new(0, 8)));
                    set.push((false, RangeInclusive::new(11, 11)));
                    set.push((false, RangeInclusive::new(14, 31)));
                    set.push((false, RangeInclusive::new(33, 255)));*/

                    set.add_subsets(SubSet::new(true, vec![
                        CharacterRange::Single(' ' as u8),
                        CharacterRange::Single('\t' as u8),
                        CharacterRange::Single('\n' as u8),
                        CharacterRange::Single('\r' as u8),
                        CharacterRange::Single(12)
                    ]));
                    index += 2;
                    continue;
                } else if slice[index+1] == 'w' as u8 {
                    /*set.push((false, RangeInclusive::new('0' as u8, '9' as u8)));
                    set.push((false, RangeInclusive::new('A' as u8, 'Z' as u8)));
                    set.push((false, RangeInclusive::new('_' as u8, '_' as u8)));
                    set.push((false, RangeInclusive::new('a' as u8, 'z' as u8)));*/

                    set.add_subsets(SubSet::new(false, vec![
                        CharacterRange::Multiple(RangeInclusive::new('0' as u8, '9' as u8)),
                        CharacterRange::Multiple(RangeInclusive::new('a' as u8, 'z' as u8)),
                        CharacterRange::Multiple(RangeInclusive::new('A' as u8, 'Z' as u8)),
                        CharacterRange::Single('_' as u8)
                    ]));

                    index += 2;
                    continue;
                } else if slice[index+1] == 'W' as u8 {
                    /*set.push((false, RangeInclusive::new(0, 47)));
                    set.push((false, RangeInclusive::new(58, 64)));
                    set.push((false, RangeInclusive::new(91, 94)));
                    set.push((false, RangeInclusive::new(96, 96)));
                    set.push((false, RangeInclusive::new(123, 255)));*/

                    set.add_subsets(SubSet::new(true, vec![
                        CharacterRange::Multiple(RangeInclusive::new('0' as u8, '9' as u8)),
                        CharacterRange::Multiple(RangeInclusive::new('a' as u8, 'z' as u8)),
                        CharacterRange::Multiple(RangeInclusive::new('A' as u8, 'Z' as u8)),
                        CharacterRange::Single('_' as u8)
                    ]));
                    index += 2;
                    continue;
                } else if slice[index+1] == 't' as u8 {
                    //set.push((false, RangeInclusive::new('\t' as u8, '\t' as u8)));

                    set.add_subset(false, CharacterRange::Single('\t' as u8));
                    index += 2;
                    continue;
                } else if slice[index+1] == 'n' as u8 {
                    //set.push((false, RangeInclusive::new('\n' as u8, '\n' as u8)));

                    set.add_subset(false, CharacterRange::Single('\n' as u8));
                    index += 2;
                    continue;
                } else if slice[index+1] == 'r' as u8 {
                    //set.push((false, RangeInclusive::new('\r' as u8, '\r' as u8)));

                    set.add_subset(false, CharacterRange::Single('\r' as u8));
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
                    //set.push((false, RangeInclusive::new(ch as u8, slice[index + counter + 1] as u8)));

                    set.add_subset(false, CharacterRange::Multiple(RangeInclusive::new(ch as u8, slice[index + counter + 1] as u8)));
                    index += counter + 2;
                } else {
                    //set.push((false, RangeInclusive::new(ch, ch)));

                    set.add_subset(false, CharacterRange::Single(ch));
                    index += counter;
                }
            } else {
                //set.push((false, RangeInclusive::new(ch, ch)));

                set.add_subset(false, CharacterRange::Single(ch));
                index += counter;
            }

        }

        set
    }
}


impl std::fmt::Display for CharacterSet {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CharacterSet, {}", self.code("ch"))


    }

}