# rcwm

## Raccoon WM

A tiling X11 window manager written in Rust

Written with XCB bindings because I'm a masochist.

It follows the style of dynamic window managers such as XMonad and QTile, with a main window/region and satellite windows on the side.

It supports multiple desktops, and can send windows between all of them.
It can also toggle window states between floating and tiling, and preserves this state between desktops.

Non-reparenting (for now, but based on the design goals, it may become a reality).

Current SLOC count: `2171`

Built upon [afwm](https://iim.gay:8080/afwm/about/) by grufwub, with inspiration from [penrose](https://docs.rs/penrose/0.2.0/penrose/index.html) by sminez.

Design goals:

- Full ICCCM + EWMH support. (I did read those manuals.)
- Multiple methods of configuration (in order of preference)
  - Lua
  - TOML
  - Directly in the source code (if used as a library)
- IPC via a custom client
- Builtin bars + support for other bars
- Available as a library for users to build custom WMs
  - Can be compiled with or without Lua support (cargo features)
