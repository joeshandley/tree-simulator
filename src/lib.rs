pub mod growth_section;
pub mod simulation;
pub mod vector;

// Convenient re-exports so users don't have to write `use tree_sim::simulation::TreeSimulation;`
pub use growth_section::{GrowthSection, GrowthSectionState};
pub use simulation::TreeSimulation;
pub use vector::Vector2D;
