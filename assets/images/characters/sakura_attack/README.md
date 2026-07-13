# Sakura 2P Attack Image Sequences

Sakura uses standalone images for both movement and attacks. The current JK
redesign has a white pleated skirt, opaque black tights, long violet hair and a
red side ribbon. Seven generated module sheets under `../sakura_jk/modules/`
are split into the 224 `256x256` PNG files under `frames/`. Sakura never receives
a Bevy `TextureAtlas` component.

Base movement is independent from combat: `../sakura_jk/base_movement_v3.png`
builds eight dedicated idle/run/jump/crouch images under
`../sakura_jk/base_frames/`. The build records each sheet's source direction,
normalizes it once into the runtime's canonical right-facing orientation, and
locks every base pose to the same foot baseline so state changes do not jump.

## Legacy source sheets and runtime groups

- `sakura_attack_ground_light.png`: 8x5 ground-light rows.
- `sakura_attack_heavy.png`: 8x5 heavy rows.
- `sakura_attack_air_combo.png`: 8x5 airborne rows.
- `sakura_attack_mobility.png`: 6x4 dash, slide, substitution, and wall movement.
- `sakura_attack_ninjutsu.png`: 8x4 cursed flame, petal wave, lightning, and shadow bind.
- `sakura_attack_ultimate.png`: 8x3 ultimate techniques.
- `sakura_attack_weapon_projection.png`: 6x4 kodachi, dual blade, odachi, and bow forms.

All frames have transparent gutters and exactly one solid Sakura body. Motion
streaks and petals are effects, not duplicate player entities.

The active runtime groups are `frames/ground_light`, `frames/heavy`,
`frames/air_combo`, `frames/mobility`, `frames/ninjutsu_projectiles`,
`frames/ultimate`, and `frames/weapon_projection`.

## Validation

```bash
python3 scripts/build_sakura_jk_attack_frames.py
cargo test --lib sakura_attack_plan_has_all_standalone_runtime_images
```
