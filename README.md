# rcwm
## Raccoon WM
A tiling X11 window manager written in Rust

Written with XCB bindings because I'm a masochist.

Right now it can do both floating and basic dynamic tiling.
It supports multiple workspaces.

Non-reparenting (for now).

Built upon [afwm](https://iim.gay:8080/afwm/about/) by grufwub.

Design goals:
- Full ICCCM + EWMH support. (I did read those manuals.)
- Configured via a custom scripting language
- IPC via a shell
- Builtin bars + support for other bars