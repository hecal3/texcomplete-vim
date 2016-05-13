use std::collections::HashMap;
use std::fmt;
use regex::{Regex,RegexSet};
use std::path::{Path,PathBuf};
use self::CompletionType::*;
use std::fs::File;
use std::io::prelude::Read;
use std::io::BufReader;
use std::rc::Rc;

pub use ::Config;

const MATCHINC: &'static [&'static str] = &[r"\\include(\[.*?\])?\{",
                                            r"\\input(\[.*?\])?\{" ];

const MATCHBIB: &'static [&'static str] = &[r"\\addbibresource(\[.*?\])?\{",
                                            r"\\bibliography(\[.*?\])?\{" ];

const MATCHSEC: &'static [&'static str] = &[r"\\section\*?(\[.*?\])?\{",
                                            r"\\chapter\*?(\[.*?\])?\{",
                                            r"\\part\*?(\[.*?\])?\{",
                                            r"\\subsection\*?(\[.*?\])?\{",
                                            r"\\subsubsection\*?(\[.*?\])?\{" ];

const MATCHLBL: &'static [&'static str] = &[r"\\label(\[.*?\])?\{",
                                            r"\\label\{" ];

const MATCHGLS: &'static [&'static str] = &[r"\\newglossaryentry\{",
                                            r"\\longnewglossaryentry\{" ];
const COMMENT: &'static str = r"[^\\]%";

const STRIP_SEC_RIGHT: &'static [char] = &['{', ' ', '*'];
const STRIP_SEC_LEFT: &'static [char] = &[' ', '\\'];

#[derive(Debug,RustcDecodable,RustcEncodable)]
pub enum CompletionType {
    Glossaryentry(HashMap<String,String>),
    Citation(HashMap<String,String>,String),
    Section(String),
    Label(u32),
}

#[derive(Debug,RustcDecodable,RustcEncodable)]
pub struct Completion {
    pub label: String,
    pub attributes: CompletionType,
}

impl Completion {
    pub fn new<N: Into<String>, C: Into<CompletionType>>(name: N, attr: C) -> Completion {
        Completion { label: name.into(), attributes: attr.into() }
    }
}

impl fmt::Display for Completion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {:?}", self.label, self.attributes)
    }
}

struct Parser {
    incset: RegexSet,
    bibset: RegexSet,
    secset: RegexSet,
    lblset: RegexSet,
    glsset: RegexSet,
    full: Regex,
    commen: Regex,
}

pub fn single_pass<P: AsRef<Path>>(filepath: P, cfg: &Config) -> Vec<Completion> {
    let incset = RegexSet::new(MATCHINC).unwrap();
    let bibset = RegexSet::new(MATCHBIB).unwrap();
    let secset = RegexSet::new(MATCHSEC).unwrap();
    let lblset = RegexSet::new(MATCHLBL).unwrap();
    let glsset = RegexSet::new(MATCHGLS).unwrap();
    
    let mut tomatch: Vec<&str> = Vec::new();
    if cfg.glossaries {
        tomatch.extend_from_slice(MATCHGLS);
    }
    if cfg.sections {
        tomatch.extend_from_slice(MATCHSEC);
    }
    if cfg.includes {
        tomatch.extend_from_slice(MATCHINC);
    }
    if cfg.bib {
        tomatch.extend_from_slice(MATCHBIB);
    }
    if cfg.labels {
        tomatch.extend_from_slice(MATCHLBL);
    }
    let conn = tomatch.join("|");
    let regexstr = format!(r"({})", conn);
    let re = Regex::new(&regexstr).unwrap();
    let comre = Regex::new(COMMENT).unwrap();

    let reg=Rc::new(Parser{
                    incset: incset,
                    bibset: bibset,
                    secset: secset,
                    lblset: lblset,
                    glsset: glsset,
                    full: re,
                    commen: comre,
                    });

    let fp = filepath.as_ref().extension();
    if fp.is_some() && fp.unwrap().to_str().unwrap_or("") == "bib" {
        parse_bibfile(filepath.as_ref())
    } else {
        _single_pass(filepath.as_ref(),cfg,reg)
    }
}

