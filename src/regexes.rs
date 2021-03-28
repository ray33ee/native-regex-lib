
use crate::native_regex::Engine;
use crate::native_regex::NativeRegex;


// Regex used to replace strings according to a particular format
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
    fn step(mut chars: crate::native_regex::CharOffsetIndices, length: usize) -> Option<Vec<Option<(usize, usize)>>> {

        let mut captures = vec![None; 3];

        //Advance to first character & bounds check
        let mut current = chars.next();

        if current.is_none() { return None; }

        //Zero capture
        let capture_0_first = current.unwrap().index();

        if current.is_none() { return None; }

        if (current.unwrap().current() as u32) != 36 { return None; }

        current = chars.next();

        {
            let mut match_count = 0;

            while current.is_some() {
                {

                    let capture_1_start = current.unwrap().index();

                    if current.is_none() { return None; }

                    if (current.unwrap().current() as u32) != 36 { break; }

                    current = chars.next();

                    captures[1] = Some((capture_1_start, if current.is_some() { current.unwrap().index() } else { length }));

                }



                match_count += 1;

                if match_count == 1 {
                    break;
                }
            }

        }

        {
            let mut match_count = 0;

            while current.is_some() {
                {

                    if current.is_none() { return None; }

                    if (current.unwrap().current() as u32) != 123 { break; }

                    current = chars.next();

                    {

                        let capture_2_start = current.unwrap().index();

                        {
                            while current.is_some() {
                                if current.is_none() { return None; }

                                if ((current.unwrap().current() as u32) < 0 || (current.unwrap().current() as u32) > 122) && (current.unwrap().current() as u32) != 124 && ((current.unwrap().current() as u32) < 126 || (current.unwrap().current() as u32) > 1114111) {
                                    break;
                                }

                                current = chars.next();


                            }

                        }

                        captures[2] = Some((capture_2_start, if current.is_some() { current.unwrap().index() } else { length }));

                    }

                    if current.is_none() { return None; }

                    if (current.unwrap().current() as u32) != 125 { break; }

                    current = chars.next();

                }



                match_count += 1;

                if match_count == 1 {
                    break;
                }
            }

        }



        captures[0] = Some((capture_0_first, if current.is_some() { current.unwrap().index() } else { length }));

        return Some(captures)
    }

    fn capture_names(&self) -> &std::collections::HashMap<& 'static str, usize> {
        &self.named_groups
    }


}
