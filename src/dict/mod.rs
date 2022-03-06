use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::io::{Read};
use std::str::FromStr;


#[derive(Eq, PartialEq)]
pub enum Error {
    EOF,
    UnknownErr(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Error::UnknownErr(e) => {
                f.write_str(e)
            }
            Error::EOF => {
                f.write_str("eof")
            }
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Error::UnknownErr(e) => {
                f.write_str(e)
            }
            Error::EOF => {
                f.write_str("eof")
            }
        }
    }
}

impl<T> From<T> for Error where T: std::error::Error {
    fn from(e: T) -> Self {
        let s = format!("{}", e);
        Error::UnknownErr(s)
    }
}

pub trait Dict {
    fn lookup(word: &str) -> Option<String>;
}

#[derive(Default)]
pub struct Word {
    pub name: String,
    alias: Vec<String>,
    pub(crate) def: Vec<String>,
}

impl Word {
    pub fn to_html(&self) -> Result<String, Error> {
        let mut html = String::new();

        for def in &self.def {
            //[m0][c darkgray] [/c][b][c red]throw[/c][/b] {{id=000044728}} [c rosyrown]\[[/c][c darkslategray][b]throw[/b][/c] [c darkslategray][b]throws[/b][/c] [c darkslategray][b]threw[/b][/c] [c darkslategray][b]throwing[/b][/c] [c darkslategray][b]thrown[/b][/c][c rosybrown]\][/c] [c darkgray] [/c][c orange]verb,[/c] [c darkgray] [/c][c orange]noun[/c] [p]BrE[/p] [c darkgray] [/c][c darkcyan]\[θrəʊ\][/c] [s]z_throw__gb_1.wav[/s] [p]NAmE[/p] [c darkgray] [/c][c darkcyan]\[θroʊ\][/c] [s]z_throw__us_1.wav[/s]
            let chars = def.chars().collect::<Vec<char>>();
            let mut cmds: Vec<String> = vec![];
            let mut pos = 0;
            while pos < chars.len() {
                let c = chars[pos];
                match c {
                    '\\' => {
                        pos += 1;
                        html.push_str(chars[pos].to_string().as_str());
                        pos += 1;
                    }
                    '/' => {
                        pos += 1;
                        if pos + 1 < chars.len() && chars[pos + 1] == ']' {
                            pos += 1;
                        } else {
                            html.push_str("/");
                        }
                    }
                    '{' => {
                        // ignore syntax
                        // {{key=value}}
                        pos += 1;
                        if pos < chars.len() && chars[pos] == '{' {
                            let mut p = pos;
                            while chars[p] != '}' {
                                p += 1;
                            }
                            if p + 1 < chars.len() && chars[p + 1] == '}' {
                                pos = p + 2;
                            }
                        } else {
                            html.push_str("{");
                        }
                    }
                    '[' => {
                        pos += 1;
                        if pos == chars.len() {
                            continue;
                        }
                        let cmd = chars[pos];
                        match cmd {
                            'm' => {
                                pos += 1;
                                let left = pos;
                                while pos < chars.len() && chars[pos] != ']' && chars[pos] != '[' {
                                    pos += 1;
                                }

                                let indent = if left < pos {
                                    let num_str = &chars[left..pos].iter().collect::<String>();
                                    let num = u32::from_str(num_str);
                                    if num.is_err() {
                                        log::error!("invalid line: number invalid: {}, line: {}", num_str, def);
                                        continue;
                                    }
                                    num.unwrap() * 20
                                } else {
                                    0
                                };
                                html.push_str(format!("<div style=\"text-indent: {}px\">", indent).as_str());
                                log::debug!("enter m");
                                if pos < chars.len() && chars[pos] != '[' {
                                    pos += 1;
                                }
                            }
                            '/' => {
                                pos += 1;

                                let expect_cmd = {
                                    match cmds.last() {
                                        Some(c) => c,
                                        None => continue,
                                    }
                                };
                                let mut cmd_end_pos = pos;
                                let expect_cmd_chars = expect_cmd.chars().collect::<Vec<char>>();
                                while cmd_end_pos < chars.len() && (cmd_end_pos - pos) < expect_cmd_chars.len() && chars[cmd_end_pos] == expect_cmd_chars[cmd_end_pos - pos] {
                                    cmd_end_pos += 1;
                                }
                                let mut poped = false;
                                if cmd_end_pos != pos {
                                    let cmd = chars[pos..cmd_end_pos].iter().collect::<String>();
                                    if expect_cmd == &cmd {
                                        match expect_cmd.as_str() {
                                            "c" | "p" => {
                                                html.push_str("</span>");
                                            }
                                            "s" => {
                                                html.push_str("]</span>");
                                            }
                                            "b" => {
                                                html.push_str("</b>");
                                            }
                                            _ => {
                                                // ignore
                                            }
                                        }
                                        cmds.pop();
                                        poped = true;
                                        pos = cmd_end_pos;
                                    }
                                }
                                if pos < chars.len() && chars[pos] == ']' {
                                    pos += 1;
                                    if !poped {
                                        cmds.pop();
                                    }
                                }
                            }
                            _ => {
                                let mut cmd_end_pos = pos + 1;
                                while cmd_end_pos < chars.len()
                                    && chars[cmd_end_pos] != ' '
                                    && chars[cmd_end_pos] != ']'
                                    && chars[cmd_end_pos] != '[' {
                                    cmd_end_pos += 1;
                                }

                                let cmd = chars[pos..cmd_end_pos].iter().collect::<String>();
                                log::debug!("enter cmd {}", cmd);
                                cmds.push(cmd.clone());
                                pos = cmd_end_pos;

                                if pos == chars.len() {
                                    continue;
                                }

                                if chars[pos] == ' ' {
                                    pos += 1;
                                }
                                let left = pos;
                                while pos < chars.len() && chars[pos] != ']' && chars[pos] != '[' {
                                    pos += 1;
                                }
                                let cmd_para = chars[left..pos].iter().collect::<String>();
                                if pos < chars.len() && chars[pos] == ']' {
                                    pos += 1;
                                }
                                match cmd.as_str() {
                                    "c" => {
                                        html.push_str(format!("<span style=\"color: {}\">", cmd_para).as_str());
                                    }
                                    "b" => {
                                        html.push_str("<b>");
                                    }
                                    "p" => {
                                        html.push_str("<span style=\"color: green\">");
                                    }
                                    "s" => {
                                        html.push_str("<span class=\"voice\">[sound:");
                                    }
                                    _ => {
                                        // ignore
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        html.push_str(c.to_string().as_str());
                        pos += 1;
                    }
                }
            };
            if cmds.len() != 0 {
                eprintln!("invalid inline: {}, still has cmds: {:?}", def, cmds);
            }
            html.push_str("</div>");
        }

        Ok(html)
    }
}

pub struct DslDict {
    file: String,
    words: HashMap<String, Word>,
}

impl DslDict {
    pub fn new(file: &str) -> DslDict {
        DslDict {
            file: file.to_owned(),
            words: HashMap::default(),
        }
    }

    pub(crate) fn query(&self, p0: &str) -> Option<&Word> {
        self.words.get(p0)
    }

    pub fn parse(&mut self) -> Result<(), Error> {
        let f = fs::File::open(self.file.as_str())?;
        let m = f.metadata()?;
        println!("size: {}", m.len());

        let mut f = BufReader::new(f);
        let mut word: Word = Default::default();
        loop {
            let line = f.read_line();
            if line.is_err() {
                let e = line.unwrap_err();
                println!("read error: {}", e);
                break;
            }
            let line = line.unwrap();

            if line.starts_with("#") {
                println!("{}", line);
                continue;
            } else if line.starts_with("\t") {
                if word.name.len() == 0 {
                    eprintln!("unmatched definition: {}", line);
                    continue;
                } else {
                    word.def.push(line);
                }
            } else {
                // new word
                if word.name.len() == 0 {
                    word.name = line;
                } else if word.def.len() > 0 {
                    let w = word;

                    self.words.insert(w.name.clone(), w);

                    word = Default::default();
                    word.name = line;
                } else {
                    word.alias.push(line);
                }
            }
        }
        Ok(())
    }
}

struct BufReader<R> {
    inner: R,
    buf: Box<[u16; 4096]>,
    r_pos: usize,
    l_pos: usize,
}

impl<R> BufReader<R> where R: Read {
    fn new(r: R) -> Self {
        let buf: Box<[u16; 4096]> = Box::new([0; 4096]);
        BufReader {
            inner: r,
            buf,
            r_pos: 0,
            l_pos: 0,
        }
    }

    fn read_line(&mut self) -> Result<String, Error> {
        let delim: u16 = '\n' as u16;
        let mut r: Vec<u16> = vec![];

        while self.l_pos < self.r_pos {
            let c = self.buf[self.l_pos];
            if c == delim {
                self.l_pos += 1;
                return Ok(String::from_utf16_lossy(r.as_slice()));
            }
            r.push(c);
            self.l_pos += 1;
        }

        self.r_pos = 0;
        self.l_pos = 0;

        let mut buf: Vec<u8> = vec![0; self.buf.len() * 2];

        loop {
            let mut n = self.inner.read(&mut buf)?;
            if n == 0 {
                return Err(Error::EOF);
            }
            if n & 1 > 0 {
                // not even number
                // read another byte
                let r = self.inner.read(&mut buf[n..n + 1])?;
                if r == 0 {
                    eprintln!("size is not event number")
                } else {
                    n += 1;
                }
            }

            let mut pos = 2;
            let mut found = false;
            while pos <= n {
                let c = ((buf[pos - 1] as u16) << 8) | buf[pos - 2] as u16;
                if found {
                    self.buf[self.r_pos] = c;
                    self.r_pos += 1;
                } else {
                    r.push(c);
                    if c == delim {
                        found = true;
                    }
                }
                pos += 2;
            }
            if found {
                return Ok(String::from_utf16_lossy(r.as_slice()));
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::BufRead;
    use super::*;

    #[test]
    fn parse_test() {
        let mut dict = DslDict::new("a.dsl");
        let r = dict.parse();
        assert!(r.is_ok());

        let word = dict.query("execrate");
        assert!(r.is_ok());
        let word = word.unwrap();
        for i in &word.def {
            println!("{}", i);
        }
    }

    #[test]
    fn to_html() {
        let word = Word {
            name: "down".to_string(),
            alias: vec![],
            def: vec![
                //"{{Thesaurus}}[m3][c darkslategray][u]Thesaurus:[/u][/c".to_owned(),
                //"[m4][ex][*]• [/][/ex][c darkgray] [/c][ex][*]{{x}}She completely dominated the conversation.{{/x}} [/*][/ex]".to_owned(),
                "[m2][ex][*]• [/*][/ex][ex[*]{{x}}The first lot of visitors has/have arrived.{{/x}} [/*][/ex]".to_owned(),
            ],
        };

        let mut f = std::io::BufReader::new(fs::File::open("down.dsl").unwrap());
        loop {
            let mut s = String::new();
            let r = f.read_line(&mut s);
            assert!(r.is_ok());
            let r = r.unwrap();
            if r == 0 {
                break;
            } else {
                //word.def.push(s.to_owned());
            }
        }

        match word.to_html() {
            Ok(html) => {
                println!("{}", html);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
