
use std::fmt::Debug;
use std::ops::Range;
use std::collections::HashMap;

use std::vec::IntoIter;

type NativeRegexLocations = Vec<Option<(usize, usize)>>;
type NativeRegexReturn<'a> = Option<NativeRegexLocations>;
type NativeRegexSignature = fn(& str, usize) -> NativeRegexReturn;

pub struct NativeRegex {
    regex: NativeRegexSignature
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Match<'t> {
    text: & 't str,
    start: usize,
    end: usize
}

#[derive(Copy, Clone)]
pub struct Matches<'t> {
    capture_match: CaptureMatches<'t>
}

#[derive(Copy, Clone)]
pub struct CaptureMatches<'t> {
    regex: NativeRegexSignature,
    text: & 't str,
    last_end: usize,
    last_match: Option<usize>
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Captures<'t> {
    text: & 't str,
    locations: NativeRegexLocations,
    named_groups: HashMap<String, usize>
}

#[derive(Copy, Clone)]
pub struct Split<'t> {
    finder: Matches<'t>,
    last: usize
}

impl<'t> Match<'t> {

    fn new(text: &'t str, start: usize, end: usize) -> Match<'t> {
        Match { text, start, end }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }

    pub fn as_str(&self) -> &'t str {
        &self.text[self.range()]
    }

}

impl<'t> From<Match<'t>> for &'t str {
    fn from(m: Match<'t>) -> &'t str {
        m.as_str()
    }
}

impl<'t> From<Match<'t>> for Range<usize> {
    fn from(m: Match<'t>) -> Range<usize> {
        m.range()
    }
}

impl NativeRegex {

    pub fn new(regex: NativeRegexSignature) -> Self {
        NativeRegex {
            regex
        }
    }

    pub fn is_match(&self, text: &str) -> bool {
        (self.regex)(text, 0).is_some()
    }

    pub fn find<'t>(&self, text: & 't str) -> Option<Match<'t>> {
        match (self.regex)(text, 0) {
            Some(matches) => {
                let (start, end) = matches.get(0).unwrap().unwrap();
                Some(Match::new (
                    text, start, end
                ))
            }
            None => None
        }
    }

    pub fn find_iter<'t>(&self, text: & 't str) -> Matches<'t> {
        Matches {
            capture_match: self.captures_iter(text)
        }
    }

    pub fn captures<'t>(&self, text: & 't str) -> Option<Captures<'t>> {
        match (self.regex)(text, 0) {
            Some(captures) => {
                Some(Captures {
                    text,
                    locations: captures,
                    named_groups: HashMap::new()
                })
            }
            None => None
        }
    }

    pub fn captures_iter<'t>(&self, text: & 't str) -> CaptureMatches<'t> {
        CaptureMatches {
            regex: self.regex,
            text,
            last_end: 0,
            last_match: None
        }
    }

    pub fn split<'t>(&self, text: & 't str) -> Split<'t> {
        Split { finder: self.find_iter(text), last: 0 }
    }

}

impl<'t> Matches<'t> {

    pub fn text(&self) -> & 't str {
        self.capture_match.text
    }

}

impl<'t> CaptureMatches<'t> {

    pub fn text(&self) -> & 't str {
        self.text
    }

}

impl<'t> Iterator for CaptureMatches<'t> {

    type Item = Captures<'t>;

    fn next(&mut self) -> Option<Captures<'t>> {
        if self.last_end > self.text.len() {
            return None;
        }
        let locations = match (self.regex)(self.text, self.last_end) {
            None => return None,
            Some(m) => m
        };

        let (start, end) = locations.get(0).unwrap().unwrap();

        if start == end {
            self.last_end = end+1;

            if self.last_match == Some(end) {
                return self.next()
            }

        } else {
            self.last_end = end;
        }

        self.last_match = Some(end);
        Some(Captures {
            text: self.text,
            locations,
            named_groups: HashMap::new()

        })

    }

}


impl<'t> Iterator for Matches<'t> {

    type Item = Match<'t>;

    fn next(&mut self) -> Option<Match<'t>> {
        match self.capture_match.next() {
            Some(capture) => {
                Some(capture.get(0).unwrap())
            }
            None => None
        }
    }

}

impl<'t> Iterator for Split<'t> {

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
                let matched = &text[self.last..m.start];
                self.last = m.end;
                Some(matched)
            }
        }
    }

}

impl<'t> Captures<'t> {

    pub fn get(&self, i: usize) -> Option<Match<'t>> {
        match self.locations.get(i) {
            Some(m) => match m {
                Some((start, end)) => Some(Match::new(self.text, *start, *end)),
                None => None
            }
            None => None
        }
    }

    pub fn name(&self, name: &str) -> Option<Match<'t>> {
        match self.named_groups.get(name) {
            Some(index) => self.get(*index),
            None => None
        }
    }

    pub fn iter(& self) -> IntoIter<Option<Match>> {
        let thing: Vec<_> = self.locations.iter().map(|m| {
            match m {
                Some((start, end)) => Some(Match::new(self.text, *start, *end)),
                None => None,
            }
        }).collect();

        thing.into_iter()
    }

    pub fn len(&self) -> usize {
        self.locations.len()
    }

}