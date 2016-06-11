#[macro_use] extern crate clap;
extern crate texparser;
extern crate rustc_serialize;

use clap::{App, Arg};
use texparser::{Config,parse_path};
use std::path::PathBuf;
use std::borrow::Cow;
use std::collections::HashMap;
use rustc_serialize::json;
use ::texparser::parser::CompletionType::*;
use ::texparser::parser::single_pass;

fn main() {
    let m = App::new("texcomplete")
        .author("hecal3 <hecal3@users.noreply.github.com>")
        .about("Parse tex glossaryentrys")
        //.version(&*format!("v{}", crate_version!()))
        .bin_name("texcomplete")
        .args_from_usage(
            "<INPUT>                      'Sets the input file or directory to use'
            -o --output [FORMAT]          'Append output fields to the completion results (e.g. \"year,author\" for bibitems, \"symbol,description\" for glossaryentrys)'
            -a --action [ACTION]          'Sets things to search for (Defaults to bib,gls,sec)'
            -s --separator [SEPARATOR]    'Sets an value separator. (Defaults to comma)'
            -i --includes                 'Also search in included files (recursive)'")
        .arg(Arg::with_name("json")
           .long("json")
           .help("Output as Json (disregards --output)")
           .conflicts_with("output"))
        .get_matches();

    if let Some(mut inp) = m.value_of_lossy("INPUT") {
        let sep = match m.value_of("separator") {
            Some(sep) => sep,
            None => ",",
        };
        let format = match m.values_of("output") {
            Some(format) => format.collect(),
            None => vec![],
        };
        let action = match m.values_of("action") {
            Some(format) => format.collect(),
            None => vec!["bib", "gls", "sec", "lbl"],
        };

        let cfg = Config{
            includes: m.is_present("includes"),
            bib: action.contains(&"bib"),
            glossaries: action.contains(&"gls"),
            sections: action.contains(&"sec"),
            labels: action.contains(&"lbl"),
        };

        let ipath = PathBuf::from(&inp.to_mut());
        let results = if ipath.is_file() {
             single_pass(&ipath, &cfg)
        } else {
             parse_path(&ipath, cfg)
        };

        if m.is_present("json") {
            let encoded = json::encode(&results).unwrap();
            println!("{}", encoded);
        } else {
            for result in results {
                match result.attributes {
                    Glossaryentry(ref map) => {
                        print!("GLOSSARYENTRY{}{}{}", sep, result.label, sep);
                        print_map(map, &format, sep);
                    },
                    Citation(ref map, ref typ) => {
                        print!("CITATION{}{}{}{}", sep, result.label, sep, typ);
                        print_map(map, &format, sep);
                    },
                    Section(ref typ) => {
                        print!("SECTION{}{}{}{}", sep, result.label, sep, typ);
                    },
                    Label(..) => {
                        print!("LABEL{}{}{}{}", sep, result.label, sep, sep);
                    },
                }
            println!("");
            }
        }
    }
}

fn print_map(map: &HashMap<String,String>, format: &[&str], sep: &str) {
    for string in format {
        match map.get(*string) {
            Some(v) => print!("{}{}", sep, escape_csv(v,sep).to_mut()),
            None => print!("{}", sep),
        }
    }
}

fn escape_csv<'a>(toprint: &'a str, sep: &str) -> Cow<'a,str> {
    if toprint.contains(sep) {
        Cow::Owned(format!("\"{}\"", toprint))
    } else {
        Cow::Borrowed(toprint)
    }
}
