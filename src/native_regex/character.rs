
use Previous::{Start, Character};
use std::str::{CharIndices};

#[derive(Clone)]
pub struct CharOffsetIndices<'t> {
    iter: CharIndices<'t>,
    prev: Previous,
    offset: usize,
    length: usize,
}

pub struct CharIterIterIndex<'t> {
    text: & 't [u8],
    iter: CharIndices<'t>,
    prev: Previous,
    start: usize
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Previous {
    //The first character of a string has no previous, so we label it Start
    Start,

    //Enum to contain the previous character
    Character(char)
}

#[derive(Clone, Debug, Copy)]
pub struct CharacterInfo {
    index: usize,
    current: Option<char>,
    previous: Previous
}

impl Previous {
    pub fn unwrap(&self) -> char {
        match self {
            Character(ch) => {
                ch.clone()
            }
            Start => {
                panic!("Failed to unwrap Previous.")
            }
        }
    }
}

impl CharacterInfo {
    fn new(index: usize, current: Option<char>, previous: Previous) -> Self {
        CharacterInfo {
            index,
            current,
            previous
        }
    }

    pub fn index(&self) -> usize { self.index }

    pub fn current(&self) -> Option<char> { self.current }

    pub fn previous(&self) -> Previous { self.previous }
}

impl<'t> CharOffsetIndices<'t> {

    pub fn advance(& mut self) -> CharacterInfo {

        let prev = self.prev.clone();

        match self.iter.next() {
            Some((index, character)) => {

                self.prev = Character(character);

                CharacterInfo::new(index + self.offset, Some(character), prev)

            }
            None => {

                CharacterInfo::new(self.length, None, prev)
            }
        }
    }

}

impl<'t> CharIterIterIndex<'t> {

    pub fn new(text: & 't str, start: usize) -> Self {

        let bytes = text.as_bytes();

        let prev = {
            let mut previous = Start;

            for (ind, ch) in text.char_indices() {
                if ind == start {
                    break;
                }
                previous = Character(ch);
            }
            previous
        };


        unsafe {
            CharIterIterIndex {
                text: bytes,
                iter: std::str::from_utf8_unchecked(&bytes[start..]).char_indices(),
                prev,
                start
            }
        }
    }

}

impl<'t> Iterator for CharIterIterIndex<'t> {
    type Item = CharOffsetIndices<'t>;

    fn next(& mut self) -> Option<Self::Item> {

        let prev = self.prev.clone();

        let (ind, ch) = self.iter.next()?;

        self.prev = Character(ch);

        unsafe {
            Some(CharOffsetIndices {
                iter: std::str::from_utf8_unchecked(&self.text[ind + self.start..]).char_indices(),
                prev,
                offset: ind + self.start,
                length: self.text.len()
            })
        }
    }

}