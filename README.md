# B-Minor compiler

This is an implementation of the B-Minor language (originally, see below) as
described in the 2020 book "Introduction to Compilers and Language Design",
by Douglas L. Thain.

The book doesn't provide a full grammar, on purpose, as this is an undergraduate
textbook and the author prefers that the student puts some effort on figuring
the grammar on themselves, so what I describe below will probably be different
from other grammars out there. Of course the student is meant to go and ask
question to the professor, but I've been out of college for quite some time (and
hopefully it won't be needed!)

The main purpose of this implementation is working on IR and optimization, as
approached by this book.

## Note on Grammar and Implementation Details

The grammar was initially reverse-engineered from the book, but later I learned
that the author teaches a CSE 40243 class at UND, and he's been gracious enough to
publish an overview of the language. He changes bits respect of what was published
in the book to keep it challenging for the students (e.g.,
[you can read the one for 2025](https://dthain.github.io/compilers-fa25/bminor)).

He's still vague at times and there are typos in the notes (e.g. in 2025 he includes
`while` among the keywords, but then he specifies there's no `while` in the language).
Among other things, the document includes a FAQ which clarifies certain questions. The
contents of `GRAMMAR.md` reflects my original understanding of the grammar, plus the
changes for the 2025 class. See the section "Additional Functionality" at the bottom.

### Divergence from the stated "standard"

Q2.2 in the 2025 FAQ regarding the `for` syntax reads:

> No, commas may only be used in print statements, function calls, function prototypes,
> and array expressions.

Meaning that this is illegal:

    for (i = 0, j = 0;; i++, j++)

By the time I read it I had implemented this already, and it stays.

## AI Usage (or Lack Thereof)

As mentioned above, the purpose of writing this compiler is purely educational,
and thus using AI for writing the code would be pointless. I'll be using it to
along with `cargo tarpaulin` to check for test coverage and to suggest and
generate test cases (which can be rather tedious) ahead of writing the code itself.

## Language Features

B-Minor is designed to have object-code compatible with ordinary C, which makes
it easy to take advantage of the C standard library (up to a point).

The language itself is not totally C-like though, and the author points out how
the type syntax is closer to Pascal or SQL. Interestingly enough, it includes a
`print` statement.

Unlike in C, strings in B-Minor are not simply an array of char. They're a
specific type and immutable. Similarly to Pascal, strings are limited to 256 characters.

> [!IMPORTANT]
> Identifiers are limited to 256 characters in length!

The language allows arrays of fixed size.

> [!IMPORTANT]
> If an array is declared with no value, all its elements will be of the "nil" type (e.g.
> zeros if an array of integers.

B-Minor is _strictly typed_. No implied coercion is defined. One can only assign values
to a variable (or function parameter) if the types match exactly. Arithmetic operators
can only be applied to ingegers. Logical operations can be performed only on booleans.
Comparisons can be performed over any kind of arguments, but only if their types match.

> [!WARNING]
> This also means that types different to a boolean **do not** decay into booleans in
> contexts where a boolean is expected (like an `if` condition.)

### Operators and precedence

Most of the operators are a subset of those from C and they have the same precedence:

    () [] f()              grouping, array subscript, function call
    ++ --                  postfix increment/decrement
    #                      unary array length
    - !                    unary negation, logical not
    ^                      exponentiation
    * / %                  multiplication, division, modulus
    + -                    addition, subtraction
    < <= > >= == !=        comparison
    && ||                  logical and, logical or
    =                      assignment

### Declarations and statements

You may declare global variables, function prototypes and function definitions. You can
declare local variables within functions, with the same scoping rules as C. Function
definitions cannot be nested.

There are no `switch`, `while`, or `do`-`while` in B-Minor, as they all can be seen as
special cases of `if` and `for`.

> [!NOTE]
> I'll probably add them at some point, though. Syntax sugar is useful for
> expressiveness.

The `print` statement looks a little bit odd in the language, because it takes a list
of expressions to be printed out... and they can be of different types. But note that
this is not at odds with what is written above, because _it's not a function_.

> [!IMPORTANT]
> The `print` statement does not automatically issue a line feed as the last character.
> The programmer needs to provide it (if needed).

### Functions

A functions' return type can only be one of the four scalar types, plus `void` to indicate
that no return value is provided. As usual, the return value(s) must match exactly the
function's declared one.

Parameters can be of any type, though, with differing passing strategies:

* `integer`, `boolean`, and `char` are passed by value.
* `string` and `array` are passed by reference.

> [!IMPORTANT]
> As in C, arrays passed by reference have an indeterminate size. Length is typically
> passed as an extra argument.

### Main function

As with C, a complete program must have a `main` function that returns an integer. The
function might have no arguments _or_ be declared with `argc` and `argv` arguments, in
the same way you'd do in C. Here are the models:

    main: function integer () = {
        // code
    }
    
    main: function integer (argc: integer, argv: array [] string) = {
        // code
    }

## Additional Functionality

These details will be included in the language in the order specified:

- [ ] Integers may be represented as decimal, hexidecimal or binary.
- [ ] Strings and characters have a number of additional escape codes.
- [ ] Double precision floating point values, types, and operators have been added.
- [ ] auto indicates a variable type to be inferred from context.
- [ ] Arrays have an intrinsic length that is bounds-checked at runtime and read by the # operator.
- [ ] A special `carray` type provides simple C-style arrays without boundary checking,
  for compatibility.

## Challenges

The book estimates that an undergraduate class should keep busy for a whole semester
building this compiler, but encourages extending the language and suggests the following
challenges (which I intend to implement):

- [ ] Add a new native type `complex`, along with the needed operators so that one can
  do useful things with them (construct numbers, operate, extract real and imaginary
  parts)
- [ ] Add a new automatic type `var`. This should work like C++'s `auto`: the language
  will infer the type of the variable. Consider extending this to function parameters,
  but note that this needs to be carefully considered as it might have some ramifications
  (e.g., name mangling).
- [ ] Improve the access to arrays by making the array accesses automatically checked at
  runtime against the known size of the array.
- [ ] Add a new mutable string type `mutstring` with fixed size but capable of having its
  contents modified, and that can be converted to and from a regular `string` as needed.
- [ ] Add an alternative control flow structure like `switch`. For an extra challenge, allow
  `switch` to select value ranges, not just constants.
- [ ] Implement structure types.
