
use std::fmt::Debug;
use std::ops::Range;
use std::collections::HashMap;

use std::vec::IntoIter;

type NativeRegexLocations = Vec<Option<(usize, usize)>>;
type NativeRegexReturn<'a> = Option<NativeRegexLocations>;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Match<'t> {
    text: & 't str,
    start: usize,
    end: usize
}


#[derive(Copy, Clone)]
pub struct Matches<'t, 'r, R>
    where R: NativeRegex
{
    capture_match: CaptureMatches<'t, 'r, R>
}

#[derive(Copy, Clone)]
pub struct CaptureMatches<'t, 'r, R>
where R: NativeRegex
{
    regex: & 'r R,
    text: & 't str,
    last_end: usize,
    last_match: Option<usize>
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Captures<'t> {
    text: & 't str,
    locations: NativeRegexLocations,
    named_groups: HashMap<& 'static str, usize>
}

#[derive(Copy, Clone)]
pub struct Split<'t, 'r, R>
    where R: NativeRegex
{
    finder: Matches<'t, 'r, R>,
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

pub trait NativeRegex: Sized {

    fn step(&self, captures: & mut Vec<Option<(usize, usize)>>, text: & [u8], index: usize) -> Option<()>;

    fn capture_names(&self) -> HashMap<& 'static str, usize>;

    fn word_class(&self, ch: u8) -> bool {
        ch >= 48 && ch <= 57 || ch >= 65 && ch <= 90 || ch == 95 || ch >= 97 && ch <= 122
    }

    fn regex_function(&self, str_text: &str, start: usize) -> NativeRegexReturn {

        let text = str_text.as_bytes();

        let mut index = start;

        let mut captures = Vec::new();

        while index < text.len() {
            match self.step(& mut captures, text, index) {
                Some(_) => {return Some(captures);}
                None => {captures.clear(); index += 1;}
            }
        }


        None
    }

    fn is_match(&self, text: &str) -> bool {
        self.regex_function(text, 0).is_some()
    }

    fn find<'t>(&self, text: & 't str) -> Option<Match<'t>> {
        match self.regex_function(text, 0) {
            Some(matches) => {
                let (start, end) = matches.get(0).unwrap().unwrap();
                Some(Match::new (
                    text, start, end
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
                    locations: captures,
                    named_groups: self.capture_names().clone()
                })
            }
            None => None
        }
    }

    fn captures_iter<'t, 'r>(& 'r self, text: & 't str) -> CaptureMatches<'t, 'r, Self>
    {

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

    fn replace<F>(&self, text: &str, rep: F) -> String
    where
    F: Fn(usize, & Captures) -> String {
        let mut iter = self.captures_iter(text).enumerate().peekable();
        if iter.peek().is_none() {
            return String::from(text);
        }


        let mut new = String::with_capacity(text.len());
        let mut last_match = 0;

        for (i, capture) in iter {
            let m = capture.get(0).unwrap();
            new.push_str(&text[last_match..m.start]);
            new.push_str(rep(i, & capture).as_str());
            last_match = m.end;
        }
        new.push_str(&text[last_match..]);
        new


    }

}

impl<'t, 'r, R> Matches<'t, 'r, R>
    where R: NativeRegex
{

    pub fn text(&self) -> & 't str {
        self.capture_match.text
    }

}

impl<'t, 'r, R> CaptureMatches<'t, 'r, R>
    where R: NativeRegex {

    pub fn text(&self) -> & 't str {
        self.text
    }

}

impl<'t, 'r, R> Iterator for CaptureMatches<'t, 'r, R>
    where R: NativeRegex {

    type Item = Captures<'t>;

    fn next(&mut self) -> Option<Captures<'t>> {
        if self.last_end > self.text.len() {
            return None;
        }
        let locations = match self.regex.regex_function(self.text, self.last_end) {
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


impl<'t, 'r, R> Iterator for Matches<'t, 'r, R>
    where R: NativeRegex {

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