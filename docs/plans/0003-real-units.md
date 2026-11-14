# Plan 0003: Real units and parameters

- **Roadmap step:** 3
- **Date:** 14 Nov 2026
- **Status:** built (14 Nov 2026)
- **Sources:** decisions with the author, 14 Nov 2026; elm research notes (`~/claude-workspace/tree-simulator/research/elm-parameters.md`, Haiku web research, mostly medium/low confidence)

## Goal

Structure first, numbers later: the simulation works in metres and hours, every tunable lives in one struct with its unit and source, and values are estimates until roadmap step 14 calibrates them (author, 14 Nov 2026).

## Decisions

| ID | Decision | Value | Source |
|---|---|---|---|
| D12 | One step = 1 simulated hour | `SIM_DT` = 1 h | Author |
| D13 | One section = one botanical internode (ADR-0005) | 3 cm (estimate) | Author |
| D14 | Simulation in metres; only drawing converts | 200 px/m at zoom 1× | Proposed, accepted |
| D15 | All tunables in `GrowthParams` with unit, source or "estimate" | — | Proposed, accepted |
| D16 | No seasons yet: a year's growth spread evenly over the year | 0.40 m/year (estimate) | Proposed, accepted |
| D17 | Fork cadence = nodes per yearly shoot | 12 ± 3 (U. glabra, medium) | Proposed, accepted |
| D18 | Speed = simulated hours per real second | 1× = 300 h/s | Proposed, accepted |
| D19 | Bending rates per hour, scaled to keep the bend per internode | Derived | Proposed, accepted |
| D20 | Trunk, minimum internode, leaf in metres | 5 cm, 0.5 cm, 8 × 5 cm (estimates) | Proposed, accepted |

Guide species: elm (single trunk, distichous, sympodial, UK native); hazel, then hornbeam and lime, as fallbacks where elm data is thin.

## Built

- `src/params.rs`: `GrowthParams` and `HOURS_PER_YEAR`.
- `GrowthSection` loses its per-section rate fields; `update_physics` takes the params.
- `TreeSimulation::new(start, seed)` / `with_params`; `hours` counter; `SIM_DT` = 1 h.
- `main.rs`: world in metres, `PX_PER_M`, age shown on screen, leaves sized from params, speed 300 h/s at 1×.
- Example and invariant tests run up to 220,000 steps (~25 years).
- `ANCESTOR_SKIP_DEPTH` 10 → 20 to cover 12 ± 3 internodes between forks.

## Measurements (seeds 1–3, release, 14 Nov 2026)

Trees finish in 4.8–7.6 simulated years, 1.9–2.9 m tall, 1.0–1.4 m crown, 1,077–2,713 sections. Mean step 220 µs, slowest 1.5 ms (was 66 µs and 0.66 ms before, mostly from the deeper ancestor skip). At top speed (1,500 h/s, 25 steps per frame) larger trees will slow down.

## Quick review

- No spring flush until seasons (step 12); yearly totals are right, timing within a year isn't.
- Collision clearance now uses real thicknesses; check visually that branches don't crowd or stall oddly.
- Step time tripled; the performance steps (6–7) may need to come before the light model if it gets worse.
