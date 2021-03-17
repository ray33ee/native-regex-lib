# Issue

The following outline does not support backtracking. This will not be supported for performance reasons.

# Explanation

This document outlines how to take an abstract syntax tree (as outlined in parse.rs `NativeRegexAST`).
This algorithm maps a token (as outlined in parse.rs `Token`) to Rust source code. Modifiers (such as
repeaters, as defined in parse.rs `RepeaterType`) will also mofify the mapped source code.

```rust
enum Token {
    CharacterClass(CharacterSet, RepeaterType), //Can be a traditional character class, [] or a shorthand character class
    Anchor(AnchorType),
    LiteralSingle(u8, RepeaterType),
    LiteralList(Vec<u8>),
    Group(NativeRegexAST, RepeaterType, GroupType),
    Alternation,
}
```

## Base

All regexes will have this base outer code

```rust
pub fn function_name(str_text: &str) -> Option<[Option<(usize, & str)>; N]> {
    let text = str_text.as_bytes();

    let mut index = 0;

    let mut captures: [Option<(usize, & str)>; N] = [None; N];

    'main: while index < text.len() {

        //Start counter
        let mut counter = 0;

        let capture_0_start = index + counter;

        ...

        captures[0] = Some((index + counter, &str_text[capture_0_start..index+counter]));

        return Some(captures);
    }


    None
}
```

where `N` is the number of captures in the regex, and `function_name` is the name supplied with the regex.
The ellipses represents the rest of the code. 

Generally each token will test for a particular condition. If this test fails within the main loop we increment to the next `index` and continue the search.
If a test fails within an inner loop (such as a capture group)

The return value above uses arrays to return the result. For N>32 this approach is not suitable, since arrays can be at most
32 elements large. Larger than 32 elements we simply use `Vec` and initialise it using the `vec!` macro to take
advantage of its optimisations. This means that the return value becomes

```rust
Option<Vec<Option<(usize, & str)>>>
```

### Break

As previously mentioned, if an attempted match is unsuccessful, control flow increments the `index` and continues.
However, within repeating captures this is not desirable, so we imply break out of the capture. These two cases are expressed as follows

```rust
index += 1;
continue;
```

and for within captures

````rust
break;
````

For the sake of brevity, we will simply use the abbreviation `NO_MATCH` to indicate that no match has been found,
and that control flow will stop.

### Bounds check

Before looking ahead we must ensure that we do not look beyond the slice. Do do this we implement the following bounds check
bebefore most operations

```rust
if index + counter + (N-1) > text.len() { //Bounds check. If this fails, there cannot possibly be a match at `index` so continue
    NO_MATCH
}
```

where `N` is the number of characters we are looking at in one go.
For example, when matching single literal characters, N=1. 
When matching `n` literal characters in a row, N=n.

Once again for brevity, when using this code we simply use the abbreviation `BOUNDS_CHECK`.

## Tokens

### Character class

The character class token can come from a shorthand or an explicit character class. 
Either way a character class is just a list of character ranges. 

```rust
BOUNDS_CHECK

if ... {
    index += 1; continue 'main;
}

counter += 1;
```

Where `...` is an expression that evaluates to false if `text[index+counter]` is not in the character set.

### Capture group

The Nth capture group will have the following template

```rust
{
    let capture_N_start = index + counter;
    
    ...
    
    captures[N] = Some((index + counter, &str_text[capture_zero_start..index + counter]));
}
```

Note: For non-capturing groups, we simply omit the two lines that get the `capture_N_start` and that set `captures[N]`.
The parenthesis are not required, but they are useful to limit the scope of the variables declared within, and help with surrounding `for` loops.

### Literal Single

```rust
BOUNDS_CHECK

if text[index+counter] == LITERAL_CHARACTER {
    NO_MATCH
}

counter += 1;
```

where `LITERAL_CHARACTER` is the character to match

### Literal List

```rust
    BOUNDS_CHECK
    
    if 
        text[index+counter] == LITERAL_CHARACTER_0 && 
        text[index+counter+1] == LITERAL_CHARACTER_1 &&
        ...
        text[index+counter+(N-1)] == LITERAL_CHARACTER_(N-1)
    {
        NO_MATCH
    }
    
    counter += N;
```

where N is the number of characters and `LITERAL_CHARACTER_n` is the nth literal in the list.

### Anchors 

#### Start

```rust
if index != 0 {
    return None;
}
```

If we're not at the start of the string, there is no way we have a match.
This template must be at the start of the main loop.

#### End

```rust
if index != text.len() - 1 {
    index += 1;
    continue;
}
```

If we're not at the end of the string, go to next index. This snippet must be at the end of the main loop.

#### Word



## Modifiers

# Repeaters

## Exactly Once

The template is unchanged

## ZeroOrOne

```rust
{
    let mut match_count = 0;
    
    for _ in &text[index + counter..] {
        ...
        
        match_count += 1;
        
        if match_count == 1 {
            break;
        }
    }
    
}
```

## ZeroOrMore

```rust
for _ in &text[index + counter..] {
    ...
}
```

## OnceOrMore

```rust
{
    let mut found = false;

    for _ in &text[index + counter..] {
        ...
        found = true;
    }

    if !found {
        NO_MATCH
    }
}
```

## ExactlyN

```rust
{
    let mut match_count = 0;
    
    for _ in &text[index + counter..] {
        ...        
        
        match_count += 1;
        
        if match_count == N {
            break;
        }
    }
    
    if match_count < N {
        NO_MATCH
    }
}
```

## RangeNM

```rust
{
    let mut match_count = 0;

    for _ in &text[index + counter..] {
        ...

        match_count += 1;

        if match_count == M {
            break;
        }
    }

    if match_count < N {
        NO_MATCH
    }
}
```

## RangeN

```rust
{
    let mut match_count = 0;

    for _ in &text[index + counter..] {
        ...

        match_count += 1;
    }

    if match_count < N {
        NO_MATCH
    }
}
```
