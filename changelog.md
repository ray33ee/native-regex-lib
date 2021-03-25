# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]
### To Do
- Add a validator to `translate` function for the function name to ensure that the name complies with the Rust naming system
- Add a support for `regex::Replacer`-like trait
- Create a web app (via WebAssembly) to convert regexes to source
- Add examples
- Add support for
  - Anchors
  - Word boundaries
  - Empty regexes
  - Named captures
  - NativeRegexSet
- Update process.md  
- Turn code in rust_translate.rs into a trait

### Unfinished Ideas
- Try to identify regexes that use backtracking, and warn user that backtracking is not supported
- Create an IR between the HIR and the outputted source code. Using something like
  - For
  - If
  - Next - command to move to next character
  - Declare - command to declare a variable and initialise it
  - Insert - command to add to capture group
  
## [0.3.0] - 2021-03-23

### Changed
- Complete overhauls of rust_translate.rs to use regex-syntax crate
- Using the HIR class to parse regex
- Supporting unicode characters
- Using iterators to iterate over unicode characters


### Added
- `SetMatches` now returns the index of the matched regex and the `Captures` object representing all the capture groups

## [0.2.6] - 2021-03-22

### Added
- `NativeRegexSet` with similar functionality to `regex::RegexSet`. Like `regex::RegexSet`, n regexes will only iterate over the text once
- `Engine` struct to expose the inner core of regexes (for use in `NativeRegexSet`)
- `SetMatches` and `SetMatchesIterator` to convey information about which regexes match and at what position in the string
- `word_class` function now changed to static  
- Output struct now stores the named captures in the struct rather than calculating them on each capture_names call
- The output struct uses references to the named captures instead of cloning them around
- `NativeRegex::step` is now a static function so its function pointer can be easily passed to `Engine`
- `NativeRegex::engine` function added to facilitate the creation of `NativeRegexSet`
- Implemented `Into<Engine>` for output code to make a more idiomatic extraction of `Engine` from `NativeRegex` types

### Fixed
- Out of bounds bug fixed, bounds checks now work properly

## [0.2.5] - 2021-03-21

### Added
- `step` function to NativeRegex trait that allows user to step through text one character at a time, to pave the way for RegexSet.
- Renamed `counter` to `offset` in output code
- We no longer preallocate the captures vector to allow `regex_function` to become a provided method

## [0.2.4] - 2021-03-20

### Added
- Converted NativeRegex into a trait with two required functions
  - fn _match(& str, usize) -> Option<...>
  - fn _hash() -> HashMap<String, usize>
    And all the functions currently in NativeRegex become provided methods
- Finished code for named captures    

## [0.2.3] - 2021-03-18

### Added
- Infrastructure for named captures

### Fixed
- We only add the `word_class` closure if word boundaries are used
- Start and end anchors now work correctly

### Changed
- Better, more sophisticated `CharacterSet` objects

## [0.2.2] - 2021-03-17

### Added
- Add support for negated classes like \S, \W, etc.
- Added support for a `Fn(usize, & Captures) -> String` replacer 
- Added support for word boundaries (\b)

## [0.2.1] - 2021-03-17

### Added
- Support for escaped characters \n \t and \r in character classes

### Removed
- Tests module, as tests are now conducted with the [native-regex-tester](https://github.com/ray33ee/native-regex-tester) package

### Fixed
- Issue with passing 'nomatch' variable to LiteralSingle tokens

## [0.2.0] - 2021-03-16

### Added
- Ability to specify the start position in regex function

### Changed
- 'file' option is now not required. If it is omitted the source is sent to `stdout` via `println!`
- Output sent due to the verbosity option is sent to `stderr`
- Reverted to indexing captures (instead of hashing) since named captures can map to indices (which in turn map to the capture)
- parse.rs and rust_translate.rs moved into a library project along with regex crate-like helper objects

### Fixed
- Correct start and end indices fixed (for capture groups)

## [0.1.8] - 2021-03-15

### Added
- Support for command line arguments
- Save source to given file
- Option to show or hide verbose output

### Changed 
- Migrated to returning captures as a hashmap instead of vector (in preparation for named captures)
- Renamed translate.rs to rust_translate.rs to make room for future translators

## [0.1.7] - 2021-03-13

### Added
- Result return type to AST creation functions to handle invalid regexes
- Support for dot matching token in `NativeRegexAST`

### Fixed
- Removed parenthesis around negated character class

### Changed
- Characters are converted into literal u8 instead of characters converted to u8

## [0.1.6] - 2021-03-13

### Fixed 
- Bounds checks now always continue in main loop
- Counting captures `NativeRegexAST::get_captures` function excludes non-capturing groups
- Cleared various warnings

## [0.1.5] - 2021-03-12

### Added
- Support for repeaters
- NO_MATCH now works correctly
- Wrapper function to add the base code to translator 
- Correct handling of captures using `vec!`

### Fixed
- Inverter bug fixed on fixed character token translation
- Double brackets around Some
- Added one to `capture_index` since index 0 is saved for the first capture, the entire match

### Changed
- Moved examples to docs folder

### Removed
- Macros and tests

## [0.1.4] - 2021-03-11

### Added
- Support for literal characters
- Support for a list of literals
- Added a simple function `NativeRegexAST::tree` that displays the contents of the AST
- Added process.md
- translate.rs to convert AST into Rust source
- Figured out bounds checks (see process.md)
- Figured out return value of Rust functions (see process.md)


## [0.1.3] - 2021-03-11

### Added
- Add support for {N}, {N,} and {N,M} repetition
- Support for '^' in character classes
- Support for shorthand classes within character classes
- Support for shorthand classes outside of character classes and literal escaped characters

## [0.1.2] - 2021-03-09

### Added
- Structure and most of the code for AST including
  - Recursively walking the regex
  - Obtaining repetition suffixes
  - Parsing character sets (not shorthand sets yet)

## [0.1.1] - 2021-03-09

### Added
- Added MIT license
- Tests for macros
- Some basic hard-coded regexes

### Changed
- Macros moved to dedicated macro.rs module

### Fixed
- Logic and `SUBSTR_LEN` issues within `indexed_native_searcher` fixed.

## [0.1.0] - 2021-03-09

### Added
- Initial commit
- Very basic macro to search for literal strings (represented as a sequence of characters sent to the macro)
