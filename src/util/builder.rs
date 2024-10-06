//! # Num Builder
//!
//! A builder pattern implementation for numeric calculations and transformations.
//!
//! This module provides a fluent API for building numeric values through a chain of
//! operations. It tracks the history of values and supports operations like:
//!
//! - Basic arithmetic (add, subtract, multiply, divide)
//! - Modulo
//! - Rounding operations for floating point numbers
//! - Random value generation
//!
//! ## Example
//! ```rust
//! use num_builder::Builder;
//!
//! let result = Builder::new(10.0)
//!     .add(5.0)
//!     .mulitply_by(2.0)
//!     .round()
//!     .build();
//! ```
//!

use num::{Float, Num};
use rand::{
    distributions::{
        uniform::{SampleRange, SampleUniform},
        Standard,
    },
    prelude::{random, thread_rng, Distribution},
    Rng,
};
use ryu::Buffer;
use std::fmt::{Display, Error, Formatter};

// Builder --------------------------------------------------------------------------- /

/// A builder pattern for handling numeric calculations.
///
/// This struct provides methods for chaining numeric operations on a value,
/// keeping track of the history of values, and building the final result.
///
/// # Type Parameters
///
/// * `T` - The numeric type being built. Must implement `num::Num` and `Copy`.
///
/// # Examples
///
/// ```
/// use num_builder::Builder;
///
/// let result = Builder::new(10.0)
///     .add(5.0)
///     .mulitply_by(2.0)
///     .round()
///     .build();
///
/// assert_eq!(result, 30.0);
/// ```
///
/// Operations can also be chained with other builders:
///
/// ```
/// use num_builder::Builder;
///
/// let builder1 = Builder::new(10.0);
/// let builder2 = Builder::new(5.0);
///
/// let result = builder1.add_from_builder(builder2).build();
/// assert_eq!(result, 15.0);
/// ```
#[derive(Debug)]
pub struct Builder<T> {
    current: T,
    previous: Vec<T>,
}

impl<T: Num + Copy> Builder<T> {
    /// Constructs a new `Builder` with an initial value.
    ///
    /// # Arguments
    ///
    /// * `initial_value` - The starting value for the builder.
    ///
    /// # Example
    ///
    /// ```
    /// use num_builder::Builder;
    ///
    /// let builder = Builder::new(10.0);
    /// ```
    ///
    /// This is equivalent to creating an empty history and setting the current value:
    ///
    /// ```
    /// use num_builder::Builder;
    ///
    /// let builder = Builder {
    ///     current: 10.0,
    ///     previous: vec![]
    /// };
    /// ```
    pub fn new(initial_value: T) -> Self {
        Self {
            current: initial_value,
            previous: vec![],
        }
    }

    /// Creates a new Builder from an existing Builder by taking its current value.
    ///
    /// # Arguments
    ///
    /// * `builder` - The Builder to create from
    ///
    /// # Example
    ///
    /// ```
    /// use num_builder::Builder;
    ///
    /// let builder1 = Builder::new(10.0);
    /// let builder2 = Builder::from(builder1); // builder2 starts with 10.0
    /// ```
    pub fn from(builder: Builder<T>) -> Self {
        Self::new(builder.build())
    }

    /// Adds a value to the current value.
    ///
    /// # Arguments
    ///
    /// * `amount` - The value to add
    ///
    /// # Example
    ///
    /// ```
    /// use num_builder::Builder;
    ///
    /// let result = Builder::new(10.0).add(5.0).build(); // 15.0
    /// ```
    pub fn add(mut self, amount: T) -> Self {
        let current = self.current;
        self.previous.push(current);
        self.current = current + amount;
        self
    }

    /// Adds the current value of another Builder.
    ///
    /// # Arguments
    ///
    /// * `builder` - The Builder whose value to add
    ///
    /// # Example
    ///
    /// ```
    /// use num_builder::Builder;
    ///
    /// let builder1 = Builder::new(10.0);
    /// let builder2 = Builder::new(5.0);
    /// let result = builder1.add_from_builder(builder2).build(); // 15.0
    /// ```
    pub fn add_from_builder(self, builder: Builder<T>) -> Self {
        self.add(builder.build())
    }

    /// Subtracts a value from the current value.
    ///
    /// # Arguments
    ///
    /// * `amount` - The value to subtract
    ///
    /// # Example
    ///
    /// ```
    /// use num_builder::Builder;
    ///
    /// let result = Builder::new(10.0).subtract(5.0).build(); // 5.0
    /// ```
    pub fn subtract(mut self, amount: T) -> Self {
        let current = self.current;
        self.previous.push(current);
        self.current = current - amount;
        self
    }

