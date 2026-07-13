use std::{fs, path::Path};

use serde_json::Value;

use crate::asset_paths;

fn png_dimensions(path: &Path) -> (u32, u32) {
    let bytes = fs::read(path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    });
    assert!(
        bytes.len() >= 24,
        "{} is too small to be a PNG",
        path.display()
    );
    assert_eq!(
        &bytes[..8],
        b"\x89PNG\r\n\x1a\n",
        "{} must be a PNG",
        path.display()
    );

    let width = u32::from_be_bytes(bytes[16..20].try_into().expect("png width bytes"));
    let height = u32::from_be_bytes(bytes[20..24].try_into().expect("png height bytes"));
    (width, height)
}

fn expected_sheet(id: &str) -> (&'static str, u32, u32) {
    match id {
        "ground_light" => (
            asset_paths::IMAGE_HF_SHIROU_ATTACK_GROUND_LIGHT_REFERENCE,
            asset_paths::REFERENCE_BOARD_GROUND_LIGHT_COLS,
            asset_paths::REFERENCE_BOARD_GROUND_LIGHT_ROWS,
        ),
        "heavy" => (
            asset_paths::IMAGE_HF_SHIROU_ATTACK_HEAVY_REFERENCE,
            asset_paths::REFERENCE_BOARD_HEAVY_COLS,
            asset_paths::REFERENCE_BOARD_HEAVY_ROWS,
        ),
        "air_combo" => (
            asset_paths::IMAGE_HF_SHIROU_ATTACK_AIR_COMBO_REFERENCE,
            asset_paths::REFERENCE_BOARD_AIR_COMBO_COLS,
            asset_paths::REFERENCE_BOARD_AIR_COMBO_ROWS,
        ),
        "mobility" => (
            asset_paths::IMAGE_HF_SHIROU_ATTACK_MOBILITY_REFERENCE,
            asset_paths::REFERENCE_BOARD_MOBILITY_COLS,
            asset_paths::REFERENCE_BOARD_MOBILITY_ROWS,
        ),
        "ninjutsu_projectiles" => (
            asset_paths::IMAGE_HF_SHIROU_ATTACK_NINJUTSU_PROJECTILES_REFERENCE,
            asset_paths::REFERENCE_BOARD_NINJUTSU_COLS,
            asset_paths::REFERENCE_BOARD_NINJUTSU_ROWS,
        ),
        "ultimate" => (
            asset_paths::IMAGE_HF_SHIROU_ATTACK_ULTIMATE_REFERENCE,
            asset_paths::REFERENCE_BOARD_ULTIMATE_COLS,
            asset_paths::REFERENCE_BOARD_ULTIMATE_ROWS,
        ),
        "weapon_projection" => (
            asset_paths::IMAGE_HF_SHIROU_ATTACK_WEAPON_PROJECTION_REFERENCE,
            asset_paths::REFERENCE_BOARD_WEAPON_PROJ_COLS,
            asset_paths::REFERENCE_BOARD_WEAPON_PROJ_ROWS,
        ),
        other => panic!("unexpected attack sheet id: {other}"),
    }
}

fn expected_row_sheets(id: &str) -> &'static [&'static str] {
    match id {
        "ground_light" => &asset_paths::IMAGE_HF_SHIROU_ATTACK_GROUND_LIGHT_ROW_SHEETS,
        "heavy"
        | "air_combo"
        | "mobility"
        | "ninjutsu_projectiles"
        | "ultimate"
        | "weapon_projection" => &[],
        other => panic!("unexpected attack sheet id: {other}"),
    }
}

fn nonempty_field<'a>(row: &'a Value, name: &str) -> &'a str {
    row.get(name)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| panic!("row is missing non-empty {name}: {row:?}"))
}

