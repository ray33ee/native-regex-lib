# Native Regex

A library of tools to allow the compiling of Regex engines into programming language source code.

# Why?

Regexes are incredibly useful, without them string handling is diffucult at best, and at worst error prone and frustrating.
They do however incur a large performance overhead, especially when compiling the regex and when matching. 
The regex compile state happens at runtime which can adversly affect program startup times. 
The Regex engines are also not as efficient as hard-coded solutions. 

This project aims to alleviate these issues by treating a Regex as source code itself, and compiling at compile time.
This is possible since the vast majority of Regexes are known at compile time.
This has the following benefits

- Bad regexes are spotted at compile time 
- No runtime overhead for compiling Regexes
- Faster regex matching and performance

# Limitations

For various reasons, some features of common regexes are not yet supported. 
Most unsupported features are so because they would add a high overhead. Usually however, these limitations have work arounds, they just require you to think a little differently.

# Precision

With a normal regex it is possible to validate IPv4 addresses using the regex 

```regexp
\b(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\.(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\b
```

since alternation is not supported, this regex is not possible with Native Regex. However the much simpler version IS supported

```regexp
([0-9]{1,3})\.([0-9]{1,3})\.([0-9]{1,3})\.([0-9]{1,3})
```

we can then use Rust to perform the bounds check. It is tempting to create these ultra precise regexes, but the reality is that it is much easier to create less precise regexes and perform the extra checks in the programming language.

## Backtracking

Perhaps the next biggest limitation is lack of backtracking. Take the regex 

```regexp
([0-9]*)([0-9])
```

This will find a match in the string "0472894739". This match will produce the three captures "0472894739", "047289473" and "9".
This regex will not match under the Native Regex function. This is because the first capture group greedily matches the entire string, and so the second group fails to match, resulting in the entire string failing.

This problem is circumvented by backtracking. This generally involves trying a less greedy match for the previous match.
If this fails we keep trying different permutations until it matches. If we keep back tracking and no permutations work, the match fails.

Backtracking is very expensive. Chaining just a few repetition quantifiers can massively increase the multiplicity, and therefore the number of combinations to test.
Backtracking, even in normal regexes should be avoided. Take the following regex used to match floating point numbers with optional scientific notation:

```regexp
[-+]?[0-9]*.?[0-9]+(?:[eE][-+]?[0-9]+)?
```

The backtracking mey not be obvious until you realise that the decimal point is optional, and when it is omitted we have

```regexp
[-+]?[0-9]*[0-9]+(?:[eE][-+]?[0-9]+)?
```

This next regex will also match scientific numbers without the need of backtracking

```regexp
[-+]?[0-9]+(.[0-9]+)?(?:[eE][-+]?[0-9]+)?
```

and is also usable with Native Regex. It seems that regexes can be often redesigned to avoid backtracking.
It is for this reason that backtracking will not be supported due to the increased overhead.

## Alternation

Alternation is not supported since it uses backtracking and can be achieved with multiple regexes.

An alternative to alternation is to use multiple regexes at once with `NativeRegexSet`

## Backreferences & Lookaround

Backreferences & look arounds are not yet supported. This is because the regex is based on the `regex` crate which does not support backreferences for performance reasons.
