# HF Shirou Attack Atlases

These images are production attack atlases for the HF Shirou action module. Runtime combat uses the Codex/model-generated v2 ground-light row atlases, while the other attack modules continue to use their full multi-row reference boards.

## Runtime Sheets

- `hf_shirou_overedge_light_combo_sheet.png`: 11 frames. `V` uses frames 0-2 for release startup, light combo uses frames 2-4, 5-7, and 8-10.
- `hf_shirou_overedge_heavy_combo_sheet.png`: 17 frames. `K` plays the full heavy combo sheet.

## Runtime Attack Atlases

- `v2_generated/rows/ground_light_v2_r01.png` through `v2_generated/rows/ground_light_v2_r05.png`: Codex/model-generated 8-frame ground-light row atlases.
- Heavy, air combo, mobility, ninjutsu, ultimate, and weapon projection runtime animation is selected from the multi-row reference boards below.

## Source And Preview Boards

- `hf_shirou_attack_ground_light_reference.png`: 8x5 ground light rows.
- `hf_shirou_attack_heavy_reference.png`: 8x5 heavy rows.
- `hf_shirou_attack_air_combo_reference.png`: 8x5 airborne combo rows.
- `hf_shirou_attack_mobility_reference.png`: 6x4 dash, evade, substitution, wall movement.
- `hf_shirou_attack_ninjutsu_projectiles_reference.png`: 8x4 fire, wind, lightning, shadow clone skills.
- `hf_shirou_attack_ultimate_reference.png`: 8x3 ultimate techniques.
- `hf_shirou_attack_weapon_projection_reference.png`: 6x4 short sword, twin blade, long sword, bow projection.
- `v2_generated/hf_shirou_ground_light_v2_reference.png`: Codex/model-generated ground-light source board. The runtime `hf_shirou_attack_ground_light_reference.png` mirrors this image.

## Sprite Split And Row Plan

- `hf_shirou_attack_rows.json`: authoritative row plan for every playable attack atlas. It records the image path, 256x256 grid, row count, input mapping, runtime `AttackAnimationStyle`, and gameplay role.
- `scripts/split_hf_shirou_attack_atlases.py`: reproducibly splits every atlas into standalone frame PNGs under `tmp/hf_shirou_attack_frames` and writes a split manifest with frame bounding boxes and edge-contact diagnostics.
- `scripts/audit_hf_shirou_attack_atlases.py`: writes checkerboard contact sheets under `tmp/hf_shirou_attack_audit` and reports edge contact, possible alpha holes, and green-key residue.
- All runtime atlas cells are 256x256 with zero offset. The current transparent PNGs are the assets loaded by Bevy; generation scratch files should stay outside the repo.

Run:

```bash
python3 scripts/split_hf_shirou_attack_atlases.py --dry-run --fail-on-edge-contact
python3 scripts/audit_hf_shirou_attack_atlases.py --out tmp/hf_shirou_attack_audit --fail-on-issues
```

## Overview Boards

- `hf_shirou_attack_modules_overview.png`: full playable module overview.
- `hf_shirou_advanced_attack_modules_overview.png`: advanced move overview using the new runtime atlases.

## Runtime Mapping

- `Shift+V`: enables the reference module mode and shows the overview board.
- `J/Z/L`: light attack; airborne uses air combo rows; crouching cycles the stable dash/slide mobility rows.
- `K`: heavy attack; crouch+K cycles ultimate rows; Overedge release still uses the Nanobanana heavy combo sheet.
- `X`: cycles fire/wind/lightning ninjutsu rows; `Shift+X` casts the shadow clone semantic using a stable player-body row plus clone afterimages; crouch+X cycles weapon projection rows.
- `Y/U/I/O/P`: direct light rows 1-5, or air rows 1-5 while airborne.
- `Crouch + Y/U/I/O/P`: direct mobility rows 1-4 for dash, slide, substitution, and wall movement; substitution/wall actions use stable player-body rows plus afterimages, while normal crouch light still cycles only stable dash/slide rows.
- `Shift+Y/U/I/O/P`: direct heavy rows 1-5.
