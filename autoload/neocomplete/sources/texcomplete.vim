let s:save_cpo = &cpo
set cpo&vim

let s:keywordmap = {
            \ '\\[gG]ls\(desc\|symbol\)\?{': ['gls'],
            \ '\\\(page\|name\|auto\)\?ref{': ['lbl'],
            \ '\\\(text\)\?cite\(\w\|author\|year\)\?\(\[[^\n]\{-}\]\)\?{': ['bib'],
            \ }

let s:typedict = {
            \ 'Glossaryentry': 'g',
            \ 'Section': 's',
            \ 'Label': 'l',
            \ 'Citation': 'b',
            \}

let s:source = {
   \ 'name' : 'texcomplete',
   \ 'kind' : 'keyword',
   \ 'mark' : '[tex]',
   \ 'rank' : 10,
   \ 'filetype' : {'tex': 1},
   \ 'input_pattern' : join(keys(s:keywordmap), '\|'),
   \ }

let s:path = exists("b:vimtex.tex") ? b:vimtex.tex : getcwd()

function! s:source.get_complete_position(context) abort
    return strlen(a:context.input)
    "return strridx(a:context.input, '{') + 1
endfunction

function! s:source.gather_candidates(context)
    let results = []
    let tocomp = []
    let tex_keyword = a:context.input
    for item in items(s:keywordmap)
        if match(tex_keyword, item[0]) != -1
            call extend(tocomp, item[1])
        endif
    endfor
    return s:get_results(join(tocomp, ','))
endfunction

fun s:get_results(mode)
    let suggestions = []
    let cmd = g:texparser_path . " --json -i -a " . a:mode . " " . s:path
    let dec = json_decode(system(cmd))
    for element in dec
        let variant = s:typedict[element['attributes']['variant']]
        let lbl = element['label']
        let menue = ""
        let abbr = lbl
        if variant ==# 'g'
            let menue = element['attributes']['fields'][0]['description']
        elseif variant ==# 's'
            let abbr = '[sec] '.lbl
        elseif variant ==# 'l'
            let abbr = '[lbl] '.lbl
        elseif variant ==# 'b'
            let abbr = element['attributes']['fields'][0]['authortext']
            let menue = element['attributes']['fields'][0]['title']
        endif

        call add(suggestions, {
           \ 'word' : lbl,
           \ 'abbr' : abbr,
           \ 'menu' : menue,
           \ 'kind' : variant,
           \ })
   endfor
   return suggestions
endf

function! neocomplete#sources#texcomplete#define() abort
    if !executable(g:texparser_path)
        return {}
    endif
    return s:source
endfunction

let &cpo = s:save_cpo
unlet s:save_cpo
