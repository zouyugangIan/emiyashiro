use crate::{StartLoadGame, StartSaveGame};
use crate::{components::*, resources::*, states::*};
use bevy::prelude::*;

/// Gameplay HUD root marker.
#[derive(Component)]
pub struct GameHUD;

/// HUD score text marker.
#[derive(Component)]
pub struct ScoreDisplay;

/// HUD distance text marker.
#[derive(Component)]
pub struct DistanceDisplay;

/// HUD health text marker.
#[derive(Component)]
pub struct HealthDisplay;

// Enhanced Pause System UI Components
#[derive(Component)]
pub struct PauseMenuRoot;

#[derive(Component)]
pub struct SaveGameButton;

#[derive(Component)]
pub struct LoadGameButton;

#[derive(Component)]
pub struct ResumeButton;

#[derive(Component)]
pub struct MainMenuButton;

// Pause hotkey button markers.
#[derive(Component)]
pub struct EscKeyButton;

#[derive(Component)]
pub struct QKeyButton;

// Save dialog text input resource.
#[derive(Resource, Default)]
pub struct SaveNameInput {
    pub current_name: String,
    pub is_editing: bool,
}

// Loaded save handoff state.
#[derive(Resource, Default)]
pub struct LoadedGameState {
    pub state: Option<CompleteGameState>,
    pub should_restore: bool,
    pub previous_state: Option<GameState>,
}

/// Save/Load UI runtime state
#[derive(Resource, Default)]
pub struct SaveLoadUiState {
    pub is_busy: bool,
    pub status_message: String,
    pub error_message: String,
    pub pending_load_index: Option<usize>,
}

// Save Dialog Components
#[derive(Component)]
pub struct SaveDialog;

#[derive(Component)]
pub struct ConfirmSaveButton;

#[derive(Component)]
pub struct CancelSaveButton;

#[derive(Component)]
pub struct SaveNameInputBox;

#[derive(Component)]
pub struct TextCursor {
    pub blink_timer: f32,
    pub visible: bool,
}

#[derive(Component)]
pub struct SaveLoadStatusText;

// Load Table Components
#[derive(Component)]
pub struct LoadTableRoot;

#[derive(Component)]
pub struct SaveFileRow {
    pub save_index: usize,
}

#[derive(Component)]
pub struct LoadButton;

#[derive(Component)]
pub struct DeleteButton;

#[derive(Component)]
pub struct RenameButton;

#[derive(Component)]
pub struct RenameDialog;

#[derive(Component)]
pub struct RenameInputBox;

#[derive(Component)]
pub struct ConfirmRenameButton;

#[derive(Component)]
pub struct CancelRenameButton;

// 闁插秴鎳￠崥宥堢翻閸忋儴绁┃?
#[derive(Resource, Default)]
pub struct RenameInput {
    pub current_name: String,
    pub original_name: String,
    pub save_index: usize,
    pub is_editing: bool,
}

type SaveDialogInteractionQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Interaction,
        &'static mut BackgroundColor,
        Option<&'static ConfirmSaveButton>,
        Option<&'static CancelSaveButton>,
    ),
    (Changed<Interaction>, With<Button>),
>;

type LoadTableInteractionQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Interaction,
        &'static mut BackgroundColor,
        Option<&'static SaveFileRow>,
        Option<&'static CancelSaveButton>,
        Option<&'static LoadButton>,
        Option<&'static RenameButton>,
        Option<&'static DeleteButton>,
    ),
    (Changed<Interaction>, With<Button>),
>;

type RenameDialogInteractionQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Interaction,
        &'static mut BackgroundColor,
        Option<&'static ConfirmRenameButton>,
        Option<&'static CancelRenameButton>,
    ),
    (Changed<Interaction>, With<Button>),
>;

type PauseMenuInteractionQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Interaction,
        &'static mut BackgroundColor,
        Option<&'static ResumeButton>,
        Option<&'static SaveGameButton>,
        Option<&'static LoadGameButton>,
        Option<&'static MainMenuButton>,
        Option<&'static EscKeyButton>,
        Option<&'static QKeyButton>,
    ),
    (Changed<Interaction>, With<Button>),
>;

type ScoreTextQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut Text,
    (
        With<ScoreDisplay>,
        Without<DistanceDisplay>,
        Without<HealthDisplay>,
    ),
>;

type DistanceTextQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut Text,
    (
        With<DistanceDisplay>,
        Without<ScoreDisplay>,
        Without<HealthDisplay>,
    ),
>;

type HealthTextQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut Text,
    (
        With<HealthDisplay>,
        Without<ScoreDisplay>,
        Without<DistanceDisplay>,
    ),
>;

/// 鐠佸墽鐤嗗〒鍛婂灆锟?HUD
fn save_player_label(character: &CharacterType) -> &'static str {
    match character {
        CharacterType::Shirou1 => "1P",
        CharacterType::Shirou2 => "2P",
    }
}

pub fn setup_game_hud(
    mut commands: Commands,
    game_assets: Option<Res<GameAssets>>,
    existing_hud: Query<Entity, With<GameHUD>>,
) {
    if !existing_hud.is_empty() {
        return;
    }

    // 閸掓稑锟?HUD 閺嶇濡悙?
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
            GameHUD,
        ))
        .with_children(|parent| {
            // 閸掑棙鏆熼弰鍓с仛
            if let Some(assets) = &game_assets {
                parent.spawn((
                    Text::new(format!(
                        "{}0",
                        crate::systems::text_constants::GameHUDText::SCORE_LABEL
                    )),
                    TextFont {
                        font: assets.font.clone(),
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    ScoreDisplay,
                ));
            } else {
                parent.spawn((
                    Text::new(format!(
                        "{}0",
                        crate::systems::text_constants::GameHUDText::SCORE_LABEL
                    )),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    ScoreDisplay,
                ));
            }

            // 鐠烘繄顬囬弰鍓с仛
            parent.spawn((
                Text::new(format!(
                    "{}0{}",
                    crate::systems::text_constants::GameHUDText::DISTANCE_LABEL,
                    crate::systems::text_constants::GameHUDText::METERS_UNIT
                )),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                DistanceDisplay,
            ));

            // 閻㈢喎鎳￠崐鍏兼▔缁€?
            parent.spawn((
                Text::new("HP: 100/100"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.95, 0.85)),
                Node {
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                HealthDisplay,
            ));

            // 閹垮秳缍旈幓鎰仛
            parent.spawn((
                Text::new(crate::systems::text_constants::PauseMenuText::CONTROLS_HINT),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
                Node {
                    margin: UiRect::all(Val::Px(10.0)),
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(10.0),
                    left: Val::Px(10.0),
                    ..default()
                },
            ));
        });
}

