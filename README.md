# texcomplete-vim

Vim autocompletion for LaTeX projects.

## Installation and Use

```vim
Plug 'hecal3/texcomplete-vim', { 'do': 'cargo build --release' }
```

This plugin has a compiled component. You will need to have rust installed to compile the binary parser.
If you use a plugin manager without post-update-hook functionality
or want to install manually, you will have to invoke `cargo build --release` yourself.

As an upside, the completion is very fast and free of lags even on Windows.

Should work out of the box for:
- glossaryentrys(\gls{, \glssymbol, etc.)
- citations (\cite{}, \textcite{}, \citeauthor, etc.)
- labels (\ref{}, \autoref{}, \nameref{}, etc.)

with either [neocomplete](https://github.com/Shougo/neocomplete.vim) or [deoplete](https://github.com/Shougo/deoplete.nvim).

The underlying binary program (sitting at ./target/release/texcomplete by default) can also provide sectioning information.
This is however currently not used in the vim plugin.

Please make sure, to have your vim working directory set to the LaTeX main directory.
Alternatively, if you have [vimtex](https://github.com/lervag/vimtex) installed,
the completion will be called on the current vimtex main file.

## Settings (optional)

```vim
let g:texparser_path = '/path/to/texcomplete-binary'
```

## Todo
- The neocomplete source might need some additional work. It does not always behave as advertised...
