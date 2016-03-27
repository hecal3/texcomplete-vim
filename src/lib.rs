#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(test, feature(test))]
extern crate glob;
extern crate regex;
extern crate rustc_serialize;

use glob::glob;
use regex::Regex;

use std::thread;
use std::path::{PathBuf,Path};
use std::fs::File;
use std::io::prelude::Read;
use std::ffi::OsStr;

pub mod parser;
use parser::Completion;
use parser::{parse_tex,parse_bib,single_pass};

#[derive(Copy,Clone)]
pub struct Config {
    pub includes: bool,
    pub threads: u64,
    pub bib: bool,
    pub glossaries: bool,
    pub sections: bool,
    pub labels: bool,
}

pub fn parse_path<P: AsRef<Path>>(path: P, cfg: Config) -> Vec<Completion> {
    let mut paths = vec![];
    //cfg.includes = false;
    if cfg.bib {
        paths.append(&mut glob_bib_files(&path));
    }
    if cfg.glossaries || cfg.labels || cfg.sections {
        paths.append(&mut glob_files(&path));
    }
    //println!("patht:{:?}", paths);
    match cfg.threads {
        0|1 => parse_path_single(paths, cfg),
        _ => parse_path_concurrent(paths, cfg),
    }
}

fn find_mainfile(paths: &[PathBuf]) -> Option<PathBuf> {
    let re = Regex::new(r"(?m)^[^%]*\\documentclass(?-m)").unwrap();
    for path in paths {
        if let Ok(mut file) = File::open(&path) {
            let mut s = String::new();
            let _ = file.read_to_string(&mut s);
            if re.is_match(&s) {
                return Some(path.clone());
            }
        }
    }
    None
}

fn parse_path_single(paths: Vec<PathBuf>, mut cfg: Config) -> Vec<Completion> {
    match find_mainfile(&paths) {
        Some(mainfilepath) => single_pass(mainfilepath, &cfg),
        None => {
            cfg.includes = false;
            let mut results = vec![];
            for path in paths {
                results.append(&mut parse_file(&path, &cfg));
            }
            results
        },
    }
}

fn parse_path_concurrent(paths: Vec<PathBuf>, mut cfg: Config) -> Vec<Completion> {
    cfg.includes = false;
    let mut threads = vec![];
    //let ncpu = num_cpus::get();
    let ncpu = cfg.threads as usize;
    let njobs = paths.len()/ncpu;
    let mut splitwork = Vec::new();
    splitwork.push(paths);
    //println!("{}", ncpu);
    for _ in 1..ncpu {
        let mut ent = splitwork.pop().unwrap();
        let nvec = ent.split_off(njobs);
        splitwork.push(ent);
        splitwork.push(nvec);
    }
    //let lcfg = cfg.clone();
    for elem in splitwork {
        let child = thread::spawn(move || {
            let mut res = Vec::new();
            for path in elem {
                res.append(&mut parse_file(&path, &cfg));
            }
            res
        });
        threads.push(child);
    }
    let mut results = vec![];
    for thread in threads {
        let result = thread.join();
        match result {
            Ok(mut result) => results.append(&mut result),
            Err(e) => println!("error {:?}", e),
        };
    }
    results
}

fn glob_files<P: AsRef<Path>>(path: P) -> Vec<PathBuf> {
    glob(&format!("{}/**/*.tex", path.as_ref().display()))
        .unwrap().filter_map(Result::ok)
        .map(|x| x.to_path_buf())
        .collect::<Vec<_>>()
}

fn glob_bib_files<P: AsRef<Path>>(path: P) -> Vec<PathBuf> {
    glob(&format!("{}/**/*.bib", path.as_ref().display()))
        .unwrap().filter_map(Result::ok)
        .map(|x| x.to_path_buf())
        .collect::<Vec<_>>()
}

/// deprecated
pub fn parse_file(path: &Path, cfg: &Config) -> Vec<Completion> {
    //println!("parsefile {}", path.display());
    if let Ok(mut file) = File::open(&path) {
        let mut s = String::new();
        let _ = file.read_to_string(&mut s);

        let mut results = vec![];

        if cfg.includes {
            let regex = if cfg.bib {
                r"\\(?P<inp>include|input|addbibresource|bibliography)\{(?P<file>\S+)\}"
            } else {
                r"\\(?P<inp>include|input)\{(?P<file>\S+)\}"
            };
            //println!("{}", regex);
            let re = Regex::new(regex).unwrap();
            let caps = re.captures_iter(&s).into_iter();
            for cap in caps {
                match (cap.name("file"), cap.name("inp"), path.parent()) {
                    (Some(files), Some(inp), Some(path)) => {
                        for file in files.split(',') {
                            let mut npath = path.to_path_buf();
                            npath.push(file);
                            //println!("{}", inp);
                            if inp.contains("bib") {
                                npath.set_extension("bib");
                            } else {
                                npath.set_extension("tex");
                            }
                            //println!("{:?}", npath.display());
                            results.append(&mut parse_file(&npath, cfg));
                        }
                    },
                    (_,_,_) => {},
                }
            }
        }

        //println!("file:{}", path.display());
        if cfg.bib && path.extension().unwrap_or_else(|| OsStr::new("")) == "bib" {
            //println!("call bib");
            results.append(&mut parse_bib(&s));
        }

        if (cfg.glossaries || cfg.sections || cfg.labels)
            && path.extension().unwrap_or_else(|| OsStr::new("")) != "bib" {
            //println!("call glos");
            results.append(&mut parse_tex(&s, cfg))
        }

        results
    } else {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use super::*;
    use std::path::Path;

    #[bench]
    fn bench_file(b: &mut test::Bencher) {

        let cfg = Config{
            includes: true,
            threads: 1,
            bib: true,
            glossaries: true,
            sections: true,
            labels: true,
        };
        b.iter(|| {
            let _ = parse_file(&Path::new("path/to/main/latex/file"), &cfg);
        })
    }
    #[bench]
    fn bench_path(b: &mut test::Bencher) {

        let cfg = Config{
            includes: true,
            threads: 1,
            bib: true,
            glossaries: true,
            sections: true,
            labels: true,
        };
        b.iter(|| {
            let _ = parse_path(&Path::new("path/to/latex/directory"), cfg);
        })
    }
}