/// 閺囧瓨鏌婂〒鍛婂灆 HUD
pub fn update_game_hud(
    mut score_text_query: ScoreTextQuery,
    mut distance_text_query: DistanceTextQuery,
    mut health_text_query: HealthTextQuery,
    player_health_query: Query<&Health, With<Player>>,
    game_stats: Res<GameStats>,
) {
    use crate::systems::text_constants::GameHUDText;

    // 閺囧瓨鏌婇崚鍡樻殶閺勫墽锟?
    if let Ok(mut score_text) = score_text_query.single_mut() {
        let score = (game_stats.distance_traveled * 10.0) as u32 + game_stats.jump_count * 50;
        **score_text = format!("{}{}", GameHUDText::SCORE_LABEL, score);
    }

    // 閺囧瓨鏌婄捄婵堫瀲閺勫墽锟?
    if let Ok(mut distance_text) = distance_text_query.single_mut() {
        **distance_text = format!(
            "{}{}{}",
            GameHUDText::DISTANCE_LABEL,
            game_stats.distance_traveled as u32,
            GameHUDText::METERS_UNIT
        );
    }

    // 閺囧瓨鏌婇悽鐔锋嚒閸婂吋妯夌粈?
    if let (Ok(mut health_text), Ok(player_health)) =
        (health_text_query.single_mut(), player_health_query.single())
    {
        **health_text = format!("HP: {:.0}/{:.0}", player_health.current, player_health.max);
    }
}

