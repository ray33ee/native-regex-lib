enum NoMatch {
    Stop,
    Break,
}

enum Range {
    Single(char),
    Multiple(char, char),
}

enum AnchorLocation {
    Start,
    End
}

enum AnchorType {
    Regular,
    Newline
}

enum Decision {
    CharacterSet(Vec<Range>), //Should be true if the character is NOT within the character set
    Literal(char), //Determine if the character is a particular literal
    LiteralString(String),
    CountEquals(usize), //
    CountLessThan(usize),
    Anchor(AnchorType, AnchorLocation),
    End, //Determine if we are at the end of the text
    Middle //Determine if we are NOT at the end of the text
}

enum CaptureType {
    Beginning,
    End
}

enum Modifier {
    Is, // A
    Not // !A
}

enum Token {
    If(Modifier, Decision, NoMatch),
    While(Decision, Vec<Token>), //Loop to check repetition
    StartCount, //Set the counter to zero. Used in repetition to check bounds
    IncrementCount, //Increment the counter every time a repetition matches
    Advance, //Advance to the next character
    CaptureTag(CaptureType, usize),  //Represents the start or end of a capture group
    Block(Vec<Token>),

    Empty,
}

struct Ehir {
    pub _tokens: Vec<Token>
}

//By default, a token list must look like
//If(End, Stop)
//CaptureTag(Beginning, 0)
//...
//CaptureTag(End, 0)
// Where ... are the extra tokens