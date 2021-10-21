use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "T", untagged)]
pub enum Fetchable<T> {
    /// The value does not exist, but can be fetched.
    None,
    /// The value has been fetched.
    Fetched(T),
}

impl<T> Fetchable<T> {
    pub fn is_fetched(&self) -> bool {
        matches!(self, Self::Fetched(_))
    }

    pub fn fetch<F>(&mut self, f: F) -> Result<&T>
    where
        F: FnOnce() -> Result<T>,
    {
        match self {
            Self::Fetched(ref value) => Ok(value),
            Self::None => {
                let value = f()?;
                *self = Self::Fetched(value);
                // This is safe, since we've just fetched successfully
                Ok(self.assume_fetched())
            }
        }
    }

    pub fn fetch_mut<F>(&mut self, f: F) -> Result<&mut T>
    where
        F: FnOnce() -> Result<T>,
    {
        match self {
            Self::Fetched(ref mut value) => Ok(value),
            Self::None => {
                let value = f()?;
                *self = Self::Fetched(value);
                // This is safe, since we've just fetched successfully
                Ok(self.assume_fetched_mut())
            }
        }
    }

    /// Panics if the resource doesn't exist
    fn assume_fetched(&self) -> &T {
        match self {
            Self::Fetched(value) => value,
            Self::None => panic!("Called .unwrap() on a Fetchable that is not fetched!"),
        }
    }

    /// Panics if the resource doesn't exist
    fn assume_fetched_mut(&mut self) -> &mut T {
        match self {
            Self::Fetched(value) => value,
            Self::None => panic!("Called .unwrap() on a Fetchable that is not fetched!"),
        }
    }
}

impl<T> From<T> for Fetchable<T> {
    fn from(value: T) -> Self {
        Fetchable::Fetched(value)
    }
}
