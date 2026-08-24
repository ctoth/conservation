#![forbid(unsafe_code)]

//! Exact left-nullspace derivation for rational transition matrices.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

use conservation_core::{AxisId, BalanceLaw, BalanceLawError, KindId, Provenance};
use num_bigint::BigInt;
use num_integer::Integer;
use num_rational::BigRational;
use num_traits::{One, Signed, Zero};

/// Identifies which matrix interpretation produced a nullspace law.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NullspaceSource {
    /// The matrix is a directed incidence matrix.
    Incidence,
    /// The matrix is a stoichiometric matrix.
    Stoichiometric,
}

impl NullspaceSource {
    fn provenance(self) -> Provenance {
        match self {
            Self::Incidence => Provenance::IncidenceNullspace,
            Self::Stoichiometric => Provenance::StoichiometricNullspace,
        }
    }
}

/// A row-major exact transition matrix whose rows correspond to axes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransitionMatrix {
    axes: Vec<AxisId>,
    entries: Vec<Vec<BigRational>>,
    transition_count: usize,
}

/// A structural error in a transition matrix.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MatrixError {
    /// A matrix has no axis rows.
    NoAxes,
    /// A matrix has no transition columns.
    NoTransitions,
    /// The number of matrix rows differs from the number of axes.
    AxisRowCount { axes: usize, rows: usize },
    /// Two rows do not contain the same number of transitions.
    RaggedRows {
        row: usize,
        expected: usize,
        actual: usize,
    },
    /// An axis identifier occurs more than once.
    DuplicateAxis(AxisId),
}

impl fmt::Display for MatrixError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoAxes => formatter.write_str("matrix must contain at least one axis"),
            Self::NoTransitions => {
                formatter.write_str("matrix must contain at least one transition")
            }
            Self::AxisRowCount { axes, rows } => {
                write!(formatter, "matrix has {axes} axes but {rows} rows")
            }
            Self::RaggedRows {
                row,
                expected,
                actual,
            } => write!(
                formatter,
                "matrix row {row} has {actual} entries; expected {expected}"
            ),
            Self::DuplicateAxis(axis) => write!(formatter, "duplicate matrix axis {axis}"),
        }
    }
}

impl Error for MatrixError {}

impl TransitionMatrix {
    /// Validates and constructs a row-major transition matrix.
    pub fn new(
        axes: impl IntoIterator<Item = AxisId>,
        entries: Vec<Vec<BigRational>>,
    ) -> Result<Self, MatrixError> {
        let axes = axes.into_iter().collect::<Vec<_>>();
        if axes.is_empty() {
            return Err(MatrixError::NoAxes);
        }
        if axes.len() != entries.len() {
            return Err(MatrixError::AxisRowCount {
                axes: axes.len(),
                rows: entries.len(),
            });
        }

        let mut unique = BTreeSet::new();
        for axis in &axes {
            if !unique.insert(axis) {
                return Err(MatrixError::DuplicateAxis(axis.clone()));
            }
        }

        let transition_count = entries.first().map_or(0, Vec::len);
        for (row, values) in entries.iter().enumerate() {
            if values.len() != transition_count {
                return Err(MatrixError::RaggedRows {
                    row,
                    expected: transition_count,
                    actual: values.len(),
                });
            }
        }
        if transition_count == 0 {
            return Err(MatrixError::NoTransitions);
        }

        let mut rows_by_axis = axes.into_iter().zip(entries).collect::<Vec<_>>();
        rows_by_axis.sort_by(|(left, _), (right, _)| left.cmp(right));
        let (axes, entries) = rows_by_axis.into_iter().unzip();

        Ok(Self {
            axes,
            entries,
            transition_count,
        })
    }

    /// Returns the ordered axes represented by the rows.
    pub fn axes(&self) -> &[AxisId] {
        &self.axes
    }

    /// Returns the number of transition columns.
    pub fn transition_count(&self) -> usize {
        self.transition_count
    }
}

/// Derives a deterministic rational vector-space basis for the left nullspace.
///
/// Each returned rational basis vector is scaled to primitive integer
/// coefficients for stable presentation. The result is not an integer-lattice
/// basis and makes no claim to span every integer solution by integer multiples.
pub fn derive_left_nullspace(
    matrix: &TransitionMatrix,
    kind: KindId,
    source: NullspaceSource,
) -> Result<Vec<BalanceLaw>, BalanceLawError> {
    let mut transpose = vec![vec![BigRational::zero(); matrix.axes.len()]; matrix.transition_count];
    for (axis_index, row) in matrix.entries.iter().enumerate() {
        for (transition_index, value) in row.iter().enumerate() {
            transpose[transition_index][axis_index] = value.clone();
        }
    }

    nullspace_basis(transpose, matrix.axes.len())
        .into_iter()
        .map(|vector| {
            BalanceLaw::new(
                kind.clone(),
                matrix.axes.iter().cloned().zip(vector),
                source.provenance(),
            )
        })
        .collect()
}

fn nullspace_basis(mut rows: Vec<Vec<BigRational>>, column_count: usize) -> Vec<Vec<BigRational>> {
    let mut pivot_columns = Vec::new();
    let mut pivot_row = 0;

    for column in 0..column_count {
        let Some(candidate) = (pivot_row..rows.len()).find(|&row| !rows[row][column].is_zero())
        else {
            continue;
        };
        rows.swap(pivot_row, candidate);

        let pivot = rows[pivot_row][column].clone();
        for value in &mut rows[pivot_row] {
            *value /= &pivot;
        }

        let normalized_pivot_row = rows[pivot_row].clone();
        for (row_index, row) in rows.iter_mut().enumerate() {
            if row_index == pivot_row || row[column].is_zero() {
                continue;
            }
            let factor = row[column].clone();
            for (value, pivot_value) in row.iter_mut().zip(&normalized_pivot_row) {
                *value -= &factor * pivot_value;
            }
        }

        pivot_columns.push(column);
        pivot_row += 1;
        if pivot_row == rows.len() {
            break;
        }
    }

    let pivots = pivot_columns.iter().copied().collect::<BTreeSet<_>>();
    (0..column_count)
        .filter(|column| !pivots.contains(column))
        .map(|free_column| {
            let mut vector = vec![BigRational::zero(); column_count];
            vector[free_column] = BigRational::one();
            for (row, pivot_column) in pivot_columns.iter().copied().enumerate() {
                vector[pivot_column] = -rows[row][free_column].clone();
            }
            primitive_integer_vector(vector)
        })
        .collect()
}

fn primitive_integer_vector(vector: Vec<BigRational>) -> Vec<BigRational> {
    let denominator_lcm = vector
        .iter()
        .fold(BigInt::one(), |lcm, value| lcm.lcm(value.denom()));
    let mut integers = vector
        .iter()
        .map(|value| value.numer() * (&denominator_lcm / value.denom()))
        .collect::<Vec<_>>();
    let divisor = integers
        .iter()
        .fold(BigInt::zero(), |gcd, value| gcd.gcd(&value.abs()));
    if !divisor.is_zero() {
        for value in &mut integers {
            *value /= &divisor;
        }
    }
    if integers
        .iter()
        .find(|value| !value.is_zero())
        .is_some_and(Signed::is_negative)
    {
        for value in &mut integers {
            *value = -value.clone();
        }
    }
    integers
        .into_iter()
        .map(BigRational::from_integer)
        .collect()
}
