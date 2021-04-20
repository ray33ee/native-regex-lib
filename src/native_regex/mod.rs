pub mod captures;
pub mod character;
pub mod replacer;
pub mod native_regex_set;

use captures::{Captures, CaptureMatches, Match, Matches};
use replacer::Replacer;
use crate::native_regex::captures::NativeRegexLocations;
use character::{Advancer, AdvancerIterator};

use std::collections::HashMap;
use crate::vectormap::VectorMap;


pub type NativeRegexReturn<'a> = Option<NativeRegexLocations>;

#[derive(Copy, Clone)]
pub struct Split<'t, 'r, R>
    where R: NativeRegex {
    finder: Matches<'t, 'r, R>,
    last: usize
}

#[derive(Clone)]
pub struct Engine {
    regex: fn (chars: Advancer, captures: & mut VectorMap<(usize, usize)>) -> Option<()>,
    named_groups: HashMap<& 'static str, usize>,
    capture_count: usize,
}

impl Engine {
    pub fn capture_count(&self) -> usize { self.capture_count }
}

pub trait NativeRegex: Sized {

    fn step(chars: Advancer, captures: & mut VectorMap<(usize, usize)>) -> Option<()>;

    fn is_word_byte(byte: u8) -> bool {
        regex_syntax::is_word_byte(byte)
    }

    fn is_word_character(character: char) -> bool {
        regex_syntax::is_word_character(character)
    }

    fn capture_names(&self) -> &HashMap<& 'static str, usize>;

    fn capture_count(&self) -> usize;

    fn engine(&self) -> Engine {
        Engine {
            regex: Self::step,
            named_groups: self.capture_names().clone(),
            capture_count: self.capture_count(),
        }
    }

    #[inline(always)]
    fn regex_function(&self, str_text: &str, start: usize) -> NativeRegexReturn {

        let mut captures = VectorMap::new(self.capture_count());

        for it in AdvancerIterator::new(str_text, start) {

            match Self::step(it, & mut captures) {
                Some(_) => {
                    return Some(captures);
                }
                None => {}
            }

            captures.clear();
        }
        None
    }

    fn is_match(&self, text: &str) -> bool {
        self.regex_function(text, 0).is_some()
    }

    fn find<'t>(&self, text: & 't str) -> Option<Match<'t>> {
        match self.regex_function(text, 0) {
            Some(matches) => {
                let (start, end) = matches.get(0).unwrap();
                Some(Match::new (
                    text, *start, *end
                ))
            }
            None => None
        }
    }

    fn find_iter<'t, 'r>(& 'r self, text: & 't str) -> Matches<'t, 'r, Self>
    {
        Matches {
            capture_match: self.captures_iter(text)
        }
    }

    fn captures<'t>(&self, text: & 't str) -> Option<Captures<'t>> {
        match self.regex_function(text, 0) {
            Some(captures) => {
                Some(Captures {
                    text,
                    count: captures.len(),
                    locations: captures,
                    named_groups: self.capture_names().clone(),
                })
            }
            None => None
        }
    }

    fn captures_iter<'t, 'r>(& 'r self, text: & 't str) -> CaptureMatches<'t, 'r, Self> {

        CaptureMatches {
            regex: &self,
            text,
            last_end: 0,
            last_match: None
        }
    }

    fn split<'t, 'r>(& 'r self, text: & 't str) -> Split<'t, 'r, Self> {
        Split { finder: self.find_iter(text), last: 0 }
    }

    fn replace<R>(&self, text: &str, mut rep: R) -> String
    where R: Replacer {

        let mut iter = self.captures_iter(text).enumerate().peekable();
        if iter.peek().is_none() {
            return String::from(text);
        }

        let mut new = String::with_capacity(text.len());
        let mut last_match = 0;

        for (i, capture) in iter {
            let m = capture.first();
            new.push_str(&text[last_match..m.start()]);
            rep.replace_append(&capture, & mut new);
            //new.push_str(rep(i, & capture).as_str());
            last_match = m.end();
        }
        new.push_str(&text[last_match..]);
        new


    }

}


impl<'t, 'r, R> Iterator for Split<'t, 'r, R>
    where R: NativeRegex {

    type Item = & 't str;

    fn next(& mut self) -> Option<Self::Item> {
        let text = self.finder.capture_match.text;
        match self.finder.next() {
            None => {
                if self.last > text.len() {
                    None
                } else {
                    let s = &text[self.last..];
                    self.last = text.len() + 1;
                    Some(s)
                }
            }
            Some(m) => {
                let matched = &text[self.last..m.start()];
                self.last = m.end();
                Some(matched)
            }
        }
    }

}