/// 濞撳懐鎮婂〒鍛婂灆 HUD
pub fn cleanup_game_hud(mut commands: Commands, hud_query: Query<Entity, With<GameHUD>>) {
    for entity in hud_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 閺嗗倸浠犻懣婊冨礋缂佸嫪锟?
#[derive(Component)]
pub struct PauseMenu;

/// 鐠佸墽鐤嗘晶鐐插繁閻ㄥ嫭娈忛崑婊嗗綅锟?
pub fn setup_pause_menu(mut commands: Commands, game_assets: Option<Res<GameAssets>>) {
    use crate::systems::text_constants::PauseMenuText;

    let font_handle = game_assets
        .as_ref()
        .map(|a| a.font.clone())
        .unwrap_or_default();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            PauseMenuRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(400.0),
                        height: Val::Px(350.0),
                        border: UiRect::all(Val::Px(2.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceEvenly,
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.95)),
                    BorderColor::all(Color::WHITE),
                ))
                .with_children(|parent| {
                    // 濞撳憡鍨欓弳鍌氫粻閺嶅洭锟?
                    parent.spawn((
                        Text::new(PauseMenuText::TITLE),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 36.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::bottom(Val::Px(20.0)),
                            ..default()
                        },
                    ));

                    // 缂佈呯敾濞撳憡鍨欓幐澶愭尦
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(200.0),
                                height: Val::Px(50.0),
                                border: UiRect::all(Val::Px(2.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.15, 0.3, 0.15, 0.8)),
                            BorderColor::all(Color::srgba(0.3, 0.6, 0.3, 1.0)),
                            ResumeButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new(PauseMenuText::RESUME_GAME),
                                TextFont {
                                    font: font_handle.clone(),
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    // 娣囨繂鐡ㄥ〒鍛婂灆閹稿锟?
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(200.0),
                                height: Val::Px(50.0),
                                border: UiRect::all(Val::Px(2.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.15, 0.15, 0.3, 0.8)),
                            BorderColor::all(Color::srgba(0.3, 0.3, 0.6, 1.0)),
                            SaveGameButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new(PauseMenuText::SAVE_GAME),
                                TextFont {
                                    font: font_handle.clone(),
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    // 閸旂姾娴囧〒鍛婂灆閹稿锟?
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(200.0),
                                height: Val::Px(50.0),
                                border: UiRect::all(Val::Px(2.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.15, 0.3, 0.15, 0.8)),
                            BorderColor::all(Color::srgba(0.3, 0.6, 0.3, 1.0)),
                            LoadGameButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new(PauseMenuText::LOAD_GAME),
                                TextFont {
                                    font: font_handle.clone(),
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    // 娑撴槒褰嶉崡鏇熷瘻锟?
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(200.0),
                                height: Val::Px(50.0),
                                border: UiRect::all(Val::Px(2.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.3, 0.15, 0.15, 0.8)),
                            BorderColor::all(Color::srgba(0.6, 0.3, 0.3, 1.0)),
                            MainMenuButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new(PauseMenuText::MAIN_MENU),
                                TextFont {
                                    font: font_handle.clone(),
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    // 闁款喚娲忚箛顐ｅ祹闁款喗瀵滈柦?
                    parent
                        .spawn((Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(60.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceEvenly,
                            align_items: AlignItems::Center,
                            margin: UiRect::top(Val::Px(20.0)),
                            ..default()
                        },))
                        .with_children(|parent| {
                            // ESC闁款喗瀵滈柦?
                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(120.0),
                                        height: Val::Px(35.0),
                                        border: UiRect::all(Val::Px(1.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        margin: UiRect::all(Val::Px(5.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.2, 0.4, 0.2, 0.8)),
                                    BorderColor::all(Color::srgba(0.4, 0.8, 0.4, 1.0)),
                                    EscKeyButton,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new(PauseMenuText::ESC_RESUME),
                                        TextFont {
                                            font: font_handle.clone(),
                                            font_size: 14.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });

                            // Q闁款喗瀵滈柦?
                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(120.0),
                                        height: Val::Px(35.0),
                                        border: UiRect::all(Val::Px(1.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        margin: UiRect::all(Val::Px(5.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.4, 0.2, 0.2, 0.8)),
                                    BorderColor::all(Color::srgba(0.8, 0.4, 0.4, 1.0)),
                                    QKeyButton,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new(PauseMenuText::Q_MAIN_MENU),
                                        TextFont {
                                            font: font_handle.clone(),
                                            font_size: 14.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                        });
                });
        });
}

/// 濞撳懐鎮婇弳鍌氫粻閼挎粌锟?
pub fn cleanup_pause_menu(mut commands: Commands, pause_query: Query<Entity, With<PauseMenuRoot>>) {
    for entity in pause_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 鐠佸墽鐤嗘穱婵嗙摠鐎电鐦藉?
pub fn setup_save_dialog(
    mut commands: Commands,
    game_assets: Option<Res<GameAssets>>,
    mut save_name_input: ResMut<SaveNameInput>,
    mut text_input_state: ResMut<crate::systems::text_input::TextInputState>,
    mut save_load_ui_state: ResMut<SaveLoadUiState>,
) {
    use crate::systems::text_constants::SaveLoadText;

    // 闁插秶鐤嗘潏鎾冲弳閻樿埖锟?- 濞撳懐鈹栨潏鎾冲弳锟?
    save_name_input.current_name.clear();
    save_name_input.is_editing = true;

    // 濠碘偓濞茬粯鏌婇惃鍕瀮閺堫剝绶崗銉ч兇锟?
    text_input_state.activate();

    save_load_ui_state.pending_load_index = None;
    if !save_load_ui_state.is_busy {
        save_load_ui_state.status_message.clear();
    }
    save_load_ui_state.error_message.clear();

    let (initial_status_text, initial_status_color) =
        if !save_load_ui_state.status_message.is_empty() {
            (
                save_load_ui_state.status_message.clone(),
                Color::srgba(0.65, 0.85, 1.0, 0.95),
            )
        } else {
            (
                SaveLoadText::INPUT_HINT.to_string(),
                Color::srgba(0.8, 0.8, 0.8, 0.6),
            )
        };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            SaveDialog,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(450.0),
                        height: Val::Px(280.0),
                        border: UiRect::all(Val::Px(2.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceEvenly,
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.95)),
                    BorderColor::all(Color::WHITE),
                ))
                .with_children(|parent| {
                    let font_handle = game_assets
                        .as_ref()
                        .map(|a| a.font.clone())
                        .unwrap_or_default();

                    // 閺嶅洭锟?
                    parent.spawn((
                        Text::new(SaveLoadText::SAVE_DIALOG_TITLE),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // 鏉堟挸鍙嗛幓鎰仛
                    parent.spawn((
                        Text::new(SaveLoadText::ENTER_SAVE_NAME),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
                    ));

                    // 鏉堟挸鍙嗗?(閺勫墽銇氳ぐ鎾冲鏉堟挸鍙嗛惃鍕倳锟?
                    parent
                        .spawn((
                            Node {
                                width: Val::Px(350.0),
                                height: Val::Px(45.0),
                                border: UiRect::all(Val::Px(2.0)),
                                justify_content: JustifyContent::FlexStart,
                                align_items: AlignItems::Center,
                                padding: UiRect::all(Val::Px(10.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
                            BorderColor::all(Color::srgba(0.5, 0.8, 1.0, 1.0)), // Blue border to indicate active input
                        ))
                        .with_children(|parent| {
                            // 鏉堟挸鍙嗛弬鍥ㄦ拱
                            parent.spawn((
                                Text::new(SaveLoadText::NAME_PLACEHOLDER),
                                TextFont {
                                    font: font_handle.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)), // Placeholder color
                                SaveNameInputBox, // 鐏忓棙鐖ｇ拋鐗堝潑閸旂姴鍩岄弬鍥ㄦ拱缂佸嫪娆㈡稉?
                            ));

                            // 閸忓锟?
                            parent.spawn((
                                Text::new("|"),
                                TextFont {
                                    font: font_handle.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
                                TextCursor {
                                    blink_timer: 0.0,
                                    visible: true,
                                },
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(15.0), // 閸掓繂顫愭担宥囩枂閿涘奔绱伴弽瑙勫祦閺傚洦婀伴梹鍨閸斻劍鈧浇鐨熼弫?
                                    ..default()
                                },
                            ));
                        });

                    // 鏉堟挸鍙嗛幓鎰仛娣団剝锟?
                    parent.spawn((
                        Text::new(initial_status_text.clone()),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(initial_status_color),
                        SaveLoadStatusText,
                    ));

                    // 閹稿鎸崇€圭懓锟?
                    parent
                        .spawn((Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(60.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceEvenly,
                            align_items: AlignItems::Center,
                            ..default()
                        },))
                        .with_children(|parent| {
                            // 绾喛顓婚幐澶愭尦
                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(120.0),
                                        height: Val::Px(40.0),
                                        border: UiRect::all(Val::Px(2.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.15, 0.3, 0.15, 0.8)),
                                    BorderColor::all(Color::srgba(0.3, 0.6, 0.3, 1.0)),
                                    ConfirmSaveButton,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new(SaveLoadText::SAVE_BUTTON),
                                        TextFont {
                                            font: font_handle.clone(),
                                            font_size: 18.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });

                            // 閸欐牗绉烽幐澶愭尦
                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(120.0),
                                        height: Val::Px(40.0),
                                        border: UiRect::all(Val::Px(2.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.3, 0.15, 0.15, 0.8)),
                                    BorderColor::all(Color::srgba(0.6, 0.3, 0.3, 1.0)),
                                    CancelSaveButton,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new(SaveLoadText::CANCEL_BUTTON),
                                        TextFont {
                                            font: font_handle.clone(),
                                            font_size: 18.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                        });
                });
        });
}

/// 濞撳懐鎮婃穱婵嗙摠鐎电鐦藉?
pub fn cleanup_save_dialog(
    mut commands: Commands,
    dialog_query: Query<Entity, With<SaveDialog>>,
    mut text_input_state: ResMut<crate::systems::text_input::TextInputState>,
) {
    for entity in dialog_query.iter() {
        commands.entity(entity).despawn();
    }

    // 閸嬫粎鏁ら弬鍥ㄦ拱鏉堟挸鍙嗙化鑽ょ埠
    text_input_state.deactivate();
}

pub fn update_save_load_status_text(
    current_state: Res<State<GameState>>,
    save_load_ui_state: Res<SaveLoadUiState>,
    mut status_query: Query<(&mut Text, &mut TextColor), With<SaveLoadStatusText>>,
) {
    use crate::systems::text_constants::SaveLoadText;

    if status_query.is_empty() {
        return;
    }

    let default_message = match current_state.get() {
        GameState::SaveDialog => SaveLoadText::INPUT_HINT,
        GameState::LoadTable => SaveLoadText::CLICK_TO_LOAD,
        _ => "",
    };

    let (message, color) = if !save_load_ui_state.error_message.is_empty() {
        (
            format!("Error: {}", save_load_ui_state.error_message),
            Color::srgba(1.0, 0.45, 0.45, 1.0),
        )
    } else if !save_load_ui_state.status_message.is_empty() {
        (
            save_load_ui_state.status_message.clone(),
            Color::srgba(0.65, 0.85, 1.0, 0.95),
        )
    } else {
        (
            default_message.to_string(),
            Color::srgba(0.8, 0.8, 0.8, 0.6),
        )
    };

    for (mut text, mut text_color) in &mut status_query {
        text.0 = message.clone();
        text_color.0 = color;
    }
}

/// Text cursor blinking system
pub fn update_text_cursor(
    time: Res<Time>,
    mut cursor_query: Query<(&mut TextCursor, &mut Visibility, &mut Node)>,
    save_name_input: Res<SaveNameInput>,
) {
    for (mut cursor, mut visibility, mut node) in cursor_query.iter_mut() {
        cursor.blink_timer += time.delta_secs();

        if cursor.blink_timer >= 0.5 {
            cursor.blink_timer = 0.0;
            cursor.visible = !cursor.visible;
            *visibility = if cursor.visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }

        // 閺囧瓨鏌婇崗澶嬬垼娴ｅ秶鐤嗛崺杞扮艾瑜版挸澧犻弬鍥ㄦ拱闂€鍨
        // 濮ｅ繋閲滅€涙顑佹径褏锟?閸嶅繒绀岀€硅棄瀹抽敍?6閸欏嘲鐡ф担鎾舵畱娴兼壆鐣婚敍?
        let text_width = save_name_input.current_name.len() as f32 * 8.0;
        node.left = Val::Px(15.0 + text_width);
    }
}

/// Enhanced text input handler for save names
pub fn handle_save_name_input(
    text_input_state: Res<crate::systems::text_input::TextInputState>,
    mut save_name_input: ResMut<SaveNameInput>,
    mut text_query: Query<&mut Text, With<SaveNameInputBox>>,
) {
    // 閸氬本顒為弬鎵畱閺傚洦婀版潏鎾冲弳缁崵绮洪悩鑸碘偓浣稿煂閺冄呮畱娣囨繂鐡ㄩ崥宥囆炴潏鎾冲弳
    if text_input_state.is_active {
        save_name_input.current_name = text_input_state.current_text.clone();
        save_name_input.is_editing = true;

        // 閺囧瓨鏌婇弰鍓с仛閺傚洦锟?
        use crate::systems::text_constants::SaveLoadText;
        for mut text in text_query.iter_mut() {
            text.0 = if text_input_state.current_text.is_empty() {
                SaveLoadText::NAME_PLACEHOLDER.to_string()
            } else {
                format!("{}|", text_input_state.current_text)
            };
        }
    } else {
        save_name_input.is_editing = false;
    }
}

/// 婢跺嫮鎮婃穱婵嗙摠鐎电鐦藉鍡曟唉锟?
pub fn handle_save_dialog_interactions(
    mut interaction_query: SaveDialogInteractionQuery,
    mut next_state: ResMut<NextState<GameState>>,
    pause_manager: Res<PauseManager>,
    mut ev_save: MessageWriter<StartSaveGame>,
    text_input_state: Res<crate::systems::text_input::TextInputState>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut save_load_ui_state: ResMut<SaveLoadUiState>,
) {
    use crate::systems::text_constants::SaveLoadText;

    let mut should_save = false;
    let mut should_cancel = false;

    // 婢跺嫮鎮婇柨顔炬磸韫囶偅宓庨柨?
    if keyboard_input.just_pressed(KeyCode::Enter) && text_input_state.is_active {
        should_save = true;
        crate::debug_log!("Enter key pressed - saving game");
    } else if keyboard_input.just_pressed(KeyCode::Escape) && text_input_state.is_active {
        should_cancel = true;
        crate::debug_log!("Escape key pressed - canceling save");
    }

    // 婢跺嫮鎮婇幐澶愭尦娴溿倓锟?
    for (interaction, mut color, confirm_btn, cancel_btn) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if confirm_btn.is_some() {
                    crate::debug_log!("Save button pressed");
                    should_save = true;
                } else if cancel_btn.is_some() {
                    crate::debug_log!("Cancel button pressed");
                    should_cancel = true;
                }
                *color = BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.9));
            }
            Interaction::Hovered => {
                if confirm_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.25, 0.5, 0.25, 0.9));
                } else if cancel_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.5, 0.25, 0.25, 0.9));
                }
            }
            Interaction::None => {
                if confirm_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.15, 0.3, 0.15, 0.8));
                } else if cancel_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.3, 0.15, 0.15, 0.8));
                }
            }
        }
    }

    if should_save {
        if save_load_ui_state.is_busy {
            save_load_ui_state.status_message = "Save operation is already running...".to_string();
            return;
        }

        if let Some(state) = &pause_manager.preserved_state {
            let raw_save_name = if text_input_state.current_text.is_empty() {
                SaveLoadText::DEFAULT_SAVE_NAME.to_string()
            } else {
                text_input_state.current_text.clone()
            };

            let validator = crate::systems::text_input::InputValidator::new();
            match validator.validate_save_name(&raw_save_name) {
                Ok(save_name) => {
                    save_load_ui_state.is_busy = true;
                    save_load_ui_state.pending_load_index = None;
                    save_load_ui_state.error_message.clear();
                    save_load_ui_state.status_message = format!("Saving '{}'...", save_name);

                    crate::debug_log!("Firing StartSaveGame event with name: '{}'", save_name);
                    ev_save.write(StartSaveGame {
                        save_name,
                        state: state.clone(),
                    });
                    next_state.set(GameState::Paused);
                }
                Err(error) => {
                    save_load_ui_state.error_message =
                        format!("{}: {}", SaveLoadText::INVALID_NAME_ERROR, error);
                    save_load_ui_state.status_message.clear();
                    crate::debug_log!("Invalid save name: {}", error);
                }
            }
        } else {
            save_load_ui_state.error_message = "No paused game snapshot to save".to_string();
            save_load_ui_state.status_message.clear();
            crate::debug_log!("No game state to save: PauseManager snapshot is empty");
            next_state.set(GameState::Paused);
        }
    } else if should_cancel {
        if save_load_ui_state.is_busy {
            save_load_ui_state.status_message = "Save is in progress, please wait...".to_string();
            return;
        }

        save_load_ui_state.pending_load_index = None;
        // 閸欐牗绉锋穱婵嗙摠閿涘矁绻戦崶鐐存畯閸嬫粏褰嶉崡?
        next_state.set(GameState::Paused);
        crate::debug_log!("Save canceled");
    }
}

