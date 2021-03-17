# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]
### To Do
- Make sure `CharacterSet::from` contains no overlapping ranges
- Add a bunch of helper functions that act as a wrapper around the Native Regex function and implement as close to the `regex` crate as possible, including
 - Matching
 - Capturing
 - Regex sets
- Add a validator for the function name to ensure that the name complies with the Rust naming system
- Support named captures (named captures are represented as maps from strings (the captures name) to integers (the captures index))

### Unfinished Ideas
- Try to identify regexes that use backtracking, and warn user that backtracking is not supported 

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
