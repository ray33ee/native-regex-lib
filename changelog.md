# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]
### To Do
- Add `regex::Replacer` to readme and mention the differences (enclosing braces {} are mandatory to improve performance)
- Create a web app (via WebAssembly) to convert regexes to source
- Add examples
  - Example using regset and use list slice matching too
- Add validator to rust compiler 
- `Replacer`: Allow closures and strings that reference the nth replacement 
- Make sure that a regex like 'it' matches 'ititi' and doesn't bail because of the final 'i'

### Unfinished Ideas
- Try to identify regexes that use backtracking, and warn user that backtracking is not supported
- Make an interpreter for Ehir

## [0.4.0] - 2021-04-17

### Added
- `character.rs` functions are now inline, massively improving performance
- Custom object `VectorMap` used to store the capture results efficiently

### Changed
- `character.rs` iterators now renamed to more meaningful names
- The main `character.rs` iterator no longer uses the unsafe `from_utf8_unchecked` function to get the next `Advancer`

### Fixed
- Fixed various changes

## [0.3.4] - 2021-04-15

### Added
- `Ehir::translate` to convert regex-syntax `Hir` into a pseudocode-like IR 
- rust_translate.rs now uses `Ehir` to create source code
- Character in `Ehir` stored as u32
- Capture group indices stored as u32
- Extra information on regex stored within `Ehir`
  - Original string regex
  - Hash map of capture names and indices
  - Total capture count

### Removed
- Compiler trait removed in favour of `Ehir`
- We work with references to `String` instead of moving `String`s around

## [0.3.3] - 2021-03-28

### Added
- Since all repeaters fall into two categories, bounded or unbounded, there are now only two repeater functions that handle all types
- `is_word_byte` and `is_word_byte` added to `NativeRegex` as wrappers around their `regex-syntax` counterparts
- Required methods added to compiler class for word boundaries 
- Support for anchors and word boundaries

### Changed
- native_regex.rs now split up into multiple rust files to aid readability
- `SetMatches` implemented as a hashmap not a vector
- Modified `CharOffsetIndices` away from the iterator pattern so we can iterate past the end of a string (such that the previous character past the end is the last character )

## [0.3.2] - 2021-03-26

### Added
- Added NativeRegex of our own to parse Replacer strings
- Support for `regex::Replacer`-like replacer object

### Fixed
- Start line and End line bugs fixed

## [0.3.1] - 2021-03-25

### Added
- `CharOffsetIndices` An iterator similar to `CharIndices` that adds an offset to the output
- `CharIterIterIndex` an iterator over a string that creates `CharOffsetIndices`
- `NativeRegexSet` now works with new system
- Compiler trait which attempts to facilitate the creation of other compilers
- `RustCompiler` which implements `Compiler`
- `name_identifier_validator` function to `Compiler` to allow users to make sure that function names are valid for their language
- `CharOffsetIndices` now gives the previous character, if there is one  
- `CharacterInfo` struct to contain information about the current character including
  - Index
  - Current character
  - Previous character

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
