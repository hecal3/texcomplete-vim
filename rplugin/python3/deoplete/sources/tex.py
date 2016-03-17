import subprocess
import re
import json
from neovim.api.nvim import NvimError
from .base import Base

class Source(Base):
    def __init__(self, vim):
        Base.__init__(self, vim)

        self.name = 'tex'
        self.mark = '[tex]'
        self.filetypes = ['tex']
        self.rank = 500

        self.__encoding = self.vim.eval('&encoding')
        self.__executable = self.vim.eval('g:texparser_path')

        try:
            self.__mainfile = self.vim.eval('b:vimtex.tex')
        except NvimError:
            self.__mainfile = self.vim.eval('getcwd()')

        self.__keywordmap = [
                [ ['gls'], r'(\\[gG]ls(desc|symbol)?\{$)' ],
                [ ['lbl'], r'(\\(page|name|auto)?ref\{$)' ],
                [ ['bib'], r'(\\(text)?cite(\w|author|year)?(\[.*?\])?\{(\w*,)*$)' ]
                ]
        self.input_pattern = "|".join([ elem[1] for elem in self.__keywordmap ])
        self.__typemap = {
                'Glossaryentry': 'g',
                'Citation': 'c',
                'Section': 's',
                'Label': 'l',
                }
        self.__citetype = {
                'article': 'a',
                'book': 'B',
                'inbook': 'b',
                'booklet': 'b',
                'conference': 'C',
                'incollection': 'c',
                'manual': 'm',
                'misc': 'M',
                'proceedings': 'P',
                'inproceedings': 'p',
                'techreport': 'r',
                'phdthesis': 'T',
                'masterthesis': 't',
                'unpublished': 'u',
                }
        self.__sectype = {
                'part': '',
                'chapter': '#',
                'section': '##',
                'subsection': '###',
                'subsubsection': '####',
                'paragraph': '#####',
                'subparagraph': '######',
                'label': 'lbl',
                }

    def get_complete_position(self, context):
        if not self.__executable:
            return -1

        m = re.search(r'\w*$', context['input'])
        return m.start() if m else -1

    def gather_candidates(self, context):
        tex_keyword = '\\'+context['input'].rsplit('\\', 1)[-1]
        candidates = []
        tocomp = []
        for item in self.__keywordmap:
            if re.match(item[1], tex_keyword):
                tocomp.extend(item[0])

        # self.vim.command('echom("'+",".join(tocomp)+'")')
        candidates = self.get_results(context, tocomp)
        candidates = sorted(candidates, key=lambda k: k['kind'], reverse=True) 
        candidates = sorted(candidates, key=lambda k: k['sort']) 
        return candidates

    def get_results(self, context, mode):
        try:
            cmd = [self.__executable, '--json', '-a', ",".join(mode), '-i', self.__mainfile]
            results = subprocess.check_output(filter(None, cmd)).decode(self.__encoding)
            results = json.loads(results)
            candidates = []
            for line in results:
                sort = ""
                variant = self.__typemap[line['attributes']['variant']]
                if variant == 'g':
                    fields = line['attributes']['fields'][0]
                    abbr = line['label']
                    desc = fields['description'] + "   (" + fields['symbol'] + ")"
                    sort = line['label']
                elif variant == 'c':
                    fields = line['attributes']['fields'][0]
                    abbr = "[" + self.__citetype[line['attributes']['fields'][1]] + "] " + fields['authortext']
                    desc = fields['title']
                    sort = line['label']
                elif variant == 's':
                    abbr = self.__sectype[line['attributes']['fields'][0]] + " " + line['label']
                    # desc = line['attributes']['fields'][0]
                    desc = ""
                elif variant == 'l':
                    abbr = "[lbl] " + line['label']
                    # sort = line['label']
                    # desc = line['attributes']['fields'][0]
                    desc = ""
                else:
                    continue

                completion = {'kind': variant, 'word': line['label'], 'abbr': abbr, 'menu': desc, 'dup': 1, 'sort': sort}
                candidates.append(completion)
        except subprocess.CalledProcessError:
            return []
        return candidates
