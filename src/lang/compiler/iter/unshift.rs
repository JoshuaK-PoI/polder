use std::iter::Peekable;


/// Allows vector elements to be taken from the front of the vector.
/// It is a safe wrapper for:
/// ```rust
/// if vec.is_empty() {
///     None
/// } else {
///     Some(vec.remove(0))
/// }
/// ```
/// that can be used as an expression, similar to `vec.pop()`.
pub(crate) trait Unshift<T> {
    /// Takes the first element of the vector and returns it.
    ///
    /// Returns `None` if the vector is empty.
    fn unshift(&mut self) -> Option<T>;
}

impl<T, U> Unshift<U> for Peekable<T>
where
    T: Iterator<Item = U>,
{
    fn unshift(&mut self) -> Option<U> {
        if self.peek().is_none() {
            None
        } else {
            self.next()
        }
    }
}
