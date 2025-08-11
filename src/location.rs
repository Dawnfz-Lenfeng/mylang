#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl Location {
    pub fn new() -> Self {
        Self {
            line: 1,
            column: 1,
            offset: 0,
        }
    }

    pub fn advance(&mut self, ch: char) {
        self.offset += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
    }
}

/// A wrapper that adds location information to any AST node
#[derive(Debug, Clone, PartialEq)]
pub struct Located<T> {
    pub inner: T,
    pub location: Location,
}

impl<T> Located<T> {
    pub fn new(inner: T, location: Location) -> Self {
        Self { inner, location }
    }

    /// Get the location of this node
    pub fn location(&self) -> Location {
        self.location
    }

    /// Get a reference to the inner value
    pub fn as_inner(&self) -> &T {
        &self.inner
    }

    /// Get the inner value, consuming this wrapper
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Map the inner value to a different type while preserving location
    pub fn map<U, F>(self, f: F) -> Located<U>
    where
        F: FnOnce(T) -> U,
    {
        Located::new(f(self.inner), self.location)
    }
}
