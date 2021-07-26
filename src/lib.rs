use std::io::{self, Read};

pub struct CatRead<T, F> {
    source: F,
    current: T,
}

impl<T, F> CatRead<T, F>
where
    T: Read,
    F: FnMut() -> Option<io::Result<T>>,
{
    pub fn new(mut source: F) -> io::Result<Self> {
        let current = source()
            .transpose()?
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no sources"))?;
        Ok(Self { source, current })
    }
}

impl<T, F> Read for CatRead<T, F>
where
    T: Read,
    F: FnMut() -> Option<io::Result<T>>,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut bytes_read = 0;

        loop {
            match self.current.read(&mut buf[bytes_read..]) {
                // Change belts or surrender
                Ok(0) => match (self.source)() {
                    Some(source) => self.current = source?,
                    None => return Ok(bytes_read),
                }

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
        let mut sources = sources.iter().map(Cursor::new);
        let catread = CatRead::new(|| sources.next().map(Ok)).unwrap();
        let actual = read_to_string(catread).unwrap();
        assert_eq!(actual, "Hello, world!");
    }

    fn read_to_string(mut reader: impl Read) -> io::Result<String> {
        let mut buf = String::new();
        reader.read_to_string(&mut buf)?;
        Ok(buf)
    }
}
