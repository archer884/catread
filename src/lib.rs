use std::io::{self, Read};

pub struct CatRead<T, S> {
    source: S,
    current: T,
}

impl<T, S> CatRead<T, S>
where
    T: Read,
    S: Iterator<Item = io::Result<T>>,
{
    pub fn new<I>(source: I) -> io::Result<Self>
    where
        I: IntoIterator<IntoIter = S>,
    {
        let mut source = source.into_iter();
        let current = source
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no sources"))??;
        Ok(Self { source, current })
    }
}

impl<T, S> Read for CatRead<T, S>
where
    T: Read,
    S: Iterator<Item = io::Result<T>>,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut bytes_read = 0;

        loop {
            match self.current.read(&mut buf[bytes_read..]) {
                // Change belts or surrender
                Ok(0) => match self.source.next() {
                    Some(source) => self.current = source?,
                    None => return Ok(bytes_read),
                },

                // Keep firing, assholes!
                Ok(len) => {
                    bytes_read += len;
                    if bytes_read == buf.len() {
                        return Ok(bytes_read);
                    }
                }

                Err(e) if e.kind() == io::ErrorKind::Interrupted => (),
                Err(e) => return Err(e),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, Cursor, Read};

    use crate::CatRead;

    #[test]
    fn it_works() {
        let sources = &["Hello, ", "world!"];
        let sources = sources.iter().map(Cursor::new);
        let catread = CatRead::new(sources.map(Ok)).unwrap();
        let actual = read_to_string(catread).unwrap();
        assert_eq!(actual, "Hello, world!");
    }

    fn read_to_string(mut reader: impl Read) -> io::Result<String> {
        let mut buf = String::new();
        reader.read_to_string(&mut buf)?;
        Ok(buf)
    }
}
