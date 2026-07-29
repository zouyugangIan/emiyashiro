use bevy::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{
    resources::{CompleteGameState, SaveFileData, SaveFileManager, SaveFileMetadata},
    systems::{error_handling::SaveSystemError, text_input::InputValidator},
};

use super::codec::{atomic_write_file, compress_data, decode_file_payload, is_compressed};

pub(super) fn write_save_file(
    save_path: &Path,
    save_data: &SaveFileData,
) -> Result<(), SaveSystemError> {
    let json = serde_json::to_string_pretty(save_data)
        .map_err(|error| SaveSystemError::SerializationFailed(error.to_string()))?;
    atomic_write_file(save_path, json.as_bytes())
        .map_err(|error| SaveSystemError::FileWriteFailed(error.to_string()))
}

pub(super) fn read_save_file(save_path: &Path) -> Result<SaveFileData, SaveSystemError> {
    let file_data =
        fs::read(save_path).map_err(|error| SaveSystemError::FileNotFound(error.to_string()))?;
    let json = decode_file_payload(&file_data)
        .map_err(|error| SaveSystemError::DeserializationFailed(error.to_string()))?;
    let mut save_data = serde_json::from_str::<SaveFileData>(&json).map_err(|error| {
        SaveSystemError::DeserializationFailed(format!("Unsupported save file format: {error}"))
    })?;

    if !save_data.verify_serialized_checksum(&json) {
        return Err(SaveSystemError::ChecksumMismatch);
    }

    save_data.metadata.file_path = save_path.to_string_lossy().to_string();
    Ok(save_data)
}

pub(super) fn save_game_state(
    save_path: PathBuf,
    game_state: CompleteGameState,
    metadata: SaveFileMetadata,
) -> Result<(), SaveSystemError> {
    write_save_file(&save_path, &SaveFileData::new(metadata, game_state))
}

pub(super) fn load_game_state(
    save_path: PathBuf,
) -> Result<(CompleteGameState, SaveFileMetadata), SaveSystemError> {
    let save_data = read_save_file(&save_path)?;
    Ok((save_data.game_state, save_data.metadata))
}

/// Refreshes the list of valid v2 saves in the configured directory.
pub fn scan_save_files(mut manager: ResMut<SaveFileManager>) {
    let save_directory = PathBuf::from(&manager.save_directory);
    manager.save_files.clear();

    let entries = match fs::read_dir(&save_directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            crate::debug_log!(
                "Save directory does not exist: {}",
                save_directory.display()
            );
            return;
        }
        Err(error) => {
            crate::debug_log!(
                "Failed to scan save directory {}: {}",
                save_directory.display(),
                error
            );
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }

        match read_save_file(&path) {
            Ok(save_data) => manager.save_files.push(save_data.metadata),
            Err(error) => {
                crate::debug_log!("Ignoring invalid save {}: {}", path.display(), error);
            }
        }
    }

    manager
        .save_files
        .sort_by_key(|save| std::cmp::Reverse(save.save_timestamp));
}

pub fn delete_save_file(
    save_name: &str,
    manager: &mut SaveFileManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let index = manager
        .save_files
        .iter()
        .position(|save| save.name == save_name)
        .ok_or_else(|| format!("Save file '{save_name}' not found"))?;
    let file_path = PathBuf::from(&manager.save_files[index].file_path);

    fs::remove_file(&file_path)
        .map_err(|error| format!("Failed to delete '{}': {error}", file_path.display()))?;
    manager.save_files.remove(index);
    Ok(())
}

