
use crate::native_regex::NativeRegex;

pub struct CaptureNameRegex {
    named_groups: std::collections::HashMap<& 'static str, usize>
}

impl CaptureNameRegex {
    pub fn new() -> Self {
        let named_groups = std::collections::HashMap::new();



        CaptureNameRegex {
            named_groups
        }
    }
}

impl Into<crate::native_regex::Engine> for CaptureNameRegex {

    fn into(self) -> crate::native_regex::Engine {
        self.engine()
    }

}

impl crate::native_regex::NativeRegex for CaptureNameRegex {

    // Function to match regex '\$(\$)?(?:\{([^{}]*)\})?'
    #[allow(unused_parens, unused_comparisons)]
    fn step(mut chars: crate::native_regex::character::Advancer, captures: & mut crate::vectormap::VectorMap<(usize, usize)>) -> Option<()> {


        //Advance to first character & bounds check
        let mut character = chars.advance();

        if character.current().is_none() { return None; }

        //Zero capture
        let capture_0_first = character.index();

        if character.current().is_none() { return None; }

        if (character.current().unwrap() as u32) != 36 { return None; }

        character = chars.advance();

        {
            let mut match_count = 0;

            while character.current().is_some() {
                {

                    let capture_1_start = character.index();

                    if character.current().is_none() { return None; }

                    if (character.current().unwrap() as u32) != 36 { break; }

                    character = chars.advance();

                    captures.insert(1, (capture_1_start, character.index()));

                }



                match_count += 1;

                if match_count == 1 {
                    break;
                }
            }

            if match_count < 0 {
                return None;
            }
        }

        {
            let mut match_count = 0;

            while character.current().is_some() {
                {

                    if character.current().is_none() { return None; }

                    if (character.current().unwrap() as u32) != 123 { break; }

                    character = chars.advance();

                    {

                        let capture_2_start = character.index();

                        {
                            let mut match_count = 0;

                            while character.current().is_some() {
                                if character.current().is_none() { return None; }

                                if ((character.current().unwrap() as u32) < 0 || (character.current().unwrap() as u32) > 122) && (character.current().unwrap() as u32) != 124 && ((character.current().unwrap() as u32) < 126 || (character.current().unwrap() as u32) > 1114111) {
                                    break;
                                }

                                character = chars.advance();



                                match_count += 1;
                            }

                            if match_count < 0 {
                                break;
                            }
                        }

                        captures.insert(2, (capture_2_start, character.index()));

                    }

                    if character.current().is_none() { return None; }

                    if (character.current().unwrap() as u32) != 125 { break; }

                    character = chars.advance();

                }



                match_count += 1;

                if match_count == 1 {
                    break;
                }
            }

            if match_count < 0 {
                return None;
            }
        }



        captures.insert(0, (capture_0_first, character.index()));

        return Some(())
    }

    fn capture_names(&self) -> &std::collections::HashMap<& 'static str, usize> {
        &self.named_groups
    }

    fn capture_count(&self) -> usize { 3 }

}