fn _single_pass<P: AsRef<Path>>(filepath: P, cfg: &Config, reg: Rc<Parser>) -> Vec<Completion> {
    let mut results = Vec::new();

    if let Ok(mut file) = File::open(filepath.as_ref()) {
        let mut s = String::new();
        let _ = file.read_to_string(&mut s);

        let mut rest = s.as_str();

        while let Some(find) = reg.full.find(rest) {
            let (start, end) = find;
            if is_comment(&rest[..start+1], &reg.commen) {
                rest = &rest[end-1..];
                continue;
            }
            let typ = &rest[start..end].trim_left();
            let (lbl, re) = match_parens(&rest[end-1..]);

            if cfg.includes && reg.incset.is_match(typ) {
                let npath = get_incfilename(filepath.as_ref(), lbl, false);
                results.append(&mut _single_pass(&npath, cfg, reg.clone()));
            }
            else if cfg.bib && reg.bibset.is_match(typ) {
                let npath = get_incfilename(filepath.as_ref(), lbl, true);
                results.append(&mut parse_bibfile(&npath));
            }
            else if cfg.labels && reg.lblset.is_match(typ) {
                results.push(Completion::new(lbl.trim(),Label(0)));
            }
            else if cfg.sections && reg.secset.is_match(typ) {
                let mat = typ.trim_right_matches(STRIP_SEC_RIGHT);
                let mat = mat.trim_left_matches(STRIP_SEC_LEFT);
                results.push(Completion::new(lbl.trim(), Section(String::from(mat))));
            } else if cfg.glossaries && reg.glsset.is_match(typ) {

                let (entry, rest) = match_parens(re);

                let mut map: HashMap<String,String> = values(entry).into_iter()
                    .map(|(k,v)| (k.to_owned(),v.to_owned()))
                    .collect();

                if typ.contains("long") {
                    let (descr, _) = match_parens(rest.trim_left());
                    map.insert("description".to_owned(), descr.to_owned());
                }
                results.push(Completion::new(lbl,Glossaryentry(map)));
            }
            rest = re;
        }
    }
    results
}

fn is_comment(inp: &str, comre: &Regex) -> bool {
    match inp.rfind('\n') {
        None => false,
        Some(nlpos) => comre.is_match(&inp[nlpos+1..]),
    }
}

fn get_incfilename<P: AsRef<Path>>(path: P, lbl: &str, bib: bool) -> PathBuf {
    let mut npath = path.as_ref().parent().unwrap().to_path_buf();
    npath.push(lbl.trim());
    if bib {
        npath.set_extension("bib");
    } else {
        npath.set_extension("tex");
    }
    npath
}

fn parse_bibfile<P: AsRef<Path>>(filepath: P) -> Vec<Completion> {
    if let Ok(file) = File::open(filepath.as_ref()) {
        let mut reader = BufReader::new(file);
        let mut s = String::new();
        let _ = reader.read_to_string(&mut s);
        parse_bib(&s)
    } else {
        return vec![];
    }
}

fn split_bib(input: &str) -> Vec<&str> {
    let re = Regex::new(r"(?m)^@(?-m)").unwrap();
    re.split(input).collect::<Vec<_>>()
}

fn parse_bib(input: &str) -> Vec<Completion> {
    let mut split = split_bib(input);
    if !split.is_empty() {
        split.remove(0);
    }
    let re = Regex::new(r"(\S*)\{").unwrap();
    let mut results = Vec::with_capacity(split.len());
    for entry in split {
        if let Some(caps) = re.captures(entry) {
            let art = caps.at(1).unwrap();
            if art.to_lowercase() == "comment" {
                continue;
            }
            let len = art.len();
            let rest = &entry[len..];
            let (dat, _) = match_parens(rest);
            
            let labelsplit: Vec<&str> = dat.splitn(2,',').collect();
            if labelsplit.len() > 1 {
                let label = labelsplit[0].trim().to_owned();

                let mut attr: HashMap<String,String> = values(labelsplit[1]).into_iter()
                    .map(|(x,y)| (x.to_owned().to_lowercase(), y.to_owned())).collect();

                let mut inval = String::new();
                {
                    match (attr.get("author"), attr.get("year")) {
                        (Some(author), Some(year)) => {
                            inval = author_text(author, year);
                        },
                        (_,_) => {},
                    }
                }
                if !inval.is_empty() {
                    attr.insert(String::from("authortext"), inval);
                }
                results.push( Completion::new(label, Citation(attr,String::from(art.to_lowercase()))));
            }
        }
    }
    results
}

