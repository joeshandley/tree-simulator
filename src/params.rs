//! Every tunable growth parameter, in SI units: metres, hours, radians (plan 0003).
//!
//! Each field states its unit and where its value comes from. "Estimate" means
//! no measured source yet: values are placeholders until roadmap step 14
//! calibrates them against real elm. Guide species: elm (Ulmus), with hazel,
//! hornbeam and lime as fallbacks (research notes:
//! `~/claude-workspace/tree-simulator/research/elm-parameters.md`, 14 Nov 2026).

/// Hours in a year. Until seasons exist (roadmap step 12), a year's growth is
/// spread evenly over all of them.
pub const HOURS_PER_YEAR: f32 = 8_760.0;

#[derive(Debug, Clone)]
pub struct GrowthParams {
    /// Length of one internode, node to node (m). One section is one internode
    /// (ADR-0005). Estimate: elm internodes are a few centimetres.
    pub internode_length: f32,
    /// Extension rate of a growing tip at full light (m per hour). Estimate:
    /// 0.40 m of leader growth per year, spread over the whole year (no seasons yet).
    pub growth_rate: f32,
    /// Trunk diameter at the start of the simulation (m). Estimate for a young sapling.
    pub trunk_thickness: f32,
    /// Below this target length a forked child is not viable and the axis ends (m).
    /// Kept at 1/6 of `internode_length`, the ratio tuned before units were real,
    /// so several fork generations fit before termination. Estimate.
    pub min_viable_internode_length: f32,
    /// Angle between a lateral and its parent at a fork (rad). Tuned, not measured.
    pub branch_angle_spread: f32,
    /// Combined leader + lateral length budget per fork (dimensionless). Raised
    /// from 0.7 so the leader survives more fork generations before the length
    /// floor, which caps the whole tree's size. Tuned.
    pub length_decay: f32,
    /// Thickness multiplier per fork (dimensionless). ~1/√2 per Leonardo's rule
    /// (a split in two conserves cross-section area). Replaced by the pipe model
    /// at roadmap step 10.
    pub thickness_decay: f32,
    /// Leader's share of a fork's thickness budget; 0.5 is symmetric. Models
    /// sympodial apical dominance (ADR-0003). Tuned.
    pub dominance_ratio: f32,
    /// How much of `dominance_ratio`'s asymmetry applies to length (0 = none,
    /// 1 = full). Full asymmetry starves laterals at the length floor after one
    /// or two forks (ADR-0003). Tuned.
    pub length_dominance_blend: f32,
    /// Internodes between forks along an axis, so one sympodial fork per yearly
    /// shoot. Source: Ulmus glabra, 10–15 leaves per shoot (bud-scale morphology
    /// abstract; medium confidence).
    pub fork_interval: u32,
    /// ± range on `fork_interval` so forking isn't metronomic. Covers the 10–15 range.
    pub fork_jitter: u32,
    /// Upward bending rate (rad per hour per unit torque). Derived from the
    /// per-second tuning before real units, scaled to keep the same bend per
    /// internode. Estimate.
    pub gravitropism_rate: f32,
    /// Bending rate towards light (rad per hour per unit torque). Derived like
    /// `gravitropism_rate`. Estimate.
    pub phototropism_rate: f32,
    /// Thickness (m) at which a section's flexibility is 1. Thinner sections bend
    /// more, thicker less (clamped 0.1–2). Scaled from the pre-units tuning. Estimate.
    pub flexibility_reference_thickness: f32,
    /// Thickness below which flexibility stops increasing (m). Estimate.
    pub min_flexibility_thickness: f32,
    /// Direction to the sun (rad, screen coordinates: −π/2 is straight up).
    pub sun_azimuth: f32,
    /// Leaf blade length and width (m), for drawing. Estimate for elm.
    pub leaf_length: f32,
    pub leaf_width: f32,
}

impl Default for GrowthParams {
    fn default() -> Self {
        let internode_length = 0.03;
        let growth_rate = 0.40 / HOURS_PER_YEAR;
        // Before real units: 12 px internodes at 50 px/s took 0.24 s each, with
        // bending weights 1.0 (gravity) and 0.5 (light) per second. Scale the
        // weights so each internode still bends by the same amount.
        let hours_per_internode = internode_length / growth_rate;
        let bend_scale = 0.24 / hours_per_internode;
        // Before real units the trunk was 20 px thick; 0.05 m now, so 400 px per m.
        let old_px_per_m = 20.0 / 0.05;
        GrowthParams {
            internode_length,
            growth_rate,
            trunk_thickness: 0.05,
            min_viable_internode_length: internode_length / 6.0,
            branch_angle_spread: 40.0_f32.to_radians(),
            length_decay: 0.75,
            thickness_decay: 0.707,
            dominance_ratio: 0.6,
            length_dominance_blend: 0.5,
            fork_interval: 12,
            fork_jitter: 3,
            gravitropism_rate: 1.0 * bend_scale,
            phototropism_rate: 0.5 * bend_scale,
            flexibility_reference_thickness: 1.5 / old_px_per_m,
            min_flexibility_thickness: 0.5 / old_px_per_m,
            sun_azimuth: crate::vector::ANGLE_UP,
            leaf_length: 0.08,
            leaf_width: 0.05,
        }
    }
}
