# Week 10 MuJoCo Visual Mirror Summary

## Goal

Week 10 added a read-only MoveIt snapshot boundary and a SO-101 MuJoCo visual mirror while preserving the architecture rule that Studio mirrors moveit-side state and does not own simulation or motion execution.

## Completed Work

- Added `GET /api/moveit/snapshot`.
- Added backend snapshot models for robot config, freshness, joint state, end-effector pose, planning scene, trajectory status, and visual model metadata.
- Added deterministic fallback/demo snapshot data for the SO-101-style robot family.
- Added frontend API types and `getMoveitSnapshot()`.
- Updated Motion Planner to render read-only snapshot data.
- Added a CSS-based SO-101 MuJoCo visual mirror.
- Kept Plan / Execute / Stop disabled.
- Updated README API and demo walkthrough documentation.

## Validation Completed

```bash
cargo fmt --manifest-path backend/Cargo.toml --check
cargo test --manifest-path backend/Cargo.toml
npm --prefix frontend run build
```

## Manual Browser Smoke Test

- Motion Planner shows the SO-101 MuJoCo visual mirror.
- Snapshot joints, pose, scene objects, and trajectory status render from the backend or fallback data.
- The UI clearly labels the viewport as a moveit/MuJoCo-owned mirror.
- Plan / Execute / Stop remain disabled.
- Visualization still follows the dviz display stack direction.
- Light and dark modes are readable.

## Known Limitations

- Snapshot data is deterministic demo/fallback data.
- Studio does not run MuJoCo in the browser.
- Studio does not issue planning, execute, or stop commands.
- No live Zenoh streaming or binary visualization payload forwarding is implemented.
- No Rerun iframe or full web-viewer embedding is implemented.
