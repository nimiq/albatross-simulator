use core::cmp::PartialOrd;
use core::fmt;
use std::error::Error;
use std::marker::PhantomData;

use rand::distributions::{Distribution, WeightedError, WeightedIndex};
use rand::distributions::uniform::{SampleBorrow, SampleUniform};
use rand::Rng;

/// Random number distribution that produces values that are uniformly
/// distributed over each of a sequence of contiguous subintervals.
/// This mimics the behaviour of
/// http://www.cplusplus.com/reference/random/piecewise_constant_distribution/
///
/// This distribution is built on a WeightedIndex and Uniform distribution.
pub struct PiecewiseConstant<W: SampleUniform + PartialOrd, I: SampleUniform, J: SampleBorrow<I>> {
    weighted_index: WeightedIndex<W>,
    intervals: Vec<J>,
    sample_type: PhantomData<I>,
}

impl<W: SampleUniform + PartialOrd, I: SampleUniform, J: SampleBorrow<I>> PiecewiseConstant<W, I, J> {
    /// Creates a new a `PiecewiseConstant` [`Distribution`] using the intervals in `intervals`
    /// and the weights in `weights`. The weights can use any type `X` for which an
    /// implementation of [`Uniform<X>`] exists. The intervals are of type `Y`.
    ///
    /// Returns an error if the size of `intervals` does not match the size of `weights + 1`,
    /// the iterator is empty, if any weight is `< 0`, or if its total value is 0.
    ///
    /// [`Uniform<X>`]: rand::distributions::uniform::Uniform
    pub fn new<W1, I1>(weights: W1, intervals: I1) -> Result<PiecewiseConstant<W, I, J>, PiecewiseConstantError>
        where W1: IntoIterator,
              W1::Item: SampleBorrow<W>,
              I1: IntoIterator<Item=J>,
              W: for<'a> ::core::ops::AddAssign<&'a W> +
              Clone +
              Default,
              I: SampleUniform,
              J: SampleBorrow<I> {
        let weights: Vec<_> = weights.into_iter().collect();
        let intervals: Vec<J> = intervals.into_iter().collect();

        if weights.len() + 1 != intervals.len() {
            return Err(PiecewiseConstantError::InvalidSize);
        }

        let weighted_index = WeightedIndex::new(weights)
            .map_err(PiecewiseConstantError::WeightedError)?;

        Ok(PiecewiseConstant { weighted_index, intervals, sample_type: PhantomData })
    }
}

impl<W, I, J> Distribution<I> for PiecewiseConstant<W, I, J> where
    W: SampleUniform + PartialOrd,
    I: SampleUniform,
    J: SampleBorrow<I> + Clone {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> I {
        let index = self.weighted_index.sample(rng);
        rng.gen_range(self.intervals[index].clone(), self.intervals[index + 1].clone())
    }
}

impl<W, I, J> Clone for PiecewiseConstant<W, I, J> where
    W: SampleUniform + PartialOrd,
    I: SampleUniform,
    J: SampleBorrow<I> + Clone,
    WeightedIndex<W>: Clone {
    fn clone(&self) -> Self {
        PiecewiseConstant {
            weighted_index: self.weighted_index.clone(),
            intervals: self.intervals.clone(),
            sample_type: PhantomData,
        }
    }
}

/// Error type returned from `PiecewiseConstantError::new`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PiecewiseConstantError {
    WeightedError(WeightedError),
    InvalidSize,
}

impl Error for PiecewiseConstantError {
    fn description(&self) -> &str {
        match *self {
            PiecewiseConstantError::WeightedError(ref e) => e.description(),
            PiecewiseConstantError::InvalidSize => "Number of intervals must be number of weights + 1",
        }
    }
    fn cause(&self) -> Option<&Error> {
        match *self {
            PiecewiseConstantError::WeightedError(ref e) => Some(e),
            PiecewiseConstantError::InvalidSize => None,
        }
    }
}

impl fmt::Display for PiecewiseConstantError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}