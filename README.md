# Tabry Rust
Documentation WIP

Tabry is a program (and optionally, a mini-language) that allows you to easily create tab completion for 3rd-party programs. Here is an example:

```
cmd zellij

sub (a,attach d,delete-session k,kill-session) @arg-session
sub da,delete-all-sessions
sub e,edit

defargs @arg-session {
  arg session {
    opts shell "zellij ls -ns"
  }
}
```

# Installation

* TODO -- Nix derivation
* TODO -- fish completion (port from ruby tabry)

```bash
# Install:
cargo install --git https://github.com/evanbattaglia/tabry-rs
# ^^^ tabry is now installed to ~/.cargo/bin, which should be in your path
echo 'source <(tabry bash)' >> ~/.bash_profile

# Add a completion
mkdir -p ~/.local/share/tabry
vi ~/.local/share/tabry/zellij.tabry # copy the above example into this file

# Test it out:
source ~/.bash_profile # or open a new terminal
zellij #<tab> should now show completions
```

# Tabry language

Please see the [documentation in the Ruby project](https://github.com/evanbattaglia/tabry/blob/master/LANGUAGE_REFERENCE.md).

# Project history

This is a port of [Tabry](https://github.com/evanbattaglia/tabry/) completion engine and compiler to Rust. Because Rust avoids the ~75ms (depending on machine, of course) startup time of Node, Ruby, etc., it is natural choice for the completion engine. Going forward I intend this to be the principal implementation of Tabry, at least for compiling and completion purposes. (The Ruby implementation for at least for now remain for the purposes of building Tabry-compatible CLIs). 