fn expected_sakura_sheet(id: &str) -> (&'static str, &'static str, (u32, u32)) {
    match id {
        "ground_light" => (
            asset_paths::IMAGE_SAKURA_ATTACK_GROUND_LIGHT,
            asset_paths::SAKURA_ATTACK_GROUND_LIGHT_GROUP,
            asset_paths::SAKURA_ATTACK_GROUND_LIGHT_GRID,
        ),
        "heavy" => (
            asset_paths::IMAGE_SAKURA_ATTACK_HEAVY,
            asset_paths::SAKURA_ATTACK_HEAVY_GROUP,
            asset_paths::SAKURA_ATTACK_HEAVY_GRID,
        ),
        "air_combo" => (
            asset_paths::IMAGE_SAKURA_ATTACK_AIR_COMBO,
            asset_paths::SAKURA_ATTACK_AIR_COMBO_GROUP,
            asset_paths::SAKURA_ATTACK_AIR_COMBO_GRID,
        ),
        "mobility" => (
            asset_paths::IMAGE_SAKURA_ATTACK_MOBILITY,
            asset_paths::SAKURA_ATTACK_MOBILITY_GROUP,
            asset_paths::SAKURA_ATTACK_MOBILITY_GRID,
        ),
        "ninjutsu_projectiles" => (
            asset_paths::IMAGE_SAKURA_ATTACK_NINJUTSU,
            asset_paths::SAKURA_ATTACK_NINJUTSU_GROUP,
            asset_paths::SAKURA_ATTACK_NINJUTSU_GRID,
        ),
        "ultimate" => (
            asset_paths::IMAGE_SAKURA_ATTACK_ULTIMATE,
            asset_paths::SAKURA_ATTACK_ULTIMATE_GROUP,
            asset_paths::SAKURA_ATTACK_ULTIMATE_GRID,
        ),
        "weapon_projection" => (
            asset_paths::IMAGE_SAKURA_ATTACK_WEAPON_PROJECTION,
            asset_paths::SAKURA_ATTACK_WEAPON_PROJECTION_GROUP,
            asset_paths::SAKURA_ATTACK_WEAPON_PROJECTION_GRID,
        ),
        other => panic!("unexpected Sakura attack sheet id: {other}"),
    }
}

#[test]
fn hf_shirou_attack_row_plan_matches_runtime_atlases() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let plan_path = root.join("assets/images/characters/reference/hf_shirou_attack_rows.json");
    let plan = fs::read_to_string(&plan_path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", plan_path.display());
    });
    let plan: Value = serde_json::from_str(&plan).expect("valid attack row plan json");

    let cell_size = plan
        .get("cell_size")
        .and_then(Value::as_array)
        .expect("cell_size array");
    let cell_width = cell_size[0].as_u64().expect("cell width") as u32;
    let cell_height = cell_size[1].as_u64().expect("cell height") as u32;
    assert_eq!((cell_width, cell_height), (256, 256));

    let sheets = plan
        .get("sheets")
        .and_then(Value::as_array)
        .expect("sheets array");
    assert_eq!(
        sheets.len(),
        7,
        "every runtime reference atlas needs a row plan"
    );

    let mut planned_rows = 0usize;
    let mut planned_frames = 0usize;
    for sheet in sheets {
        let id = sheet.get("id").and_then(Value::as_str).expect("sheet id");
        let (expected_asset_path, expected_cols, expected_rows) = expected_sheet(id);
        let columns = sheet
            .get("columns")
            .and_then(Value::as_u64)
            .expect("columns") as u32;
        let asset_path = sheet
            .get("asset_path")
            .and_then(Value::as_str)
            .expect("asset_path");
        let source = sheet.get("source").and_then(Value::as_str).expect("source");
        let rows = sheet.get("rows").and_then(Value::as_array).expect("rows");
        let row_sheets = expected_row_sheets(id);

        assert_eq!(
            asset_path, expected_asset_path,
            "{id} uses the runtime asset path"
        );
        assert_eq!(
            columns, expected_cols,
            "{id} column count matches runtime constants"
        );
        assert_eq!(
            rows.len() as u32,
            expected_rows,
            "{id} row count matches runtime constants"
        );

        let source_path = root.join(source);
        let (width, height) = png_dimensions(&source_path);
        assert_eq!(width, columns * cell_width, "{id} width matches grid");
        assert_eq!(
            height,
            rows.len() as u32 * cell_height,
            "{id} height matches grid"
        );
        assert!(
            row_sheets.is_empty() || row_sheets.len() == rows.len(),
            "{id} row sheets must either be omitted or cover every planned row"
        );

        for (index, row) in rows.iter().enumerate() {
            assert_eq!(
                row.get("row").and_then(Value::as_u64),
                Some(index as u64 + 1),
                "{id} row numbers must stay sequential"
            );
            if let Some(row_sheet) = row_sheets.get(index) {
                let row_sheet_path = root.join("assets").join(row_sheet);
                assert_eq!(
                    png_dimensions(&row_sheet_path),
                    (columns * cell_width, cell_height),
                    "{id} row {} runtime PNG is a single-row atlas",
                    index + 1
                );
            }
            nonempty_field(row, "name");
            nonempty_field(row, "primary_input");
            nonempty_field(row, "runtime_style");
            nonempty_field(row, "combo_role");
            nonempty_field(row, "gameplay");
        }

        planned_rows += rows.len();
        planned_frames += rows.len() * columns as usize;
    }

    assert_eq!(planned_rows, 30);
    assert_eq!(planned_frames, 224);
}