fn author_text(authors: &str, year: &str) -> String {
    let mut names = vec![];
    for author in authors.split("and") {
        let name: Vec<&str> = author.trim().split(',').collect();
        names.push(name[0]);
    }

    match names.len() {
        1 => format!("{} ({})", names[0], year),
        2 => format!("{} & {} ({})", names[0], names[1], year),
        _ => format!("{} et al. ({})", names[0], year),
    }
}

fn values(input: &str) -> Vec<(&str,&str)> {
    let mut cont = true;
    let mut rest = input.trim();
    let mut a = Vec::new();

    while cont {
        if let Some(s) = rest.find('=') {
            let value: Option<&str>;
            let key = &rest[..s].trim();
            rest = rest[s+1..].trim();

            rest = match rest.chars().next() {
                 Some('{') => {
                    let (i,j) = match_parens(rest);
                    value = Some(i);
                    //println!("parensmatch {:?}", value);
                    j
                 },
                 _ => {
                     match rest.find(',') {
                         Some(i) => {
                             value = Some(&rest[..i]);
                             //println!("commamatch {:?}", value);
                             &rest[i+1..]
                         },
                         None => {
                             cont = false;
                             value = Some(rest);
                             //println!("lastentry {:?}", value);
                             rest
                         },
                     }
                 },
            };
            //println!("{:?}", value);
            let k: &[_] = &[' ', '{', '}', ',', '\t', '\n', '\r', '%'];
            let v: &[_] = &[' ', ',', '\t', '\n', '\r', '%'];
            if let Some(val) = value {
                a.push((key.trim_matches(k), val.trim_matches(v)))
            }
        } else {
            break;
        }
    }
    a
}

fn match_parens(input: &str) -> (&str,&str) {
    let mut counter = 0;
    let mut start = 0;;
    let mut end = 0;
    let mut iter = input.chars();

    loop {
        let cha = iter.next();
        if let Some(c) = cha {
            end += c.len_utf8();
        }
        match cha {
            Some('{') => {
                counter += 1;
                if start == 0 {
                    start = counter;
                }
            },
            Some('}') => {
                counter -= 1;
                if counter == 0 {
                    break;
                };
            },
            None => break,
            _ => {},
        }
    }
    (&input[start..end-1], &input[end..])
}

#[cfg(test)]
mod tests {
    extern crate test;
    use super::*;
    use std::path::Path;

    #[test]
    fn find_bibfile() {
        let path = Path::new("path/to/bibfile");
        assert_eq!(path.is_file(), true);
        assert_eq!(path.is_dir(), false);
    }

    #[test]
    fn open_bibfile() {
        let path = Path::new("path/to/bibfile");
        let entrys = super::parse_bibfile(path);
        assert_eq!(entrys.len(), 25350);
    }

    #[bench]
    fn _large_bibfile(b: &mut test::Bencher) {
        let path = Path::new("path/to/large/bibfile");
        b.iter(|| {
            let _ = super::parse_bibfile(path);
        })
    }

    #[bench]
    fn bench_single_pass(b: &mut test::Bencher) {

        let cfg = Config{
            includes: true,
            bib: true,
            glossaries: true,
            sections: true,
            labels: true,
        };
        b.iter(|| {
            let _ = single_pass(Path::new("path/to/main/latex/file"), &cfg);
        })
    }
}
