# HF Shirou Attack Module Design

This document pins the three requested production steps to concrete assets and
runtime code.

## 1. Split Images

All repaired runtime attack atlases live in
`assets/images/characters/reference/` as transparent PNGs. They use a fixed
256x256 cell size and zero offset.

Use the split tool when individual frames need inspection:

```bash
python3 scripts/split_hf_shirou_attack_atlases.py --dry-run --fail-on-edge-contact
python3 scripts/split_hf_shirou_attack_atlases.py --out tmp/hf_shirou_attack_frames
python3 scripts/audit_hf_shirou_attack_atlases.py --out tmp/hf_shirou_attack_audit --fail-on-issues
```

The tool reads
`assets/images/characters/reference/hf_shirou_attack_rows.json`, validates each
atlas dimension, exports `r##_f##.png` frames, and writes
`tmp/hf_shirou_attack_frames/manifest.json` with alpha bounding boxes and
edge-contact diagnostics. The audit tool also writes checkerboard contact sheets
for visual review and separates blocking edge/key-residue problems from normal
energy-ring negative-space warnings.

## 2. Sprite Grid, Inputs, Combos

The row plan is authoritative in
`assets/images/characters/reference/hf_shirou_attack_rows.json`.

| Atlas | Grid | Runtime rows | Main inputs |
| --- | ---: | --- | --- |
| Ground light | 8x5 | `GroundLightRow(1..5)` | `J/Z/L` cycles rows 1-5, `Y/U/I/O/P` selects rows 1-5 on ground |
| Heavy | 8x5 | `HeavyRefRow(1..5)` | `K` cycles rows 1-5, `Shift+Y/U/I/O/P` selects rows 1-5 |
| Air combo | 8x5 | `AirComboRow(1..5)` | `J/Z/L` cycles rows 1-5 airborne, `Y/U/I/O/P` selects rows 1-5 airborne |
| Mobility | 6x4 | `MobilityRefRow(1..4)` | crouch light cycles rows 1-2, crouch `Y/U/I/O/P` selects rows 1-4 |
| Ninjutsu | 8x4 | `NinjutsuRefRow(1..4)` | `X` cycles rows 1-3, `Shift+X` uses row 4 semantic |
| Ultimate | 8x3 | `UltimateRefRow(1..3)` | crouch `K` cycles rows 1-3 |
| Weapon projection | 6x4 | `WeaponProjRefRow(1..4)` | crouch `X` cycles rows 1-4 |

Special generated rows that contain multiple bodies are not played as literal
player-body frames. Substitution, wall movement, and shadow clone rows use a
stable single-body row plus abstract afterimage/VFX entities so the game never
duplicates the player or sinks the body into the floor.

## 3. Runtime Assembly

The attack module is wired through these code paths:

- `src/asset_paths.rs`: atlas paths and grid constants.
- `src/plugins/core.rs`: Bevy asset handles and `TextureAtlasLayout` setup.
- `src/components/animation.rs`: `AttackAnimationStyle` variants and sheet selection.
- `src/systems/combat.rs`: input resolution, combo cycling, attack movement, hitboxes, windup, projectile rows, afterimages, and special-row stabilization.
- `src/systems/sprite_animation.rs`: atlas row frame selection for each attack style.
- `src/systems/attack_modules.rs`: in-game reference preview boards.

Validation coverage:

- `src/tests/reference_attack_assets_tests.rs` verifies the row plan, atlas
  dimensions, runtime asset paths, row counts, and total frame count.
- `src/tests/systems_tests.rs` covers key-to-row selection, reference attack
  chaining, mobility momentum, heavy feedback strength, projectile windup, and
  no-extra-player behavior for clone/substitution rows.
- `src/systems/attack_modules.rs` unit tests cover preview reachability and
  runtime grid availability.
