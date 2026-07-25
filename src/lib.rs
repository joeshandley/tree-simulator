pub mod branch;
pub mod simulation;
pub mod vector;

// Convenient re-exports so users don't have to write `use tree_sim::simulation::TreeSimulation;`
pub use branch::{Branch, BranchState};
pub use simulation::TreeSimulation;
pub use vector::Vector2D;
