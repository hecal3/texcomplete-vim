
if exists('g:texparser_init')
  finish
endif
let g:texparser_init = 1

if !exists('g:texparser_path')
    let path = escape(expand('<sfile>:h'), '\')
    let path = fnamemodify(path, ':h') . '/target/release/'
    "if isdirectory(path)
        "let s:pathsep = has("win32") ? ';' : ':'
        "let $PATH .= s:pathsep . path
    "endif
    let g:texparser_path = path . 'texcomplete'
endif
