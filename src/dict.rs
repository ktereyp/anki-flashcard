use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::io::{BufRead, Read, Take};

pub enum Error {
    UnknownErr(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Error::UnknownErr(e) => {
                f.write_str(e)
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

pub struct DslDict {
    file: String,
}

impl DslDict {
    fn parse(&mut self) -> Result<(), Error> {
        let f = fs::File::open(self.file.as_str())?;
        let m = f.metadata()?;
        println!("size: {}", m.len());

        let mut f = std::io::BufReader::new(f);
        let mut count = 0;
        while count < 100 {
            count += 1;

            let mut buf = vec![];
            let n = f.read_until(b'\n', &mut buf)?;
            if n == 0 {
                return Ok(());
            }
            let b = {
                let l = buf.len() / 2;
                unsafe {
                    std::slice::from_raw_parts_mut(buf.as_mut_ptr().cast::<u16>(), l)
                }
            };
            let s = String::from_utf16_lossy(b);
            println!("s: {}, size: {}", s, n);
        }
        Ok(())
    }
}

struct BufReader<R> {
    inner: R,
    buf: Box<[u16]>,
    cap: usize,
    pos: usize,
}

impl<R> BufReader<R> where R: Read {
    fn new(r: R) -> Self <R> {
        let mut buf = unsafe{
            let buf = Box::new_uninit_slice(cap).assume_init() ;
        };
        inner.initializer().initialize(&mut buf);
        BufReader {
            inner: r,
            buf,
            cap: 512,
            pos: 0,
        }
    }
}

impl<R: Read> Read for BufReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<R: Read> std::io::BufRead for BufReader<R> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        todo!()
    }

    fn consume(&mut self, amt: usize) {
        let s = "".to_owned();
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_test() {
        let mut dict = DslDict {
            file: "/home/try/Documents/Dict_Longman_Oxford/Oxford/b.dsl".to_owned(),
        };
        let r = dict.parse();
        assert!(r.is_ok());
    }
}
