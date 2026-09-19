//! Exact, indivisible exchanges over one authoritative state root.
#![forbid(unsafe_code)]

mod engine;
mod expression;
mod model;

pub use engine::{Engine, Participation, Prepared, Receipt};
pub use expression::{Constraint, Expr, Relation};
pub use model::{
    Capacity, Dimension, Domain, Error, Exchange, Fact, Law, Model, Placement, Quantity,
    RecordWrite, Stock,
};
