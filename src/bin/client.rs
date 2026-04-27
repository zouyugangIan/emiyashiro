use bevy::{asset::AssetPlugin, prelude::*};
use emiyashiro::plugins::EmiyaShiroClientPlugin;
use std::path::PathBuf;

fn resolve_asset_dir() -> PathBuf {
    // 1) 显式覆盖（CI/发布/容器最稳）
    if let Ok(v) = std::env::var("EMIYASHIRO_ASSET_DIR") {
        return PathBuf::from(v);
    }
    // 2) 运行时有 CARGO_MANIFEST_DIR（某些调试器会注入）
    if let Ok(v) = std::env::var("CARGO_MANIFEST_DIR") {
        return PathBuf::from(v).join("assets");
    }
    // 3) 编译时回退（cargo build/run 场景稳定）
    if let Some(v) = option_env!("CARGO_MANIFEST_DIR") {
        return PathBuf::from(v).join("assets");
    }
    // 4) 最后回退（从 target/debug 启动时尝试回到项目根）
    if let Ok(exe) = std::env::current_exe()
        && let Some(root) = exe
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
    {
        let candidate = root.join("assets");
        if candidate.exists() {
            return candidate;
        }
    }
    PathBuf::from("assets")
}

fn main() {
    let asset_dir = resolve_asset_dir()
        .canonicalize()
        .unwrap_or_else(|_| resolve_asset_dir());
    eprintln!("[bootstrap] asset_dir={}", asset_dir.display());

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_dir.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "EmiyaShiro(G-Engine)".into(),
                        resolution: (1024, 768).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(EmiyaShiroClientPlugin)
        .run();
}