    /// Subtracts the current value of another Builder.
    ///
    /// # Arguments
    ///
    /// * `builder` - The Builder whose value to subtract
    ///
    /// # Example
    ///
    /// ```
    /// use num_builder::Builder;
    ///
    /// let builder1 = Builder::new(10.0);
    /// let builder2 = Builder::new(5.0);
    /// let result = builder1.subtract_from_builder(builder2).build(); // 5.0
    /// ```
    pub fn subtract_from_builder(self, builder: Builder<T>) -> Self {
        self.subtract(builder.build())
    }

    /// Multiplies the current value by another value.
    ///
    /// # Arguments
    ///
    /// * `amount` - The value to multiply by
    ///
    /// # Example
    ///
    /// ```
    /// use num_builder::Builder;
    ///
    /// let result = Builder::new(10.0).mulitply_by(2.0).build(); // 20.0
    /// ```
    pub fn mulitply_by(mut self, amount: T) -> Self {
        let current = self.current;
        self.previous.push(current);
        self.current = current * amount;
        self
    }

    /// Multiplies by the current value of another Builder.
    ///
    /// # Arguments
    ///
    /// * `builder` - The Builder whose value to multiply by
    ///
    /// # Example
    ///
    /// ```
    /// use num_builder::Builder;
    ///
    /// let builder1 = Builder::new(10.0);
    /// let builder2 = Builder::new(2.0);
    /// let result = builder1.mulitply_by_builder(builder2).build(); // 20.0
    /// ```
    pub fn mulitply_by_builder(self, builder: Builder<T>) -> Self {
        self.mulitply_by(builder.build())
    }

    /// Divides the current value by another value.
    ///
    /// # Arguments
    ///
    /// * `amount` - The value to divide by
    ///
    /// # Example
    ///
    /// ```
    /// use num_builder::Builder;
    ///
    /// let result = Builder::new(10.0).divide_by(2.0).build(); // 5.0
    /// ```
    pub fn divide_by(mut self, amount: T) -> Self {
        let current = self.current;
        self.previous.push(current);
        self.current = current / amount;
        self
    }

    /// Divides by the current value of another Builder.
    ///
    /// # Arguments
    ///
    /// * `builder` - The Builder whose value to divide by
    ///
    /// # Example
    ///
    /// ```
    /// use num_builder::Builder;
    ///
    /// let builder1 = Builder::new(10.0);
    /// let builder2 = Builder::new(2.0);
    /// let result = builder1.divide_by_builder(builder2).build(); // 5.0
    /// ```
    pub fn divide_by_builder(self, builder: Builder<T>) -> Self {
        self.divide_by(builder.build())
    }

    /// Computes the modulo of the current value with another value.
    ///
    /// # Arguments
    ///
    /// * `amount` - The value to compute modulo with
    ///
    /// # Example
    ///
    /// ```
    /// use num_builder::Builder;
    ///
    /// let result = Builder::new(5.0).modulo(3.0).build(); // 2.0
    /// ```
    pub fn modulo(mut self, amount: T) -> Self {
        let current = self.current;
        self.previous.push(current);
        self.current = current % amount;
        self
    }

    /// Computes modulo using the current value of another Builder.
    ///
    /// # Arguments
    ///
    /// * `builder` - The Builder whose value to use for modulo
    ///
    /// # Example
    ///
    /// ```
    /// use num_builder::Builder;
    ///
    /// let builder1 = Builder::new(5.0);
    /// let builder2 = Builder::new(3.0);
    /// let result = builder1.modulo_from_builder(builder2).build(); // 2.0
    /// ```
    pub fn modulo_from_builder(self, builder: Builder<T>) -> Self {
        self.modulo(builder.build())
    }

    /// Returns the current value.
    ///
    /// # Example
    ///
    /// ```
    /// use num_builder::Builder;
    ///
    /// let value = Builder::new(10.0).build(); // 10.0
    /// ```
    pub fn build(&self) -> T {
        self.current
    }

    /// Returns a vector containing all previous values and the current value.
    ///
    /// # Example
    ///
    /// ```
    /// use num_builder::Builder;
    ///
    /// let values = Builder::new(10.0)
    ///     .add(5.0)
    ///     .multiply_by(2.0)
    ///     .all(); // [10.0, 15.0, 30.0]
    /// ```
    pub fn all(&self) -> Vec<T> {
        let mut all = self.previous.clone();
        all.push(self.current);
        all
    }
}

impl<T: Num + Copy> Builder<T>
where
    Standard: Distribution<T>,
{
    /// Generates a new random value using the standard distribution.
    ///
    /// # Example
    ///
    /// ```
    /// use num_builder::Builder;
    ///
    /// let builder = Builder::new(10.0).randomize(); // Sets to random f64
    /// ```
    pub fn randomize(mut self) -> Self {
        let current = self.current;
        self.previous.push(current);
        self.current = random();
        self
    }
}

