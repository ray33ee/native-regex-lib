
use Previous::{Start, Character};
use std::str::{CharIndices};

//An iterator-like object that advances over a string providing character information via CharacterInfo
#[derive(Clone)]
pub struct Advancer<'t> {
    iter: CharIndices<'t>,
    prev: Previous,
    start: usize,
    length: usize,
}

//An iterator that iterates over a string and returns an Advancer for each character
pub struct AdvancerIterator<'t> {
    text: & 't [u8],
    iter: CharIndices<'t>,
    prev: Previous,
    start: usize
}

//Enum representing the previous character or Start if at the beginning
#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Previous {
    //The first character of a string has no previous, so we label it Start
    Start,

    //Enum to contain the previous character
    Character(char)
}

//Contains information about a character, including the index, previous character, and the actual character itself
#[derive(Clone, Debug, Copy)]
pub struct CharacterInfo {
    index: usize,
    current: Option<char>,
    previous: Previous
}

impl Previous {
    #[inline(always)]
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
    #[inline(always)]
    fn new(index: usize, current: Option<char>, previous: Previous) -> Self {
        CharacterInfo {
            index,
            current,
            previous
        }
    }

    #[inline(always)]
    pub fn index(&self) -> usize { self.index }

    #[inline(always)]
    pub fn current(&self) -> Option<char> { self.current }

    #[inline(always)]
    pub fn previous(&self) -> Previous { self.previous }
}

impl<'t> Advancer<'t> {

    pub fn prev(&self) -> Previous { self.prev.clone() }

    #[inline(always)]
    pub fn advance(& mut self) -> CharacterInfo {

        let prev = self.prev.clone();

        match self.iter.next() {
            Some((index, character)) => {

                self.prev = Character(character);

                CharacterInfo::new(index + self.start, Some(character), prev)

            }
            None => {

                CharacterInfo::new(self.length, None, prev)
            }
        }
    }

}

impl<'t> AdvancerIterator<'t> {

    #[inline(always)]
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
            AdvancerIterator {
                text: bytes,
                iter: std::str::from_utf8_unchecked(&bytes[start..]).char_indices(),
                prev,
                start
            }
        }
    }

}

impl<'t> Iterator for AdvancerIterator<'t> {
    type Item = Advancer<'t>;

    #[inline(always)]
    fn next(& mut self) -> Option<Self::Item> {

        let iterator = self.iter.clone();

        let prev = self.prev.clone();

        let ch = self.iter.next()?.1;

        self.prev = Character(ch);

        Some(Advancer {
            iter: iterator,
            prev,
            start: self.start,
            length: self.text.len()
        })
    }

}