# Sakura 2P Attack Atlases

These are the production attack atlases used by Sakura's image-sequence player.
They preserve the existing movement-image workflow and switch only attacks to a
fixed `256x256` Bevy texture atlas.

## Runtime sheets

- `sakura_attack_ground_light.png`: 8x5 ground-light rows.
- `sakura_attack_heavy.png`: 8x5 heavy rows.
- `sakura_attack_air_combo.png`: 8x5 airborne rows.
- `sakura_attack_mobility.png`: 6x4 dash, slide, substitution, and wall movement.
- `sakura_attack_ninjutsu.png`: 8x4 cursed flame, petal wave, lightning, and shadow bind.
- `sakura_attack_ultimate.png`: 8x3 ultimate techniques.
- `sakura_attack_weapon_projection.png`: 6x4 kodachi, dual blade, odachi, and bow forms.

All frames have transparent gutters and exactly one solid Sakura body. Motion
streaks and petals are effects, not duplicate player entities.

## Validation

```bash
python3 scripts/split_hf_shirou_attack_atlases.py \
  --spec assets/images/characters/sakura_attack_rows.json \
  --dry-run --fail-on-edge-contact
python3 scripts/audit_hf_shirou_attack_atlases.py \
  --spec assets/images/characters/sakura_attack_rows.json \
  --out tmp/sakura_attack_audit --fail-on-issues
```
