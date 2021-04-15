

pub type NativeRegexLocations = Vec<Option<(usize, usize)>>;

use std::collections::HashMap;
use std::ops::Range;
use crate::native_regex::NativeRegex;
use std::vec::IntoIter;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Captures<'t> {
    pub text: & 't str,
    pub locations: NativeRegexLocations,
    pub named_groups: HashMap<& 'static str, usize>
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Match<'t> {
    text: & 't str,
    start: usize,
    end: usize
}





#[derive(Copy, Clone)]
pub struct Matches<'t, 'r, R>
    where R: NativeRegex {
    pub capture_match: CaptureMatches<'t, 'r, R>
}

#[derive(Copy, Clone)]
pub struct CaptureMatches<'t, 'r, R>
    where R: NativeRegex {
    pub regex: & 'r R,
    pub text: & 't str,
    pub last_end: usize,
    pub last_match: Option<usize>
}


impl<'t, 'r, R> Matches<'t, 'r, R>
    where R: NativeRegex {

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
                Some(capture.first())
            }
            None => None
        }
    }

}

impl<'t> Match<'t> {

    pub fn new(text: &'t str, start: usize, end: usize) -> Match<'t> {
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

    pub fn first(&self) -> Match<'t> {
        let (start, end) = self.locations.get(0).unwrap().unwrap();
        Match::new(self.text, start, end)
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

    pub fn expand(&self, mut replacement: &str, dst: &mut String) {
        use crate::regexes::CaptureNameRegex;

        let capture_reg = CaptureNameRegex::new();

        while !replacement.is_empty() {

            match capture_reg.captures(replacement) {
                Some(captures) => {

                    dst.push_str(&replacement[..captures.first().start]);


                    let identifier = match captures.get(1) {
                        Some(m) => {
                            //If the first group matches, we have an escaped dollar sign, $$.
                            replacement = &replacement[m.end..];
                            "$"
                        }
                        None => {
                            replacement = &replacement[captures.first().end..];
                            match captures.get(2) {
                                Some(m) => {
                                    match m.as_str().parse::<usize>() {
                                        Ok(number) => {
                                            self.get(number).map(|x| x.as_str()).unwrap_or("")
                                        }
                                        Err(_) => {
                                            self.name(m.as_str()).map(|x| x.as_str()).unwrap_or("")
                                        }
                                    }
                                }
                                None => {
                                    ""
                                }
                            }
                        }
                    };

                    dst.push_str(identifier);

                }
                None => {
                    dst.push_str(replacement);
                    break;
                }
            }

        }
    }

}