/// 鐠佸墽鐤嗘晶鐐插繁閻ㄥ嫬濮炴潪鍊熴€冮弽鑲╂櫕锟?
pub fn setup_load_table(
    mut commands: Commands,
    game_assets: Option<Res<GameAssets>>,
    save_file_manager: Res<SaveFileManager>,
    mut save_load_ui_state: ResMut<SaveLoadUiState>,
) {
    use crate::systems::text_constants::SaveLoadText;

    save_load_ui_state.pending_load_index = None;
    if !save_load_ui_state.is_busy {
        save_load_ui_state.status_message.clear();
        save_load_ui_state.error_message.clear();
    }

    let (initial_status_text, initial_status_color) =
        if !save_load_ui_state.error_message.is_empty() {
            (
                format!("Error: {}", save_load_ui_state.error_message),
                Color::srgba(1.0, 0.45, 0.45, 1.0),
            )
        } else if !save_load_ui_state.status_message.is_empty() {
            (
                save_load_ui_state.status_message.clone(),
                Color::srgba(0.65, 0.85, 1.0, 0.95),
            )
        } else {
            (
                SaveLoadText::CLICK_TO_LOAD.to_string(),
                Color::srgba(0.8, 0.8, 0.8, 0.7),
            )
        };

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        LoadTableRoot,
    )).with_children(|parent| {
        parent.spawn((
            Node {
                width: Val::Px(700.0),
                height: Val::Px(550.0),
                border: UiRect::all(Val::Px(2.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.95)),
            BorderColor::all(Color::WHITE),
        )).with_children(|parent| {
            let font_handle = game_assets.as_ref().map(|a| a.font.clone()).unwrap_or_default();

            // 閺嶅洭锟?
            parent.spawn((
                Text::new(crate::systems::text_constants::SaveLoadText::LOAD_DIALOG_TITLE),
                TextFont {
                    font: font_handle.clone(),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // 閹垮秳缍旈幓鎰仛
            parent.spawn((
                Text::new(initial_status_text.clone()),
                TextFont {
                    font: font_handle.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(initial_status_color),
                SaveLoadStatusText,
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            // 鐞涖劍鐗告径鎾劥
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(40.0),
                    flex_direction: FlexDirection::Row,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.8)),
                BorderColor::all(Color::srgba(0.5, 0.5, 0.5, 1.0)),
            )).with_children(|parent| {
                use crate::systems::text_constants::SaveLoadText;
                let headers = [
                    SaveLoadText::COL_NAME,
                    SaveLoadText::COL_PLAYERS,
                    SaveLoadText::COL_SCORE,
                    SaveLoadText::COL_DISTANCE,
                    SaveLoadText::COL_TIME,
                    SaveLoadText::COL_DATE,
                    SaveLoadText::COL_ACTIONS
                ];
                let widths = [18.0, 8.0, 12.0, 12.0, 12.0, 18.0, 20.0];

                for (header, width) in headers.iter().zip(widths.iter()) {
                    parent.spawn((
                        Node {
                            width: Val::Percent(*width),
                            height: Val::Percent(100.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::right(Val::Px(1.0)),
                            ..default()
                        },
                        BorderColor::all(Color::srgba(0.5, 0.5, 0.5, 1.0)),
                    )).with_children(|parent| {
                        parent.spawn((
                            Text::new(*header),
                            TextFont {
                                font: font_handle.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
                }
            });

            // 濠婃艾濮╅崠鍝勭厵
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(320.0),
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::clip_y(),
                    ..default()
                },
            )).with_children(|parent| {
                // 閺勫墽銇氱€涙ɑ銆傞弬鍥︽
                if save_file_manager.save_files.is_empty() {
                    parent.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(60.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                    )).with_children(|parent| {
                        parent.spawn((
                            Text::new(crate::systems::text_constants::SaveLoadText::NO_SAVES_FOUND),
                            TextFont {
                                font: font_handle.clone(),
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.6)),
                        ));
                    });
                } else {
                    for (index, save_file) in save_file_manager.save_files.iter().enumerate() {
                        parent.spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(45.0),
                                flex_direction: FlexDirection::Row,
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(if index % 2 == 0 {
                                Color::srgba(0.25, 0.25, 0.25, 0.8)
                            } else {
                                Color::srgba(0.2, 0.2, 0.2, 0.8)
                            }),
                            BorderColor::all(Color::srgba(0.4, 0.4, 0.4, 1.0)),
                        )).with_children(|parent| {
                            let widths = [18.0, 8.0, 12.0, 12.0, 12.0, 18.0, 20.0];
                            let values = [
                                save_file.name.clone(),
                                save_player_label(&save_file.selected_character).to_string(), // 姒涙顓婚崡鏇氭眽濞撳憡鍨欓敍灞炬弓閺夈儱褰叉禒搴＄摠濡楋絾鏆熼幑顔款嚢锟?
                                save_file.score.to_string(),
                                format!("{:.1}m", save_file.distance),
                                format!("{:.1}s", save_file.play_time),
                                save_file.save_timestamp.format("%m/%d %H:%M").to_string(),
                            ];

                            // 閺勫墽銇氱€涙ɑ銆傛穱鈩冧紖
                            for (i, (value, width)) in values.iter().zip(widths.iter()).enumerate() {
                                if i < 6 {
                                    // 锟?閸掓妯夌粈鐑樻殶锟?
                                    parent.spawn((
                                        Button,
                                        Node {
                                            width: Val::Percent(*width),
                                            height: Val::Percent(100.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::right(Val::Px(1.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::NONE),
                                        BorderColor::all(Color::srgba(0.4, 0.4, 0.4, 1.0)),
                                        SaveFileRow { save_index: index },
                                    )).with_children(|parent| {
                                        parent.spawn((
                                            Text::new(value.clone()),
                                            TextFont {
                                                font: font_handle.clone(),
                                                font_size: 13.0,
                                                ..default()
                                            },
                                            TextColor(if i == 1 {
                                                // 閻溾晛顔嶉弫浼村櫤閸掓ぞ濞囬悽銊ょ瑝閸氬矂顤侀懝?
                                                Color::srgba(0.7, 0.9, 1.0, 1.0)
                                            } else {
                                                Color::WHITE
                                            }),
                                        ));
                                    });
                                }
                            }

                            // 閹垮秳缍旈幐澶愭尦锟?
                            parent.spawn((
                                Node {
                                    width: Val::Percent(20.0),
                                    height: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::SpaceEvenly,
                                    align_items: AlignItems::Center,
                                    padding: UiRect::all(Val::Px(2.0)),
                                    ..default()
                                },
                            )).with_children(|parent| {
                                // 闁插秴鎳￠崥宥嗗瘻锟?
                                parent.spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(45.0),
                                        height: Val::Px(25.0),
                                        border: UiRect::all(Val::Px(1.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.2, 0.3, 0.4, 0.8)),
                                    BorderColor::all(Color::srgba(0.4, 0.6, 0.8, 1.0)),
                                    RenameButton,
                                    SaveFileRow { save_index: index },
                                )).with_children(|parent| {
                                    parent.spawn((
                                        Text::new(crate::systems::text_constants::SaveLoadText::RENAME_BUTTON),
                                        TextFont {
                                            font: font_handle.clone(),
                                            font_size: 10.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });

                                // 閸掔娀娅庨幐澶愭尦
                                parent.spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(45.0),
                                        height: Val::Px(25.0),
                                        border: UiRect::all(Val::Px(1.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.4, 0.2, 0.2, 0.8)),
                                    BorderColor::all(Color::srgba(0.8, 0.4, 0.4, 1.0)),
                                    DeleteButton,
                                    SaveFileRow { save_index: index },
                                )).with_children(|parent| {
                                    parent.spawn((
                                        Text::new(crate::systems::text_constants::SaveLoadText::DELETE_BUTTON),
                                        TextFont {
                                            font: font_handle.clone(),
                                            font_size: 10.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                            });
                        });
                    }
                }
            });

            // 鎼存洟鍎撮幐澶愭尦
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(60.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceEvenly,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
            )).with_children(|parent| {
                // 閸掗攱鏌婇幐澶愭尦
                parent.spawn((
                    Button,
                    Node {
                        width: Val::Px(100.0),
                        height: Val::Px(40.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.15, 0.25, 0.35, 0.8)),
                    BorderColor::all(Color::srgba(0.3, 0.5, 0.7, 1.0)),
                    LoadButton, // 闁插秶鏁ら崝鐘烘祰閹稿鎸崇紒鍕娴ｆ粈璐熼崚閿嬫煀
                )).with_children(|parent| {
                    parent.spawn((
                        Text::new(crate::systems::text_constants::SaveLoadText::REFRESH_BUTTON),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

                // 鏉╂柨娲栭幐澶愭尦
                parent.spawn((
                    Button,
                    Node {
                        width: Val::Px(100.0),
                        height: Val::Px(40.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.3, 0.15, 0.15, 0.8)),
                    BorderColor::all(Color::srgba(0.6, 0.3, 0.3, 1.0)),
                    CancelSaveButton, // 闁插秶鏁ら崣鏍ㄧХ閹稿鎸崇紒鍕
                )).with_children(|parent| {
                    parent.spawn((
                        Text::new(crate::systems::text_constants::SaveLoadText::BACK_BUTTON),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
            });
        });
    });
}

/// 濞撳懐鎮婇崝鐘烘祰鐞涖劍锟?
pub fn cleanup_load_table(mut commands: Commands, table_query: Query<Entity, With<LoadTableRoot>>) {
    for entity in table_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 鐠佸墽鐤嗛柌宥呮嚒閸氬秴顕拠婵囶攱
pub fn setup_rename_dialog(
    mut commands: Commands,
    game_assets: Option<Res<GameAssets>>,
    mut rename_input: ResMut<RenameInput>,
    mut text_input_state: ResMut<crate::systems::text_input::TextInputState>,
) {
    // 闁插秶鐤嗘潏鎾冲弳閻樿埖鈧緤绱濇担璺ㄦ暏閸樼喎顫愰崥宥囆炴担婊€璐熸妯款吇锟?
    rename_input.current_name = rename_input.original_name.clone();
    rename_input.is_editing = true;

    // 濠碘偓濞茬粯鏋冮張顒冪翻閸忋儳閮寸紒鐔疯嫙鐠佸墽鐤嗛崚婵嗩潗锟?
    text_input_state.activate();
    text_input_state.current_text = rename_input.original_name.clone();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            RenameDialog,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(450.0),
                        height: Val::Px(280.0),
                        border: UiRect::all(Val::Px(2.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceEvenly,
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.95)),
                    BorderColor::all(Color::WHITE),
                ))
                .with_children(|parent| {
                    let font_handle = game_assets
                        .as_ref()
                        .map(|a| a.font.clone())
                        .unwrap_or_default();

                    // 閺嶅洭锟?
                    parent.spawn((
                        Text::new(
                            crate::systems::text_constants::SaveLoadText::RENAME_DIALOG_TITLE,
                        ),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // 閸樼喎顫愰崥宥囆為弰鍓с仛
                    parent.spawn((
                        Text::new(format!("Current name: {}", rename_input.original_name)),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
                    ));

                    // 鏉堟挸鍙嗛幓鎰仛
                    parent.spawn((
                        Text::new(crate::systems::text_constants::SaveLoadText::ENTER_NEW_NAME),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
                    ));

                    // 鏉堟挸鍙嗗?
                    parent
                        .spawn((
                            Node {
                                width: Val::Px(350.0),
                                height: Val::Px(40.0),
                                border: UiRect::all(Val::Px(2.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                padding: UiRect::all(Val::Px(10.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
                            BorderColor::all(Color::srgba(0.5, 0.5, 0.5, 1.0)),
                            RenameInputBox,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new(rename_input.current_name.clone()),
                                TextFont {
                                    font: font_handle.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    // 閹稿鎸崇€圭懓锟?
                    parent
                        .spawn((Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(60.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceEvenly,
                            align_items: AlignItems::Center,
                            ..default()
                        },))
                        .with_children(|parent| {
                            // 绾喛顓婚幐澶愭尦
                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(120.0),
                                        height: Val::Px(40.0),
                                        border: UiRect::all(Val::Px(2.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.15, 0.3, 0.15, 0.8)),
                                    BorderColor::all(Color::srgba(0.3, 0.6, 0.3, 1.0)),
                                    ConfirmRenameButton,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                        Text::new(crate::systems::text_constants::SaveLoadText::CONFIRM_BUTTON),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                                });

                            // 閸欐牗绉烽幐澶愭尦
                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(120.0),
                                        height: Val::Px(40.0),
                                        border: UiRect::all(Val::Px(2.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.3, 0.15, 0.15, 0.8)),
                                    BorderColor::all(Color::srgba(0.6, 0.3, 0.3, 1.0)),
                                    CancelRenameButton,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                        Text::new(crate::systems::text_constants::SaveLoadText::CANCEL_BUTTON),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                                });
                        });
                });
        });
}

/// 濞撳懐鎮婇柌宥呮嚒閸氬秴顕拠婵囶攱
pub fn cleanup_rename_dialog(
    mut commands: Commands,
    dialog_query: Query<Entity, With<RenameDialog>>,
    mut text_input_state: ResMut<crate::systems::text_input::TextInputState>,
) {
    for entity in dialog_query.iter() {
        commands.entity(entity).despawn();
    }

    // 閸嬫粎鏁ら弬鍥ㄦ拱鏉堟挸鍙嗙化鑽ょ埠
    text_input_state.deactivate();
}

/// 婢跺嫮鎮婃晶鐐插繁閻ㄥ嫬濮炴潪鍊熴€冮弽闂存唉锟?
pub fn handle_load_table_interactions(
    mut interaction_query: LoadTableInteractionQuery,
    mut next_state: ResMut<NextState<GameState>>,
    mut save_file_manager: ResMut<SaveFileManager>,
    mut ev_load: MessageWriter<StartLoadGame>,
    mut loaded_game_state: ResMut<LoadedGameState>,
    mut rename_input: ResMut<RenameInput>,
    mut save_load_ui_state: ResMut<SaveLoadUiState>,
) {
    let mut selected_save_index: Option<usize> = None;
    let mut should_cancel = false;
    let mut should_refresh = false;
    let mut rename_index: Option<usize> = None;
    let mut delete_index: Option<usize> = None;

    for (interaction, mut color, save_row, cancel_btn, load_btn, rename_btn, delete_btn) in
        &mut interaction_query
    {
        match *interaction {
            Interaction::Pressed => {
                if let Some(&SaveFileRow { save_index }) = save_row {
                    if rename_btn.is_some() {
                        rename_index = Some(save_index);
                    } else if delete_btn.is_some() {
                        delete_index = Some(save_index);
                    } else {
                        selected_save_index = Some(save_index);
                    }
                } else if cancel_btn.is_some() {
                    should_cancel = true;
                } else if load_btn.is_some() {
                    should_refresh = true;
                }
                *color = BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.9));
            }
            Interaction::Hovered => {
                if save_row.is_some() {
                    if rename_btn.is_some() {
                        *color = BackgroundColor(Color::srgba(0.3, 0.45, 0.6, 0.9));
                    } else if delete_btn.is_some() {
                        *color = BackgroundColor(Color::srgba(0.6, 0.3, 0.3, 0.9));
                    } else {
                        *color = BackgroundColor(Color::srgba(0.4, 0.4, 0.4, 0.9));
                    }
                } else if cancel_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.5, 0.25, 0.25, 0.9));
                } else if load_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.25, 0.4, 0.55, 0.9));
                }
            }
            Interaction::None => {
                if let Some(_row) = save_row {
                    if rename_btn.is_some() {
                        *color = BackgroundColor(Color::srgba(0.2, 0.3, 0.4, 0.8));
                    } else if delete_btn.is_some() {
                        *color = BackgroundColor(Color::srgba(0.4, 0.2, 0.2, 0.8));
                    } else {
                        *color = BackgroundColor(Color::NONE);
                    }
                } else if cancel_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.3, 0.15, 0.15, 0.8));
                } else if load_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.15, 0.25, 0.35, 0.8));
                }
            }
        }
    }

    // 婢跺嫮鎮婇崝鐘烘祰鐎涙ɑ锟?
    if let Some(index) = selected_save_index {
        if save_load_ui_state.is_busy {
            save_load_ui_state.status_message = "Load operation is already running...".to_string();
            return;
        }

        if index < save_file_manager.save_files.len() {
            let save_file = &save_file_manager.save_files[index];

            if save_load_ui_state.pending_load_index != Some(index) {
                save_load_ui_state.pending_load_index = Some(index);
                save_load_ui_state.error_message.clear();
                save_load_ui_state.status_message =
                    format!("Click '{}' again to confirm load", save_file.name);
                return;
            }

            save_load_ui_state.pending_load_index = None;
            save_load_ui_state.is_busy = true;
            save_load_ui_state.error_message.clear();
            save_load_ui_state.status_message = format!("Loading save '{}'...", save_file.name);

            crate::debug_log!(
                "Firing StartLoadGame event for path: '{}'",
                &save_file.file_path
            );
            ev_load.write(StartLoadGame {
                file_path: save_file.file_path.clone(),
            });
            // The UI will now wait for the async task to finish.
            // We could transition to a "Loading" state here, but for now,
            // we'll just stay on the load screen. The progress bar from
            // async_file_ops should appear.
        }
    }
    // 婢跺嫮鎮婇柌宥呮嚒锟?
    else if let Some(index) = rename_index {
        if save_load_ui_state.is_busy {
            save_load_ui_state.status_message =
                "Operation in progress, rename is temporarily disabled".to_string();
            return;
        }

        if index < save_file_manager.save_files.len() {
            let save_file = &save_file_manager.save_files[index];
            save_load_ui_state.pending_load_index = None;
            rename_input.original_name = save_file.name.clone();
            rename_input.save_index = index;
            next_state.set(GameState::RenameDialog);
            crate::debug_log!("Renaming save: {}", save_file.name);
        }
    }
    // 婢跺嫮鎮婇崚鐘绘珟
    else if let Some(index) = delete_index {
        if save_load_ui_state.is_busy {
            save_load_ui_state.status_message =
                "Operation in progress, delete is temporarily disabled".to_string();
            return;
        }

        if index < save_file_manager.save_files.len() {
            let save_name = save_file_manager.save_files[index].name.clone();
            match crate::systems::pause_save::delete_save_file(&save_name, &mut save_file_manager) {
                Ok(_) => {
                    save_load_ui_state.pending_load_index = None;
                    save_load_ui_state.error_message.clear();
                    save_load_ui_state.status_message = format!("Deleted save '{}'", save_name);
                    crate::debug_log!(
                        "{}: {}",
                        crate::systems::text_constants::SaveLoadText::DELETE_SUCCESS,
                        save_name
                    );
                    // 閸掗攱鏌婄€涙ɑ銆傞崚妤勶拷?
                    should_refresh = true;
                }
                Err(e) => {
                    save_load_ui_state.error_message = e.to_string();
                    save_load_ui_state.status_message.clear();
                    crate::debug_log!(
                        "{}: {}",
                        crate::systems::text_constants::SaveLoadText::DELETE_ERROR,
                        e
                    );
                }
            }
        }
    }
    // 婢跺嫮鎮婇崚閿嬫煀
    else if should_refresh {
        if save_load_ui_state.is_busy {
            save_load_ui_state.status_message =
                "Operation in progress, refresh is temporarily disabled".to_string();
            return;
        }

        save_load_ui_state.pending_load_index = None;
        save_load_ui_state.error_message.clear();
        save_load_ui_state.status_message = "Save list refreshed".to_string();

        // 鐟欙箑褰傜€涙ɑ銆傞弬鍥︽閹殿偅寮块獮鍫曞櫢閺傛澘濮炴潪绲孖
        crate::systems::pause_save::scan_save_files(save_file_manager);
        next_state.set(GameState::LoadTable);
        crate::debug_log!("Refreshing save list and reloading UI");
    }
    // 婢跺嫮鎮婃潻鏂挎礀
    else if should_cancel {
        if save_load_ui_state.is_busy {
            save_load_ui_state.status_message = "Load is in progress, please wait...".to_string();
            return;
        }

        save_load_ui_state.pending_load_index = None;
        if !save_load_ui_state.is_busy {
            save_load_ui_state.status_message.clear();
            save_load_ui_state.error_message.clear();
        }

        // 閺嶈宓侀弶銉︾爱閻樿埖鈧浇绻戦崶鐐插煂濮濓絿鈥橀惃鍕勾锟?
        let target_state = loaded_game_state
            .previous_state
            .clone()
            .unwrap_or(GameState::Menu);
        next_state.set(target_state.clone());
        loaded_game_state.previous_state = None; // 濞撳懐鎮婇悩鑸碘偓?
        match target_state {
            GameState::Menu => crate::debug_log!("Back to main menu"),
            GameState::Paused => crate::debug_log!("Back to pause menu"),
            _ => crate::debug_log!("Back to previous state"),
        }
    }

    // Explicitly use should_refresh to silence the warning, as the compiler seems to miss its usage.
    let _ = should_refresh;
}

/// Enhanced text input handler for rename operations
pub fn handle_rename_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut rename_input: ResMut<RenameInput>,
    mut text_query: Query<&mut Text, With<RenameInputBox>>,
    text_input_state: ResMut<crate::systems::text_input::TextInputState>,
) {
    if !rename_input.is_editing {
        return;
    }

    // 閸氬本顒為弬鎵畱閺傚洦婀版潏鎾冲弳缁崵绮洪悩鑸碘偓浣稿煂闁插秴鎳￠崥宥堢翻锟?
    if text_input_state.is_active {
        rename_input.current_name = text_input_state.current_text.clone();

        // 閺囧瓨鏌婇弰鍓с仛閺傚洦锟?
        for mut text in text_query.iter_mut() {
            text.0 = if text_input_state.current_text.is_empty() {
                crate::systems::text_constants::SaveLoadText::NAME_PLACEHOLDER.to_string()
            } else {
                format!("{}|", text_input_state.current_text)
            };
        }
    }

    // 婢跺嫮鎮婇柨顔炬磸韫囶偅宓庨柨?
    if keyboard_input.just_pressed(KeyCode::Enter) && text_input_state.is_active {
        crate::debug_log!("Enter key pressed - confirming rename");
    } else if keyboard_input.just_pressed(KeyCode::Escape) && text_input_state.is_active {
        crate::debug_log!("Escape key pressed - canceling rename");
    }
}

/// 婢跺嫮鎮婇柌宥呮嚒閸氬秴顕拠婵囶攱娴溿倓锟?
pub fn handle_rename_dialog_interactions(
    mut interaction_query: RenameDialogInteractionQuery,
    mut next_state: ResMut<NextState<GameState>>,
    mut save_file_manager: ResMut<SaveFileManager>,
    mut rename_input: ResMut<RenameInput>,
    text_input_state: Res<crate::systems::text_input::TextInputState>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let mut should_confirm = false;
    let mut should_cancel = false;

    // 婢跺嫮鎮婇柨顔炬磸韫囶偅宓庨柨?
    if keyboard_input.just_pressed(KeyCode::Enter) && text_input_state.is_active {
        should_confirm = true;
        crate::debug_log!("Enter key pressed - confirming rename");
    } else if keyboard_input.just_pressed(KeyCode::Escape) && text_input_state.is_active {
        should_cancel = true;
        crate::debug_log!("Escape key pressed - canceling rename");
    }

    // 婢跺嫮鎮婇幐澶愭尦娴溿倓锟?
    for (interaction, mut color, confirm_btn, cancel_btn) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if confirm_btn.is_some() {
                    should_confirm = true;
                } else if cancel_btn.is_some() {
                    should_cancel = true;
                }
                *color = BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.9));
            }
            Interaction::Hovered => {
                if confirm_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.25, 0.5, 0.25, 0.9));
                } else if cancel_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.5, 0.25, 0.25, 0.9));
                }
            }
            Interaction::None => {
                if confirm_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.15, 0.3, 0.15, 0.8));
                } else if cancel_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.3, 0.15, 0.15, 0.8));
                }
            }
        }
    }

    if should_confirm {
        // 閹笛嗩攽闁插秴鎳￠崥宥嗘惙锟?
        let new_name = if text_input_state.current_text.is_empty() {
            rename_input.original_name.clone()
        } else {
            text_input_state.current_text.clone()
        };

        if rename_input.save_index < save_file_manager.save_files.len() {
            match crate::systems::pause_save::rename_save_file(
                &rename_input.original_name,
                &new_name,
                save_file_manager.as_mut(),
            ) {
                Ok(_) => {
                    crate::debug_log!(
                        "{}: {} -> {}",
                        crate::systems::text_constants::SaveLoadText::RENAME_SUCCESS,
                        rename_input.original_name,
                        new_name
                    );
                    rename_input.is_editing = false;
                    next_state.set(GameState::LoadTable);
                }
                Err(e) => {
                    crate::debug_log!(
                        "{}: {}",
                        crate::systems::text_constants::SaveLoadText::RENAME_ERROR,
                        e
                    );
                    next_state.set(GameState::LoadTable);
                }
            }
        }
    } else if should_cancel {
        // 閸欐牗绉烽柌宥呮嚒閸氬稄绱濇潻鏂挎礀閸旂姾娴囩悰銊︾壐
        rename_input.is_editing = false;
        next_state.set(GameState::LoadTable);
        crate::debug_log!("Rename cancelled");
    }
}

