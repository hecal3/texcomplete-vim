#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(test, feature(test))]
extern crate glob;
extern crate regex;
extern crate rustc_serialize;

use glob::glob;
use regex::Regex;

use std::path::{PathBuf,Path};
use std::fs::File;
use std::io::prelude::Read;

pub mod parser;
use parser::Completion;
use parser::single_pass;

#[derive(Copy,Clone)]
pub struct Config {
    pub includes: bool,
    pub bib: bool,
    pub glossaries: bool,
    pub sections: bool,
    pub labels: bool,
}

pub fn parse_path<P: AsRef<Path>>(path: P, cfg: Config) -> Vec<Completion> {
    let mut paths = vec![];
    if cfg.bib {
        paths.append(&mut glob_bib_files(&path));
    }
    if cfg.glossaries || cfg.labels || cfg.sections {
        paths.append(&mut glob_files(&path));
    }
    parse_path_single(&paths, cfg)
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

fn parse_path_single(paths: &[PathBuf], mut cfg: Config) -> Vec<Completion> {
    match find_mainfile(paths) {
        Some(mainfilepath) => single_pass(mainfilepath, &cfg),
        None => {
            cfg.includes = false;
            let mut results = vec![];
            for path in paths {
                results.append(&mut single_pass(&path, &cfg));
            }
            results
        },
    }
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

#[cfg(test)]
mod tests {
    extern crate test;
    use super::*;
    use std::path::Path;

    #[bench]
    fn bench_path(b: &mut test::Bencher) {

        let cfg = Config{
            includes: true,
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