impl<T: Num + Copy + SampleUniform> Builder<T> {
    /// Generates a random value within a given range.
    ///
    /// # Arguments
    ///
    /// * `range` - The range to generate a value within
    ///
    /// # Example
    ///
    /// ```
    /// use num_builder::Builder;
    ///
    /// let builder = Builder::new(10.0).randomize_within(0.0..20.0);
    /// ```
    pub fn randomize_within<R: SampleRange<T>>(mut self, range: R) -> Self {
        let current = self.current;
        self.previous.push(current);
        let mut rng = thread_rng();
        self.current = rng.gen_range(range);
        self
    }
}

impl<T: Float> Builder<T> {
    /// Returns a new Builder with the current value rounded to the nearest whole number.
    ///
    /// # Example
    ///
    /// ```
    /// use num_builder::Builder;
    ///
    /// let result = Builder::new(2.6).round().build(); // 3.0
    /// ```
    ///
    /// # See also
    ///
    /// * [`ceil`] - Rounds up to the nearest whole number
    /// * [`floor`] - Rounds down to the nearest whole number
    ///
    pub fn round(mut self) -> Self {
        let current = self.current;
        self.previous.push(current);
        self.current = current.round();
        self
    }

    /// Returns a new Builder with the current value rounded up to the nearest whole number.
    ///
    /// # Example
    ///
    /// ```
    /// use num_builder::Builder;
    ///
    /// let result = Builder::new(2.1).ceil().build(); // 3.0
    /// ```
    ///
    /// # See also
    ///
    /// * [`round`] - Rounds to the nearest whole number
    /// * [`floor`] - Rounds down to the nearest whole number
    ///
    pub fn ceil(mut self) -> Self {
        let current = self.current;
        self.previous.push(current);
        self.current = current.ceil();
        self
    }

    /// Returns a new Builder with the current value rounded down to the nearest whole number.
    ///
    /// # Example
    ///
    /// ```
    /// use num_builder::Builder;
    ///
    /// let result = Builder::new(2.6).floor().build(); // 2.0
    /// ```
    ///
    /// # See also
    ///
    /// * [`round`] - Rounds to the nearest whole number
    /// * [`ceil`] - Rounds up to the nearest whole number
    ///
    pub fn floor(mut self) -> Self {
        let current = self.current;
        self.previous.push(current);
        self.current = current.floor();
        self
    }
}

impl<T: Num + Copy> PartialEq for Builder<T> {
    fn eq(&self, other: &Self) -> bool {
        self.current == other.current
    }
}

impl<T> Display for Builder<T>
where
    T: ryu::Float,
{
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), Error> {
        let mut buffer = Buffer::new();
        let value = buffer.format(self.current);
        write!(formatter, "{}", value)
    }
}

/*
🧪 Tests --------------------------------------------------------------------------- /
*/

#[cfg(test)]
mod tests {
    use super::Builder;

    #[test]
    fn it_builds_floats_with_simple_operations() {
        let builder = Builder::new(23.0)
            .add(5.0)
            .subtract(1.0)
            .mulitply_by(4.0)
            .divide_by(2.0);
        assert_eq!(builder.all().len(), 5);
        let output = builder.build();
        assert_eq!(output, 54.0);
    }

    #[test]
    fn it_builds_integers_with_simple_operations() {
        let builder = Builder::new(23)
            .add(5)
            .subtract(1)
            .mulitply_by(4)
            .divide_by(2);
        assert_eq!(builder.all().len(), 5);
        let output = builder.build();
        assert_eq!(output, 54);
    }

    #[test]
    fn it_can_apply_modulo_to_floats() {
        assert_eq!(Builder::new(4.0).modulo(2.0).build(), 0.0);
        assert_eq!(Builder::new(4.0).modulo(3.0).build(), 1.0);
        assert_eq!(Builder::new(5.0).modulo(3.0).build(), 2.0);
    }

    #[test]
    fn it_can_apply_modulo_to_integers() {
        assert_eq!(Builder::new(4).modulo(2).build(), 0);
        assert_eq!(Builder::new(4).modulo(3).build(), 1);
        assert_eq!(Builder::new(5).modulo(3).build(), 2);
    }

    #[test]
    fn it_can_round_floats() {
        let output = Builder::new(2.1).round().build();
        assert_eq!(output, 2.0);
    }

    #[test]
    fn it_can_round_floats_up() {
        let output = Builder::new(2.1).ceil().build();
        assert_eq!(output, 3.0);
    }

    #[test]
    fn it_can_round_floats_down() {
        let output = Builder::new(2.6).floor().build();
        assert_eq!(output, 2.0);
    }
}
