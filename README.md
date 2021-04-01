# rcwm
## Raccoon WM
A tiling X11 window manager written in Rust

Written with XCB bindings because I'm a masochist.

It follows the style of dynamic window managers such as XMonad and QTile, with a main window/region and satellite windows on the side.

It supports multiple desktops, and can send windows between all of them.
It can also toggle window states between floating and tiling, and preserves this state between desktops.

Non-reparenting (for now).

Current SLOC count: `1765`

Built upon [afwm](https://iim.gay:8080/afwm/about/) by grufwub.

Design goals:
- Full ICCCM + EWMH support. (I did read those manuals.)
- Configured via a custom scripting language
- IPC via a shell
- Builtin bars + support for other bars