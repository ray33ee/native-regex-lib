
use crate::native_regex::Captures;
use crate::native_regex::Engine;
use std::collections::{HashSet, HashMap};
use std::collections::hash_map::Iter;
use crate::native_regex::{AdvancerIterator};
use crate::vectormap::VectorMap;

#[derive(Debug)]
pub struct SetMatches<'t> {
    matches: HashMap<usize, Captures<'t>>
}



pub struct SetMatchesIterator<'a, 't> {
    it: Iter<'a, usize, Captures<'t>>
}

impl<'a,'t> Iterator for SetMatchesIterator<'a, 't> {
    type Item = (& 'a usize, & 'a Captures<'t>);

    fn next(& mut self) -> Option<Self::Item> {
        self.it.next()
    }

}

impl<'t> SetMatches<'t> {
    fn new() -> Self {
        SetMatches {
            matches: HashMap::new()
        }
    }

    pub fn iter(&self) -> SetMatchesIterator {
        SetMatchesIterator {
            it: self.matches.iter()
        }
    }
}

pub struct NativeRegexSet {
    engines: Vec<Engine>,
    max_capture_count: usize
}

impl NativeRegexSet {

    pub fn new<I>(regexes: I) -> Self
        where I: IntoIterator<Item = Engine> {

        let engines = regexes.into_iter().collect::<Vec<_>>();

        let max_capture_count = engines.iter().map(|engine| engine.capture_count ).max().unwrap();

        NativeRegexSet {
            engines,
            max_capture_count
        }
    }

    pub fn is_match(&self, text: & str) -> bool {


        let mut captures = VectorMap::new(self.max_capture_count);

        for it in AdvancerIterator::new(text, 0) {
            for engine in self.engines.iter() {
                if (engine.regex)(it.clone(), & mut captures).is_some() {
                    return true;
                }
                captures.clear();
            }
        }
        false
    }

    pub fn matches<'t>(&self, text: & 't str) -> SetMatches<'t> {

        //List of engines that have not yet matched
        let mut set_matches = SetMatches::new();

        let mut finished_set = HashSet::new();

        let mut captures = VectorMap::new(self.max_capture_count);

        for it in AdvancerIterator::new(text, 0) {

            if finished_set.len() == self.engines.len() {
                break;
            }

            for (engine_index, engine) in self.engines.iter().enumerate() {
                if !finished_set.contains(&engine_index) {

                    match (engine.regex)(it.clone(), & mut captures) {
                        Some(_) => {
                            finished_set.insert(engine_index); //Flag the engine for removal

                            let caps = Captures {
                                text,
                                named_groups: engine.named_groups.clone(),
                                locations: captures.clone(),
                                count: engine.capture_count,
                            };

                            set_matches.matches.insert(engine_index, caps);

                        }
                        None => {}
                    }
                    captures.clear();
                }
            }
        }

        set_matches
    }
}