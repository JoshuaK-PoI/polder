use std::iter::Peekable;

use crate::lang::compiler::CompilerResult;

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

pub(crate) trait UnshiftExpect<T, K, E> {
    fn unshift_expect(&mut self, expected: K) -> CompilerResult<&mut T>;
    fn unshift_expect_any(&mut self, expected: &[K]) -> CompilerResult<&mut T>;
    // fn unshift_if(&mut self, expected: K) -> Option<&mut T>;
    fn next_matches(&mut self, expected: K) -> bool;
    // fn next_matches_any(&mut self, expected: &[K]) -> bool;
}
