///Parser result type.
pub enum ParseResult<T, E> {
    ///Successfully parsed data.
    Ok(T),
    ///Parser failed in a trivial/recoverable way
    Recoverable(E),
    ///Parser failed in a catastrophic/unrecoverable way
    Unrecoverable(E),
}

impl<T, E> From<T> for ParseResult<T, E> {
    fn from(v: T) -> Self {
        Self::Ok(v)
    }
}

impl<T, E> ParseResult<T, E> {
    /// Returns `true` if the parse result is [`Ok`].
    ///
    /// [`Ok`]: ParseResult::Ok
    #[must_use]
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok(..))
    }

    /// Returns `true` if the parse result is [`Recoverable`].
    ///
    /// [`Recoverable`]: ParseResult::Recoverable
    #[must_use]
    pub fn is_recoverable(&self) -> bool {
        matches!(self, Self::Recoverable(..))
    }

    /// Returns `true` if the parse result is [`Unrecoverable`].
    ///
    /// [`Unrecoverable`]: ParseResult::Unrecoverable
    #[must_use]
    pub fn is_unrecoverable(&self) -> bool {
        matches!(self, Self::Unrecoverable(..))
    }

    /// Maps a `ParseResult<T, E>` to `ParseResult<U, E>` by applying a function to a
    /// contained [`Ok`] value, leaving an [`Err`] value untouched.
    ///
    /// [`Ok`]: ParseResult::Ok
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> ParseResult<U, E> {
        use ParseResult as PR;
        match self {
            PR::Ok(item) => PR::Ok(f(item)),
            PR::Recoverable(e) => PR::Recoverable(e),
            PR::Unrecoverable(e) => PR::Unrecoverable(e),
        }
    }

    /// Maps a `ParseResult<T, E>` to `ParseResult<T, U>` by applying a function to a
    /// contained [`Recoverable`] or [`Unrecoverable`] value, leaving an [`Ok`] value untouched.
    ///
    /// [`Ok`]: ParseResult::Ok
    /// [`Recoverable`]: ParseResult::Recoverable
    /// [`Unrecoverable`]: ParseResult::Unrecoverable
    pub fn map_err<U>(self, f: impl FnOnce(E) -> U) -> ParseResult<T, U> {
        use ParseResult as PR;
        match self {
            PR::Ok(item) => PR::Ok(item),
            PR::Recoverable(e) => PR::Recoverable(f(e)),
            PR::Unrecoverable(e) => PR::Unrecoverable(f(e)),
        }
    }

    ///Replaces the value inside the [`Ok`] variant, leaving [`Recoverable`] and [`Unrecoverable`]
    ///untouched.
    ///
    /// [`Ok`]: ParseResult::Ok
    /// [`Recoverable`]: ParseResult::Recoverable
    /// [`Unrecoverable`]: ParseResult::Unrecoverable
    pub fn replace<U>(self, value: U) -> ParseResult<U, E> {
        use ParseResult as PR;
        match self {
            PR::Ok(_) => PR::Ok(value),
            PR::Recoverable(e) => PR::Recoverable(e),
            PR::Unrecoverable(e) => PR::Unrecoverable(e),
        }
    }

    ///Replaces the value inside the [`Recoverable`] and [`Unrecoverable`] variants, leaving [`Ok`]
    ///untouched.
    ///
    /// [`Ok`]: ParseResult::Ok
    /// [`Recoverable`]: ParseResult::Recoverable
    /// [`Unrecoverable`]: ParseResult::Unrecoverable
    pub fn replace_err<U>(self, err: U) -> ParseResult<T, U> {
        use ParseResult as PR;
        match self {
            PR::Ok(ok) => PR::Ok(ok),
            PR::Recoverable(_) => PR::Recoverable(err),
            PR::Unrecoverable(_) => PR::Unrecoverable(err),
        }
    }
}
