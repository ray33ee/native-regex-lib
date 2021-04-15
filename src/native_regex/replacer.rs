
use std::borrow::Cow;
use crate::native_regex::captures::Captures;

pub trait Replacer {

    fn replace_append(&mut self, caps: &Captures, dst: &mut String);

}

impl<F, T> Replacer for F
    where
        F: FnMut(&Captures) -> T,
        T: AsRef<str>,
{
    fn replace_append(&mut self, caps: &Captures, dst: &mut String) {
        dst.push_str((*self)(caps).as_ref());
    }
}

impl<'a> Replacer for &'a str {
    fn replace_append(&mut self, caps: &Captures, dst: &mut String) {
        caps.expand(*self, dst);
    }

}

impl<'a> Replacer for &'a String {
    fn replace_append(&mut self, caps: &Captures, dst: &mut String) {
        self.as_str().replace_append(caps, dst)
    }
}

impl Replacer for String {
    fn replace_append(&mut self, caps: &Captures, dst: &mut String) {
        self.as_str().replace_append(caps, dst)
    }
}

impl<'a> Replacer for Cow<'a, str> {
    fn replace_append(&mut self, caps: &Captures, dst: &mut String) {
        self.as_ref().replace_append(caps, dst)
    }
}

#[derive(Clone, Debug)]
pub struct NoExpand<'t>(pub &'t str);

impl<'t> NoExpand<'t> {
    pub fn new(replacement: & 't str) -> Self { NoExpand ( replacement ) }
}

impl<'t> Replacer for NoExpand<'t> {
    fn replace_append(&mut self, _: &Captures, dst: &mut String) {
        dst.push_str(self.0);
    }
}

