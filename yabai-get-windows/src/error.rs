use log::error;
use std::fmt;

pub trait LogError {
    type Unwrapped;
    fn log_error(self) -> Option<Self::Unwrapped>;
}

impl<T, E: fmt::Display> LogError for Result<T, E> {
    type Unwrapped = T;
    fn log_error(self) -> Option<Self::Unwrapped> {
        self.inspect_err(|e| error!("{}", e)).ok()
    }
}

pub trait AllOrNone<T> {
    fn all_or_none(self) -> Option<T>;
}

impl<U, V> AllOrNone<(U, V)> for (Option<U>, Option<V>) {
    fn all_or_none(self) -> Option<(U, V)> {
        Some((self.0?, self.1?))
    }
}

impl<T> AllOrNone<T> for Option<Option<T>> {
    fn all_or_none(self) -> Option<T> {
        self.unwrap_or(None)
    }
}

impl<T> AllOrNone<T> for Option<Option<Option<T>>> {
    fn all_or_none(self) -> Option<T> {
        self.unwrap_or(None).unwrap_or(None)
    }
}
