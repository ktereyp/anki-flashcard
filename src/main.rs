extern crate core;

mod dict;

use std::io::{BufRead, BufReader, Write};
use std::process::exit;
use dict::*;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    dict_dir: String,
    #[clap(short, long)]
    vol_list: String,
    #[clap(short, long)]
    out: String,
    #[clap(short, long, default_value_t = 1)]
    count: u8,
}

fn main() {
    let args = Args::parse();

    let mut dicts = vec![];

    match std::fs::read_dir(args.dict_dir.as_str()) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(file) => {
                        let mut dict = DslDict::new(file.path().to_str().unwrap());
                        let r = dict.parse();
                        if r.is_err() {
                            eprintln!("parse error: {}", r.unwrap_err());
                            exit(1);
                        }
                        dicts.push(dict);
                    }
                    Err(e) => {
                        println!("cannot read dir: {}", e);
                        exit(1);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("cannot read dir {}: {}", args.dict_dir, e);
            exit(1);
        }
    }

    let word_list = std::fs::File::open(args.vol_list.as_str());
    if word_list.is_err() {
        eprintln!("cannot open vocabulary file: {}", word_list.unwrap_err());
        exit(1);
    }

    let word_list = word_list.unwrap();
    let mut reader = BufReader::new(word_list);

    let mut vocabulary = {
        let f = std::fs::File::create(args.out.clone() + "/anki_csv.txt");
        if f.is_err() {
            eprintln!("cannot create vocabulary.txt: {}", f.unwrap_err());
            exit(1);
        }
        f.unwrap()
    };

    let mut missing = 0;
    loop {
        let mut word = String::new();
        let r = reader.read_line(&mut word);
        if r.is_err() {
            break;
        }
        let word = word.trim();
        if word.is_empty() {
            break;
        }

        let word_entry = {
            let mut r = None;
            for dict in &dicts {
                r = dict.query(word);
                if r.is_some() {
                    break;
                }
            }
            r
        };
        match word_entry {
            Some(w) => {
                let html = w.to_html();
                if html.is_err() {
                    eprintln!("error: {}", html.unwrap_err());
                    exit(1);
                }
                let html = html.unwrap().replace("\n", "").replace("|", "%7C");

                let audio = {
                    let mut audio = String::new();
                    let start_mark = "[sound:";
                    let audio_start = html.find(start_mark);
                    if audio_start.is_some() {
                        let audio_start = audio_start.unwrap() + start_mark.len();
                        let audio_end = html.as_str()[audio_start..].find(".wav");
                        if audio_end.is_some() {
                            let audio_end = audio_start + audio_end.unwrap() + 4;
                            let r = &html.as_str()[audio_start..audio_end];
                            audio.push_str(r);
                        }
                    }
                    audio
                };

                let _ = vocabulary.write_all(word.as_bytes());
                let _ = vocabulary.write_all("|".as_bytes());
                let _ = vocabulary.write_all(html.as_bytes());
                let _ = vocabulary.write_all("|".as_bytes());
                let _ = vocabulary.write_all(format!("[sound:{}]\n", audio).as_bytes());
            }
            None => {
                println!("not found: {}", word);
                missing += 1;
            }
        }
    }
    println!("missing words: {}", missing);
}
