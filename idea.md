Onivim 3?
lets go man that's an idea
Okay so like it's a neovim client but blazingly fast (tm)

Easy plugins in lua and dig into engine source performance with rust (goated idea?)

UNDO SELECTIONS, let me select, operation undo and still selected

So it's like neovim but uses the gpui
NO NEEDED CONFIG FILE, everything *just works tm*
Lsp is BAKED in and *very* easy to add, 
To add an lsp you can paste in a git repo and it's added to a list of extentions
You shouldn't need to dig around in a config file, but you *can*
Everything is very plain but it's snappy, not sluggish
Has vim keybindings built in and configurable

I hate the half baked lua/vimscript nvim style so it's a new api that is *somewhat* interoperable
It's just not dumb and works properly

This is a dumb idea and yet I don't care

1.) Vim motions and optionally regular vscode style input
2.) Multithreaded extentions in lua or rust
3.) Native extentions that are built in (can be disabled)
4.) Lsp implementation
5.) A clean gui that is simple but also snappy (unlike nvim with a lot of extentions) with file tree and tabs (native) and git tree (extention)
6.)  Opinionated defaults that are configurable, and disablable as modules, so in config ui you can use GUI module [x] (tabs and file tree) and disable it
7.) Re implementation of the vim standard without the legacy code keeping the keybindings