#[test]
fn sakura_attack_plan_has_all_standalone_runtime_images() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let plan_path = root.join("assets/images/characters/sakura_attack_rows.json");
    let plan: Value = serde_json::from_str(
        &fs::read_to_string(&plan_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", plan_path.display())),
    )
    .expect("valid Sakura attack row plan json");
    let sheets = plan["sheets"].as_array().expect("Sakura sheets array");
    assert_eq!(sheets.len(), 7);

    let mut frame_total = 0;
    for sheet in sheets {
        let id = sheet["id"].as_str().expect("Sakura sheet id");
        let (source_asset, frame_group, grid) = expected_sakura_sheet(id);
        assert_eq!(sheet["asset_path"].as_str(), Some(source_asset));
        assert_eq!(sheet["columns"].as_u64(), Some(grid.0 as u64));
        let rows = sheet["rows"].as_array().expect("Sakura rows array");
        assert_eq!(rows.len(), grid.1 as usize);

        for row in 1..=grid.1 as u8 {
            for frame in 1..=grid.0 as usize {
                let asset_path = asset_paths::sakura_attack_frame_path(frame_group, row, frame);
                let file_path = root.join("assets").join(&asset_path);
                assert_eq!(
                    png_dimensions(&file_path),
                    asset_paths::SAKURA_ATTACK_CELL,
                    "standalone Sakura frame {asset_path} must keep its source cell size"
                );
                frame_total += 1;
            }
        }
    }

    assert_eq!(frame_total, 224);

    let manifest_path = root.join("assets/images/characters/sakura_attack/frames/manifest.json");
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(&manifest_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", manifest_path.display())),
    )
    .expect("valid Sakura standalone frame manifest");

    assert_eq!(manifest["runtime_facing"].as_str(), Some("right"));
    let base_frames = manifest["base_frames"]
        .as_array()
        .expect("dedicated Sakura base frames");
    assert_eq!(base_frames.len(), 8);
    for frame in base_frames {
        let asset_path = frame["file"].as_str().expect("base frame file");
        assert_eq!(
            png_dimensions(&root.join(asset_path)),
            asset_paths::SAKURA_ATTACK_CELL,
            "base frame {asset_path} must use the same square render cell"
        );
        assert_eq!(frame["border_alpha_pixels"].as_u64(), Some(0));
        assert_eq!(frame["chroma_green_pixels"].as_u64(), Some(0));
        let bbox = frame["bbox"].as_array().expect("non-empty base alpha bbox");
        let visible_height =
            bbox[3].as_u64().expect("bbox bottom") - bbox[1].as_u64().expect("bbox top");
        assert!(
            visible_height >= 100,
            "Sakura should stay large in every base frame: {frame:?}"
        );
        assert_eq!(
            bbox[3].as_u64(),
            Some(232),
            "base-state images must share a stable foot baseline: {frame:?}"
        );
    }

    let runtime_frames = manifest["sheets"]
        .as_array()
        .expect("runtime sheets")
        .iter()
        .flat_map(|sheet| sheet["frames"].as_array().expect("runtime frames"));
    let mut manifest_frame_total = 0;
    for frame in runtime_frames {
        assert_eq!(frame["border_alpha_pixels"].as_u64(), Some(0));
        assert_eq!(frame["chroma_green_pixels"].as_u64(), Some(0));
        let bbox = frame["bbox"].as_array().expect("non-empty alpha bbox");
        let visible_height =
            bbox[3].as_u64().expect("bbox bottom") - bbox[1].as_u64().expect("bbox top");
        assert!(
            visible_height >= 120,
            "Sakura should occupy a large portion of every runtime frame: {frame:?}"
        );
        manifest_frame_total += 1;
    }
    assert_eq!(manifest_frame_total, 224);
}