pub fn rename_save_file(
    old_name: &str,
    new_name: &str,
    manager: &mut SaveFileManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let validated_name = InputValidator::new().validate_save_name(new_name)?;
    if manager
        .save_files
        .iter()
        .any(|save| save.name == validated_name && save.name != old_name)
    {
        return Err("Save name already exists".into());
    }

    let index = manager
        .save_files
        .iter()
        .position(|save| save.name == old_name)
        .ok_or_else(|| format!("Save file '{old_name}' not found"))?;
    let old_path = PathBuf::from(&manager.save_files[index].file_path);
    let save_directory = old_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .ok_or("Save file path has no parent directory")?;
    let new_path = save_directory.join(format!("{validated_name}.json"));

    if new_path != old_path && new_path.exists() {
        return Err(format!("Save file already exists: {}", new_path.display()).into());
    }

    let old_file_data = fs::read(&old_path)?;
    let was_compressed = is_compressed(&old_file_data);
    let json = decode_file_payload(&old_file_data)?;
    let save_data: SaveFileData = serde_json::from_str(&json)?;
    if !save_data.verify_serialized_checksum(&json) {
        return Err("Cannot rename a save with an invalid checksum".into());
    }

    let mut metadata = save_data.metadata;
    metadata.name = validated_name;
    metadata.file_path = new_path.to_string_lossy().to_string();
    let updated_save = SaveFileData::new(metadata.clone(), save_data.game_state);
    let updated_json = serde_json::to_string_pretty(&updated_save)?;
    let updated_bytes = if was_compressed {
        compress_data(updated_json.as_bytes(), 3)?
    } else {
        updated_json.into_bytes()
    };

    atomic_write_file(&new_path, &updated_bytes)?;
    if new_path != old_path {
        fs::remove_file(&old_path)?;
    }
    manager.save_files[index] = metadata;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::states::{CharacterType, CharacterType::Shirou};

    fn metadata(name: &str, file_path: &Path) -> SaveFileMetadata {
        SaveFileMetadata {
            name: name.to_string(),
            score: 42,
            distance: 12.0,
            play_time: 3.0,
            save_timestamp: chrono::Utc::now(),
            file_path: file_path.to_string_lossy().to_string(),
            selected_character: CharacterType::Sakura,
        }
    }

    fn legacy_character_save_json() -> String {
        let save_data = SaveFileData::new(
            SaveFileMetadata {
                selected_character: Shirou,
                ..metadata("legacy-character-name", Path::new("legacy.json"))
            },
            CompleteGameState::default(),
        );
        let mut document = serde_json::to_value(save_data).expect("serialize save value");
        document["metadata"]["selected_character"] = serde_json::json!("Shirou1");
        document["game_state"]["selected_character"] = serde_json::json!("Shirou1");
        document["checksum"] = serde_json::json!("");
        let unsigned = serde_json::to_string_pretty(&document).expect("serialize unsigned save");
        document["checksum"] =
            serde_json::json!(super::super::codec::calculate_checksum(unsigned.as_bytes()));
        serde_json::to_string_pretty(&document).expect("serialize signed legacy save")
    }

    #[test]
    fn load_rejects_checksum_mismatch() {
        let temp_path =
            std::env::temp_dir().join(format!("emiyashiro-load-{}.json", uuid::Uuid::new_v4()));
        let mut save_data = SaveFileData::new(
            metadata("checksum-check", &temp_path),
            CompleteGameState::default(),
        );
        save_data.checksum = "broken-checksum".to_string();
        fs::write(
            &temp_path,
            serde_json::to_string_pretty(&save_data).expect("serialize test save"),
        )
        .expect("write test save");

        assert!(matches!(
            read_save_file(&temp_path),
            Err(SaveSystemError::ChecksumMismatch)
        ));
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn load_accepts_legacy_character_names_with_valid_checksum() {
        let temp_path =
            std::env::temp_dir().join(format!("emiyashiro-legacy-{}.json", uuid::Uuid::new_v4()));
        fs::write(&temp_path, legacy_character_save_json()).expect("write legacy save");

        let save_data = read_save_file(&temp_path).expect("legacy save should load");
        assert_eq!(save_data.game_state.selected_character, Shirou);
        assert_eq!(save_data.metadata.selected_character, Shirou);
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn rename_updates_metadata_and_checksum() {
        let temp_directory =
            std::env::temp_dir().join(format!("emiyashiro-rename-save-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_directory).expect("create save directory");
        let old_path = temp_directory.join("old-slot.json");
        let old_metadata = metadata("old-slot", &old_path);
        write_save_file(
            &old_path,
            &SaveFileData::new(old_metadata.clone(), CompleteGameState::default()),
        )
        .expect("write save");
        let mut manager = SaveFileManager {
            save_directory: temp_directory.to_string_lossy().to_string(),
            save_files: vec![old_metadata],
            current_save_name: None,
            selected_save_index: None,
        };

        rename_save_file("old-slot", "new-slot", &mut manager).expect("rename save");

        let new_path = temp_directory.join("new-slot.json");
        assert!(!old_path.exists());
        let renamed = read_save_file(&new_path).expect("read renamed save");
        assert_eq!(renamed.metadata.name, "new-slot");
        assert_eq!(
            renamed.metadata.file_path,
            new_path.to_string_lossy().as_ref()
        );

        let mut app = App::new();
        app.insert_resource(manager)
            .add_systems(Update, scan_save_files);
        app.update();
        let scanned = app.world().resource::<SaveFileManager>();
        assert_eq!(scanned.save_files.len(), 1);
        assert_eq!(scanned.save_files[0].name, "new-slot");

        let _ = fs::remove_dir_all(temp_directory);
    }
}
