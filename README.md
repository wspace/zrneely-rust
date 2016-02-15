Whitespace
==========

Whitespace is a language invented in 2003 by two students at the University of St. Andrews
in Scotland. It is unique in that it has only three valid characters: space, tab, and
newline. All instructions are formed by different patterns of those characters.

The original site describing the language is down; however, it is preserved in the
wayback machine [here](https://web.archive.org/web/20150523181043/http://compsoc.dur.ac.uk/whitespace/index.php "Whitespace Specification").


Interestingly, Whitespace has been bootstrapped; there is a Whitespace interpreter written in
Whitespace.


The Compiler
============

This is a Just-In-Time compiler/interpreter for Whitespace. It interprets a program written
in Whitespace, parses it, and generates and runs x86-64 machine code which is equivalent to
the input program. Unfortunately we do violate the specification a little for simplicity's sake:
integers are not arbitrary-width, and are instead always 64 bits wide.


Motivation
==========

Because it is so simple, Whitespace is easy to parse. One reason I wrote this is to learn
the parser combinator library [nom](https://crates.io/crates/nom "Nom Library"). My biggest
other goal was to become more familar with x86-64 assembly programming and the Rust language,
which the compiler is written in.
