#[inline]
pub fn read(source: &mut dyn std::io::Read, len: usize) -> std::io::Result<Vec<u8>> {
    let mut out = vec![];
    if len == 0 {
        source.read_to_end(&mut out)?;
    } else {
        out.resize(len, 0u8);
        source.read_exact(&mut out)?;
    }
    return Ok(out);
}

#[inline]
pub fn read_delimited(source: &mut dyn std::io::BufRead, delimiter: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut out = vec![];
    let mut delim_buffer = vec![];
    delim_buffer.resize(delimiter.len() - 1, 0u8);
    loop {
        // Read up to first delimiter byte
        source.read_until(delimiter[0], &mut out)?;

        // Read rest of delimiter
        let mut delim_size = 0usize;
        while delim_size < delimiter.len() - 1 {
            let count = source.read(&mut delim_buffer[delim_size..])?;
            if count == 0 {
                // EOF, delimiter wasn't read, return everything
                out.extend(delim_buffer);
                return Ok(out);
            }
            delim_size += count;
        }

        // If the whole delimiter was read return
        if delim_buffer == delimiter[1..] {
            out.pop();
            return Ok(out);
        }

        // Something that wasn't the delimiter was read, treat as data and continue
        out.extend(&delim_buffer);
    }
}

#[cfg(feature = "async")]
pub mod async_ {
    pub use futures::io::{
        AsyncReadExt,
        AsyncBufReadExt,
        AsyncWriteExt,
    };

    #[inline]
    pub async fn read<T: futures::io::AsyncReadExt + Unpin>(source: &mut T, len: usize) -> std::io::Result<Vec<u8>> {
        let mut out = vec![];
        if len == 0 {
            source.read_to_end(&mut out).await?;
        } else {
            out.resize(len, 0u8);
            source.read_exact(&mut out).await?;
        }
        return Ok(out);
    }

    #[inline]
    pub async fn read_delimited<
        T: futures::io::AsyncBufReadExt + Unpin,
    >(source: &mut T, delimiter: &[u8]) -> std::io::Result<Vec<u8>> {
        let mut out = vec![];
        let mut delim_buffer = vec![];
        delim_buffer.resize(delimiter.len() - 1, 0u8);
        loop {
            // Read up to first delimiter byte
            source.read_until(delimiter[0], &mut out).await?;

            // Read rest of delimiter
            let mut delim_size = 0usize;
            while delim_size < delimiter.len() - 1 {
                let count = source.read(&mut delim_buffer[delim_size..]).await?;
                if count == 0 {
                    // EOF, delimiter wasn't read, return everything
                    out.extend(delim_buffer);
                    return Ok(out);
                }
                delim_size += count;
            }

            // If the whole delimiter was read return
            if delim_buffer == delimiter[1..] {
                out.pop();
                return Ok(out);
            }

            // Something that wasn't the delimiter was read, treat as data and continue
            out.extend(&delim_buffer);
        }
    }
}

pub mod lowheap_error {
    pub trait ReadErrCtx<T> {
        fn errorize(self, text: &'static str) -> Result<T, &'static str>;
    }

    impl<T, E> ReadErrCtx<T> for Result<T, E> {
        fn errorize(self, text: &'static str) -> Result<T, &'static str> {
            match self {
                Err(_) => return Err(text),
                Ok(v) => return Ok(v),
            }
        }
    }
}

#[cfg(not(noheap))]
pub mod error {
    use std::fmt::Display;

    #[derive(Debug)]
    pub struct ReadError {
        pub node: &'static str,
        pub inner: String,
    }

    impl Display for ReadError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            return format_args!("Error reading in node {}: {}", self.node, self.inner).fmt(f);
        }
    }

    impl std::error::Error for ReadError { }

    pub trait ReadErrCtx<T> {
        fn errorize(self, node: &'static str) -> Result<T, ReadError>;
    }

    impl<T, E: Display> ReadErrCtx<T> for Result<T, E> {
        fn errorize(self, node: &'static str) -> Result<T, ReadError> {
            match self {
                Err(e) => return Err(ReadError {
                    node: node,
                    inner: e.to_string(),
                }),
                Ok(v) => return Ok(v),
            }
        }
    }
}
