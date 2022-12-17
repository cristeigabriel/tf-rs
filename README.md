# tf-rs
A Team Fortress 2 SDK written in Rust that I update every now and then. Most of this has been written in early November. I've published it so that I can track interest and remind myself of it's existance.

# Features
Currently it's fairly limited, but it includes:
- Memory modules, scanning (patterns, string references, both support nth match)
- C/C++ ABI stuffs (Vftables, C strings, etc...)
- Generic pointer wrappers
- WINAPI utilities
- Everything error handled
- Thread-safe global context utilities
- Some basic game stuff (vfcalls, hooking, etc...)

# Future
I want to redo many things, primarily to get rid of some stupid macros and abstract the things they're currently abstracting better - be that fitting stuff into the type system or using procedural macros, revamp memory scanning (a thought I haven't explored yet is making scans return an iterator), reconsider some project organization details, etc. Can't promise anything though.

# Contributing
I don't really think this is ready for contributions, there aren't even any guidelines I have in mind yet, let alone conventions I'd like to enforce.

# License
Currently no license, I'll eventually add some, I don't really have much reason to bother to add one yet.