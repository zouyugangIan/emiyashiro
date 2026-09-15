# Generation provenance

Tool: built-in `image_gen` (two calls). Source PNGs copied unchanged into this directory.

## Locomotion

Use case: identity-preserve.
Asset type: production transparent PNG 2D side-scrolling game locomotion sprite atlas.
Input image 1 (core sheet) and image 2 (run sheet) are EDIT TARGETS with damaged transparency and stiff incomplete motions. Input image 3 (ground light attack row) is the AUTHORITATIVE character identity, clothing, proportions and painted anime sprite style reference.
Primary request: replace the damaged core/run movements with ONE cohesive movement sprite sheet of HF Shirou matching image 3: short red hair, brown open jacket, white shirt, navy jeans, brown shoes, red-black glowing left forearm. Face and move towards the RIGHT. Draw complete opaque body interiors, intact shirt, jeans, both shoes and hands; no holes, fragmented limbs, cut-off weapons or body parts.
Composition: exactly 1024x1024 pixels, invisible regular 4 columns x 4 rows of 256x256 pixels each. EXACTLY 16 separate full-body figures, one in each cell, never straddling a cell. Horizontal hip anchor at cell x=128, head approximately y=52 for standing figures, standing sole baseline y=236. Keep all body/effects inside x=22..234 and y=20..240 of each cell, with transparent gutters. Identical character scale in every cell, no automatic zoom to fill space.
Row 1 (cells 0,1,2,3): four subtle relaxed ready idle breathing poses, facing right three-quarter, grounded feet on same baseline.
Row 2 (cells 4,5,6,7): first half of a RUN cycle, four DISTINCT sequential poses: right foot contact, right leg compression, left leg passing, airborne extension.
Row 3 (cells 8,9,10,11): second half of SAME RUN cycle: left foot contact, left leg compression, right leg passing, airborne extension returning smoothly to row 2 cell 1. Eight complete and anatomically coherent poses. Preserve constant body size and moderate forward lean.
Row 4 (cells 12,13,14,15): jump takeoff with extending legs, airborne apex with knees tucked, descending with feet prepared below body, compact grounded crouch/landing with both feet intact. Keep hip x anchor consistent.
Style: crisp high-quality hand-painted anime game sprites, clean silhouette, restrained shading, consistent lighting matching reference 3, readable at 160px display size.
Background: genuine alpha transparency, NOT a visible checkerboard, no black or white filled backdrop.
Constraints: no text, no labels, no numbers, no grid lines, no shadows beyond feet, no extra character, no detached body fragments, no VFX covering torso, no transparency inside solid clothing. One 4x4 sheet only.

## Ground thrust and low sweep

Use case: precise-object-edit.
Asset type: production transparent 2D side-scroller sprite sheet replacing TWO damaged HF Shirou attacks.
Input image 1 is the edit target/source for the character style and the damaged thrust/low sweep moves in its rows 3 and 4. Input image 2 is the clean identity reference.
Create ONE square atlas with EXACTLY 16 separate complete character poses, in a uniform 4 columns by 4 rows arrangement, one figure in every cell with very wide clear transparent gutters. Keep each figure fully inside its own cell, centered at the same horizontal hip origin and grounded sole baseline in every frame. All frames face and attack RIGHT. Character scale is identical in all 16 cells.
Rows 1 and 2 together are an 8-frame grounded FORWARD THRUST: guarded anticipation, draw glowing red-black forearm back, thrust forward, full extension with a short intact red blade, impact held with small red spark, deceleration, retract, return to guard. Keep both feet and body intact and grounded; no detached boot, no separate floating blade tip.
Rows 3 and 4 together are an 8-frame LOW HORIZONTAL SWEEP: crouch guard, coil, sweep forward near knee height, full sweeping red arc, follow through, settle weight, lift to ready, return to guard. Clean body fully visible through all frames; red arc stays compact and away from cell boundaries.
Identity: red-haired anime young man, brown jacket, white shirt, navy jeans, brown shoes, red-black glowing arm, same as clean reference.
Style: detailed sharp painted anime game sprite matching provided reference. Black outlines and subtle cel shading. A complete consistent person in each cell. Complete opaque clothing and skin, NO accidental transparent holes.
Output actual transparent PNG background, square 4x4 sheet, no checker pattern, no labels, no text, no lines, no scenery, no detached effects or extra characters. Leave at least 12% of each cell dimension clear all round; the whole character and any slash must fit in the central 76% of each cell. Character standing height around 58% of cell height, consistent throughout, sole line around 87% of cell height. Absolutely no limbs, hair, blade tips or effects crossing any cell boundary.