/// 婢跺嫮鎮婇弳鍌氫粻閼挎粌宕熼幐澶愭尦娴溿倓锟?
pub fn handle_pause_menu_interactions(
    mut interaction_query: PauseMenuInteractionQuery,
    mut next_state: ResMut<NextState<GameState>>,
    mut pause_manager: ResMut<PauseManager>,
    mut loaded_game_state: ResMut<LoadedGameState>,
    mut save_load_ui_state: ResMut<SaveLoadUiState>,
) {
    for (interaction, mut color, resume_btn, save_btn, load_btn, menu_btn, esc_btn, q_btn) in
        &mut interaction_query
    {
        match *interaction {
            Interaction::Pressed => {
                if resume_btn.is_some() || esc_btn.is_some() {
                    // 缂佈呯敾濞撳憡锟?
                    next_state.set(GameState::Playing);
                    crate::debug_log!("Resume game");
                } else if save_btn.is_some() {
                    // 鏉╂稑鍙嗘穱婵嗙摠鐎电鐦藉?
                    next_state.set(GameState::SaveDialog);
                    crate::debug_log!("Open save dialog");
                } else if load_btn.is_some() {
                    // 鏉╂稑鍙嗛崝鐘烘祰鐞涖劍鐗搁敍宀冾唶瑜版洘娼靛┃鎰Ц锟?
                    loaded_game_state.previous_state = Some(GameState::Paused);
                    save_load_ui_state.pending_load_index = None;
                    save_load_ui_state.error_message.clear();
                    if !save_load_ui_state.is_busy {
                        save_load_ui_state.status_message.clear();
                    }
                    next_state.set(GameState::LoadTable);
                    crate::debug_log!("Open load table from pause menu");
                } else if menu_btn.is_some() || q_btn.is_some() {
                    // 鏉╂柨娲栨稉鏄忓綅锟?
                    pause_manager.resume_game(); // Clear paused snapshot state
                    save_load_ui_state.pending_load_index = None;
                    save_load_ui_state.is_busy = false;
                    next_state.set(GameState::Menu);
                    crate::debug_log!("Return to main menu");
                }
                *color = BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.9));
            }
            Interaction::Hovered => {
                if resume_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.25, 0.5, 0.25, 0.9));
                } else if save_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.25, 0.25, 0.5, 0.9));
                } else if load_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.25, 0.5, 0.25, 0.9));
                } else if menu_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.5, 0.25, 0.25, 0.9));
                } else if esc_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.3, 0.6, 0.3, 0.9));
                } else if q_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.6, 0.3, 0.3, 0.9));
                }
            }
            Interaction::None => {
                if resume_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.15, 0.3, 0.15, 0.8));
                } else if save_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.15, 0.15, 0.3, 0.8));
                } else if load_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.15, 0.3, 0.15, 0.8));
                } else if menu_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.3, 0.15, 0.15, 0.8));
                } else if esc_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.2, 0.4, 0.2, 0.8));
                } else if q_btn.is_some() {
                    *color = BackgroundColor(Color::srgba(0.4, 0.2, 0.2, 0.8));
                }
            }
        }
    }
}
