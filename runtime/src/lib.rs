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
