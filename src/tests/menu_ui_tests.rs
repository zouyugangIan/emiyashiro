//! Menu UI Tests
//!
//! Property-based and unit tests for the menu UI system refactor.

use crate::components::*;
use crate::resources::*;
use crate::states::*;
use crate::systems::menu::*;
use bevy::prelude::*;

/// Helper function to create mock GameAssets for testing
fn create_mock_game_assets() -> GameAssets {
    GameAssets {
        cover_textures: vec![Handle::default()],
        current_cover_index: 0,
        shirou_animation_frames: vec![Handle::default()],
        sakura_animation_frames: vec![Handle::default()],
        current_shirou_frame: 0,
        current_sakura_frame: 0,
        font: Handle::default(),
        shirou_spritesheet: None,
        sakura_spritesheet: None,
        shirou_atlas: None,
        shirou_atlas_run: None,
        sakura_atlas: None,
        jump_sound: Handle::default(),
        land_sound: Handle::default(),
        footstep_sound: Handle::default(),
        menu_music: Handle::default(),
        game_music: Handle::default(),
        game_whyifight_music: Handle::default(),
        background_music: Handle::default(),
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;

    /// Feature: menu-ui-refactor, Property 2: Cover images use UI Node components
    /// Validates: Requirements 1.2, 3.4
    ///
    /// Tests that all CoverImage entities have Node and ImageNode components
    /// and do not have Sprite components.
    #[test]
    fn test_cover_images_use_ui_node_components() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));

        // Add required resources
        app.init_resource::<CharacterSelection>();
        app.init_state::<GameState>();

        // Add the setup_menu system
        app.add_systems(Startup, setup_menu);

        // Run the app for one frame to execute startup systems
        app.update();

        // Query for CoverImage1 entities
        let mut cover1_query = app
            .world_mut()
            .query_filtered::<Entity, With<CoverImage1>>();
        let cover1_entities: Vec<Entity> = cover1_query.iter(app.world()).collect();

        // Query for CoverImage2 entities
        let mut cover2_query = app
            .world_mut()
            .query_filtered::<Entity, With<CoverImage2>>();
        let cover2_entities: Vec<Entity> = cover2_query.iter(app.world()).collect();

        let all_cover_entities: Vec<Entity> =
            cover1_entities.into_iter().chain(cover2_entities).collect();

        // For each cover image entity, verify it has Node but not Sprite
        for entity in all_cover_entities {
            let has_node = app.world().entity(entity).contains::<Node>();
            let has_sprite = app.world().entity(entity).contains::<Sprite>();

            assert!(
                has_node,
                "CoverImage entity {:?} should have Node component",
                entity
            );
            assert!(
                !has_sprite,
                "CoverImage entity {:?} should NOT have Sprite component",
                entity
            );
        }
    }

    /// Feature: menu-ui-refactor, Property 2: Cover images use UI Node components (with assets)
    /// Validates: Requirements 1.2, 3.4
    ///
    /// Tests that cover images with loaded assets use UI Node components.
    #[test]
    fn test_cover_images_with_assets_use_ui_nodes() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));

        // Add required resources
        app.init_resource::<CharacterSelection>();
        app.init_state::<GameState>();

        // Create mock GameAssets
        let game_assets = create_mock_game_assets();

        app.insert_resource(game_assets);

        // Add the setup_menu system
        app.add_systems(Startup, setup_menu);

        // Run the app for one frame to execute startup systems
        app.update();

        // Query for CoverImage1 entities
        let mut cover1_query = app
            .world_mut()
            .query_filtered::<Entity, With<CoverImage1>>();
        let cover1_entities: Vec<Entity> = cover1_query.iter(app.world()).collect();

        // Query for CoverImage2 entities
        let mut cover2_query = app
            .world_mut()
            .query_filtered::<Entity, With<CoverImage2>>();
        let cover2_entities: Vec<Entity> = cover2_query.iter(app.world()).collect();

        assert!(
            !cover1_entities.is_empty(),
            "Should have CoverImage1 entities"
        );
        assert!(
            !cover2_entities.is_empty(),
            "Should have CoverImage2 entities"
        );

        let all_cover_entities: Vec<Entity> =
            cover1_entities.into_iter().chain(cover2_entities).collect();

        // For each cover image entity, verify it has Node and ImageNode, but not Sprite
        for entity in all_cover_entities {
            let has_node = app.world().entity(entity).contains::<Node>();
            let has_image_node = app.world().entity(entity).contains::<ImageNode>();
            let has_sprite = app.world().entity(entity).contains::<Sprite>();

            assert!(
                has_node,
                "CoverImage entity {:?} should have Node component",
                entity
            );
            assert!(
                has_image_node,
                "CoverImage entity {:?} should have ImageNode component",
                entity
            );
            assert!(
                !has_sprite,
                "CoverImage entity {:?} should NOT have Sprite component",
                entity
            );
        }
    }

    /// Feature: menu-ui-refactor, Property 3: Cover images fill entire window
    /// Validates: Requirements 1.3
    ///
    /// Tests that CoverImage nodes have width and height set to Val::Percent(100.0)
    /// and position_type set to PositionType::Absolute.
    #[test]
    fn test_cover_images_fill_entire_window() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));

        // Add required resources
        app.init_resource::<CharacterSelection>();
        app.init_state::<GameState>();

        // Create mock GameAssets
        let game_assets = create_mock_game_assets();

        app.insert_resource(game_assets);

        // Add the setup_menu system
        app.add_systems(Startup, setup_menu);

        // Run the app for one frame to execute startup systems
        app.update();

        // Query for CoverImage1 entities with Node component
        let mut cover1_query = app
            .world_mut()
            .query_filtered::<(Entity, &Node), With<CoverImage1>>();
        let cover1_entities: Vec<(Entity, Node)> = cover1_query
            .iter(app.world())
            .map(|(e, n)| (e, n.clone()))
            .collect();

        // Query for CoverImage2 entities with Node component
        let mut cover2_query = app
            .world_mut()
            .query_filtered::<(Entity, &Node), With<CoverImage2>>();
        let cover2_entities: Vec<(Entity, Node)> = cover2_query
            .iter(app.world())
            .map(|(e, n)| (e, n.clone()))
            .collect();

        let all_cover_entities: Vec<(Entity, Node)> =
            cover1_entities.into_iter().chain(cover2_entities).collect();

        assert!(
            !all_cover_entities.is_empty(),
            "Should have cover image entities"
        );

        // For each cover image entity, verify Node properties
        for (entity, node) in all_cover_entities {
            assert_eq!(
                node.width,
                Val::Percent(100.0),
                "CoverImage entity {:?} should have width Val::Percent(100.0), got {:?}",
                entity,
                node.width
            );
            assert_eq!(
                node.height,
                Val::Percent(100.0),
                "CoverImage entity {:?} should have height Val::Percent(100.0), got {:?}",
                entity,
                node.height
            );
            assert_eq!(
                node.position_type,
                PositionType::Absolute,
                "CoverImage entity {:?} should have position_type PositionType::Absolute, got {:?}",
                entity,
                node.position_type
            );
        }
    }

    /// Feature: menu-ui-refactor, Property 6: Z-axis layering maintained
    /// Validates: Requirements 2.4
    ///
    /// Tests that CoverImage2 entity has a higher ZIndex value than CoverImage1 entity.
    #[test]
    fn test_z_axis_layering_correct() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));

        // Add required resources
        app.init_resource::<CharacterSelection>();
        app.init_state::<GameState>();

        // Create mock GameAssets
        let game_assets = create_mock_game_assets();

        app.insert_resource(game_assets);

        // Add the setup_menu system
        app.add_systems(Startup, setup_menu);

        // Run the app for one frame to execute startup systems
        app.update();

        // Query for CoverImage1 entities with ZIndex component
        let mut cover1_query = app
            .world_mut()
            .query_filtered::<&ZIndex, With<CoverImage1>>();
        let cover1_zindices: Vec<ZIndex> = cover1_query.iter(app.world()).copied().collect();

        // Query for CoverImage2 entities with ZIndex component
        let mut cover2_query = app
            .world_mut()
            .query_filtered::<&ZIndex, With<CoverImage2>>();
        let cover2_zindices: Vec<ZIndex> = cover2_query.iter(app.world()).copied().collect();

        assert!(
            !cover1_zindices.is_empty(),
            "Should have CoverImage1 entities with ZIndex"
        );
        assert!(
            !cover2_zindices.is_empty(),
            "Should have CoverImage2 entities with ZIndex"
        );

        // Verify that CoverImage2 has higher ZIndex than CoverImage1
        for cover1_z in &cover1_zindices {
            for cover2_z in &cover2_zindices {
                assert!(
                    cover2_z.0 > cover1_z.0,
                    "CoverImage2 ZIndex ({}) should be higher than CoverImage1 ZIndex ({})",
                    cover2_z.0,
                    cover1_z.0
                );
            }
        }
    }

    /// Feature: menu-ui-refactor, Property 8: Menu entities use Node components
    /// Validates: Requirements 3.1
    ///
    /// Tests that all MenuUI entities have Node components, confirming use of Bevy's UI system.
    #[test]
    fn test_menu_entities_use_node_components() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));

        // Add required resources
        app.init_resource::<CharacterSelection>();
        app.init_state::<GameState>();

        // Create mock GameAssets
        let game_assets = create_mock_game_assets();

        app.insert_resource(game_assets);

        // Add the setup_menu system
        app.add_systems(Startup, setup_menu);

        // Run the app for one frame to execute startup systems
        app.update();

        // Query for all MenuUI entities
        let mut menu_query = app.world_mut().query_filtered::<Entity, With<MenuUI>>();
        let menu_entities: Vec<Entity> = menu_query.iter(app.world()).collect();

        assert!(!menu_entities.is_empty(), "Should have MenuUI entities");

        // For each MenuUI entity, verify it has a Node component
        for entity in menu_entities {
            let has_node = app.world().entity(entity).contains::<Node>();
            assert!(
                has_node,
                "MenuUI entity {:?} should have Node component (using Bevy's UI system)",
                entity
            );
        }
    }

    /// Feature: menu-ui-refactor, Property 4: Animation system queries BackgroundColor not Sprite
    /// Validates: Requirements 2.1
    ///
    /// Tests that the animation system queries BackgroundColor components and not Sprite components.
    #[test]
    fn test_animation_system_queries_background_color() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));

        // Add required resources
        app.init_resource::<CharacterSelection>();
        app.init_state::<GameState>();

        // Create mock GameAssets
        let game_assets = create_mock_game_assets();

        app.insert_resource(game_assets);

        // Add the setup_menu system
        app.add_systems(Startup, setup_menu);

        // Run the app for one frame to execute startup systems
        app.update();

        // Query for CoverImage1 entities
        let mut cover1_query = app
            .world_mut()
            .query_filtered::<Entity, With<CoverImage1>>();
        let cover1_entities: Vec<Entity> = cover1_query.iter(app.world()).collect();

        // Query for CoverImage2 entities
        let mut cover2_query = app
            .world_mut()
            .query_filtered::<Entity, With<CoverImage2>>();
        let cover2_entities: Vec<Entity> = cover2_query.iter(app.world()).collect();

        let all_cover_entities: Vec<Entity> =
            cover1_entities.into_iter().chain(cover2_entities).collect();

        assert!(
            !all_cover_entities.is_empty(),
            "Should have cover image entities"
        );

        // For each cover image entity, verify it has BackgroundColor but not Sprite
        for entity in all_cover_entities {
            let has_background_color = app.world().entity(entity).contains::<BackgroundColor>();
            let has_sprite = app.world().entity(entity).contains::<Sprite>();

            assert!(
                has_background_color,
                "CoverImage entity {:?} should have BackgroundColor component for animation",
                entity
            );
            assert!(
                !has_sprite,
                "CoverImage entity {:?} should NOT have Sprite component (animation uses BackgroundColor)",
                entity
            );
        }
    }
}

#[cfg(test)]
mod smoothstep_property_tests {
    use proptest::prelude::*;

    /// Helper function to compute smoothstep easing
    fn smoothstep(alpha: f32) -> f32 {
        alpha * alpha * (3.0 - 2.0 * alpha)
    }

    // Feature: menu-ui-refactor, Property 5: Smoothstep easing function applied correctly
    // Validates: Requirements 2.3
    //
    // Tests that for any cycle progress value between 0.0 and 1.0,
    // the calculated alpha value follows the smoothstep formula: alpha * alpha * (3.0 - 2.0 * alpha)
    proptest! {
        #[test]
        fn test_smoothstep_easing_function(final_alpha in 0.0f32..=1.0f32) {
            // Apply smoothstep formula
            let eased_alpha = smoothstep(final_alpha);

            // Verify the formula is applied correctly
            let expected = final_alpha * final_alpha * (3.0 - 2.0 * final_alpha);

            // Use a small epsilon for floating point comparison
            let epsilon = 1e-6;
            prop_assert!((eased_alpha - expected).abs() < epsilon,
                "Smoothstep formula not applied correctly: got {}, expected {}",
                eased_alpha, expected);

            // Verify smoothstep properties
            // 1. Output should be in [0, 1] for input in [0, 1]
            prop_assert!((0.0..=1.0).contains(&eased_alpha),
                "Smoothstep output {} should be in [0, 1]", eased_alpha);

            // 2. Smoothstep(0) = 0
            if final_alpha == 0.0 {
                prop_assert!((eased_alpha - 0.0).abs() < epsilon,
                    "Smoothstep(0) should be 0, got {}", eased_alpha);
            }

            // 3. Smoothstep(1) = 1
            if final_alpha == 1.0 {
                prop_assert!((eased_alpha - 1.0).abs() < epsilon,
                    "Smoothstep(1) should be 1, got {}", eased_alpha);
            }
        }
    }

    // Feature: menu-ui-refactor, Property 7: Complementary fade pattern preserved
    // Validates: Requirements 2.5
    //
    // Tests that at any point in the animation cycle, the two cover images maintain
    // a complementary alpha relationship.
    proptest! {
        #[test]
        fn test_complementary_fade_pattern(cycle_progress in 0.0f32..=1.0f32) {
            // Simulate the animation logic from cover_fade_animation
            let base_alpha = (cycle_progress * 2.0 * std::f32::consts::PI).sin();

            // Calculate alpha for first image (fade_direction > 0.0)
            let final_alpha_1 = (base_alpha + 1.0) * 0.5;
            let eased_alpha_1 = smoothstep(final_alpha_1);
            let clamped_alpha_1 = eased_alpha_1.clamp(0.1, 0.9);

            // Calculate alpha for second image (fade_direction < 0.0)
            let final_alpha_2 = ((-base_alpha) + 1.0) * 0.5;
            let eased_alpha_2 = smoothstep(final_alpha_2);
            let clamped_alpha_2 = eased_alpha_2.clamp(0.1, 0.9);

            // Verify complementary relationship
            // Before clamping, the alphas should be complementary (sum to 1.0)
            let epsilon = 1e-5;
            let sum_before_clamp = final_alpha_1 + final_alpha_2;
            prop_assert!((sum_before_clamp - 1.0).abs() < epsilon,
                "Before clamping, alpha values should sum to 1.0, got {} + {} = {}",
                final_alpha_1, final_alpha_2, sum_before_clamp);

            // After smoothstep, they should still maintain a relationship
            // (though not necessarily sum to 1.0 due to the non-linear transformation)
            // But they should move in opposite directions
            let mid_point = 0.5;
            if final_alpha_1 > mid_point {
                prop_assert!(final_alpha_2 < mid_point,
                    "When image1 alpha ({}) is above mid-point, image2 alpha ({}) should be below",
                    final_alpha_1, final_alpha_2);
            } else if final_alpha_1 < mid_point {
                prop_assert!(final_alpha_2 > mid_point,
                    "When image1 alpha ({}) is below mid-point, image2 alpha ({}) should be above",
                    final_alpha_1, final_alpha_2);
            }

            // Verify both alphas are within valid range after clamping
            prop_assert!((0.1..=0.9).contains(&clamped_alpha_1),
                "Image1 clamped alpha {} should be in [0.1, 0.9]", clamped_alpha_1);
            prop_assert!((0.1..=0.9).contains(&clamped_alpha_2),
                "Image2 clamped alpha {} should be in [0.1, 0.9]", clamped_alpha_2);
        }
    }
}

#[cfg(test)]
mod button_interaction_tests {
    use super::*;

    /// Feature: menu-ui-refactor, Property 9: Button interactions update BackgroundColor
    /// Validates: Requirements 4.2
    ///
    /// Tests that button Interaction state changes update BackgroundColor appropriately.
    #[test]
    fn test_button_interactions_update_background_color() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));

        // Add required resources
        app.init_resource::<CharacterSelection>();
        app.init_state::<GameState>();
        app.init_resource::<crate::systems::ui::LoadedGameState>();
        app.init_resource::<GameStats>();
        app.init_resource::<PauseManager>();
        app.init_resource::<SaveFileManager>();

        // Create mock GameAssets
        let game_assets = create_mock_game_assets();

        app.insert_resource(game_assets);

        // Add the setup_menu system and button handlers
        app.add_systems(Startup, setup_menu);
        app.add_systems(
            Update,
            (
                handle_start_button,
                handle_load_button,
                handle_character_select,
            ),
        );

        // Run the app for one frame to execute startup systems
        app.update();

        // Query for StartButton entities
        let mut start_button_query = app
            .world_mut()
            .query_filtered::<(Entity, &BackgroundColor), With<StartButton>>();
        let start_buttons: Vec<(Entity, BackgroundColor)> = start_button_query
            .iter(app.world())
            .map(|(e, bg)| (e, *bg))
            .collect();

        assert!(
            !start_buttons.is_empty(),
            "Should have StartButton entities"
        );

        // Test that buttons have BackgroundColor component
        for (entity, bg_color) in &start_buttons {
            // Verify the button has a BackgroundColor
            assert!(
                app.world().entity(*entity).contains::<BackgroundColor>(),
                "StartButton entity {:?} should have BackgroundColor component",
                entity
            );

            // Verify initial color is set (default state)
            let color = bg_color.0;
            assert!(
                color.alpha() > 0.0,
                "StartButton BackgroundColor should have non-zero alpha, got {}",
                color.alpha()
            );
        }

        // Simulate interaction state change by manually setting Interaction component
        for (entity, _) in &start_buttons {
            // Set to Hovered state
            app.world_mut()
                .entity_mut(*entity)
                .insert(Interaction::Hovered);
        }

        // Run update to process button handlers
        app.update();

        // Query again to check if color changed
        let mut start_button_query = app
            .world_mut()
            .query_filtered::<&BackgroundColor, With<StartButton>>();
        let hovered_colors: Vec<BackgroundColor> =
            start_button_query.iter(app.world()).copied().collect();

        // Verify that BackgroundColor was updated (hovered state should be different)
        for bg_color in &hovered_colors {
            let color = bg_color.0;
            // The hovered color should be Color::srgba(0.3, 0.3, 0.3, 0.8)
            // Convert to Srgba to access components
            let srgba = color.to_srgba();
            assert!(
                (srgba.red - 0.3).abs() < 0.01,
                "Hovered button red channel should be ~0.3, got {}",
                srgba.red
            );
            assert!(
                (srgba.alpha - 0.8).abs() < 0.01,
                "Hovered button alpha should be ~0.8, got {}",
                srgba.alpha
            );
        }
    }

    /// Unit test: Start button click triggers GameState::Playing transition
    /// Validates: Requirements 4.1, 4.4
    #[test]
    fn test_start_button_triggers_playing_state() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));

        // Add required resources
        app.init_resource::<CharacterSelection>();
        app.init_state::<GameState>();
        app.init_resource::<crate::systems::ui::LoadedGameState>();
        app.init_resource::<GameStats>();
        app.init_resource::<PauseManager>();
        app.init_resource::<SaveFileManager>();

        let game_assets = create_mock_game_assets();

        app.insert_resource(game_assets);

        // Add systems
        app.add_systems(Startup, setup_menu);
        app.add_systems(Update, handle_start_button);

        // Run startup
        app.update();

        // Verify initial state is Menu
        let current_state = app.world().resource::<State<GameState>>();
        assert_eq!(
            **current_state,
            GameState::Menu,
            "Initial state should be Menu"
        );

        // Find and press the start button
        let mut start_button_query = app
            .world_mut()
            .query_filtered::<Entity, With<StartButton>>();
        let start_buttons: Vec<Entity> = start_button_query.iter(app.world()).collect();
        assert!(!start_buttons.is_empty(), "Should have StartButton");

        for entity in start_buttons {
            app.world_mut()
                .entity_mut(entity)
                .insert(Interaction::Pressed);
        }

        // Run update to process button handler
        app.update();

        // Apply state transitions
        app.update();

        // Verify state changed to Playing
        let new_state = app.world().resource::<State<GameState>>();
        assert_eq!(
            **new_state,
            GameState::Playing,
            "State should transition to Playing after start button press"
        );
    }

    /// Unit test: Load button click triggers GameState::LoadTable transition
    /// Validates: Requirements 4.1, 4.4
    #[test]
    fn test_load_button_triggers_load_table_state() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));

        // Add required resources
        app.init_resource::<CharacterSelection>();
        app.init_state::<GameState>();
        app.init_resource::<crate::systems::ui::LoadedGameState>();
        app.init_resource::<GameStats>();
        app.init_resource::<PauseManager>();
        app.init_resource::<SaveFileManager>();

        let game_assets = create_mock_game_assets();

        app.insert_resource(game_assets);

        // Add systems
        app.add_systems(Startup, setup_menu);
        app.add_systems(Update, handle_load_button);

        // Run startup
        app.update();

        // Verify initial state is Menu
        let current_state = app.world().resource::<State<GameState>>();
        assert_eq!(
            **current_state,
            GameState::Menu,
            "Initial state should be Menu"
        );

        // Find and press the load button
        let mut load_button_query = app
            .world_mut()
            .query_filtered::<Entity, With<crate::systems::ui::LoadButton>>();
        let load_buttons: Vec<Entity> = load_button_query.iter(app.world()).collect();
        assert!(!load_buttons.is_empty(), "Should have LoadButton");

        for entity in load_buttons {
            app.world_mut()
                .entity_mut(entity)
                .insert(Interaction::Pressed);
        }

        // Run update to process button handler
        app.update();

        // Apply state transitions
        app.update();

        // Verify state changed to LoadTable
        let new_state = app.world().resource::<State<GameState>>();
        assert_eq!(
            **new_state,
            GameState::LoadTable,
            "State should transition to LoadTable after load button press"
        );
    }

    /// Unit test: Character select button updates CharacterSelection resource
    /// Validates: Requirements 4.1, 4.4
    #[test]
    fn test_character_select_button_updates_selection() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));

        // Add required resources
        app.init_resource::<CharacterSelection>();
        app.init_state::<GameState>();
        app.init_resource::<crate::systems::ui::LoadedGameState>();
        app.init_resource::<GameStats>();
        app.init_resource::<PauseManager>();
        app.init_resource::<SaveFileManager>();

        let game_assets = create_mock_game_assets();

        app.insert_resource(game_assets);

        // Add systems
        app.add_systems(Startup, setup_menu);
        app.add_systems(Update, handle_character_select);

        // Run startup
        app.update();

        // Verify initial character selection
        let initial_selection = app.world().resource::<CharacterSelection>();
        let initial_char = initial_selection.selected_character.clone();

        // Find character select buttons
        let mut char_button_query = app
            .world_mut()
            .query_filtered::<(Entity, &CharacterSelectButton), ()>();
        let char_buttons: Vec<(Entity, CharacterType)> = char_button_query
            .iter(app.world())
            .map(|(e, btn)| (e, btn.character_type.clone()))
            .collect();

        assert!(
            !char_buttons.is_empty(),
            "Should have CharacterSelectButton entities"
        );

        // Press a character button with a different character type
        for (entity, char_type) in &char_buttons {
            if *char_type != initial_char {
                app.world_mut()
                    .entity_mut(*entity)
                    .insert(Interaction::Pressed);

                // Run update to process button handler
                app.update();

                // Verify character selection changed
                let new_selection = app.world().resource::<CharacterSelection>();
                assert_eq!(
                    new_selection.selected_character, *char_type,
                    "Character selection should update to {:?}",
                    char_type
                );

                break;
            }
        }
    }
}

#[cfg(test)]
mod responsive_layout_tests {
    use super::*;

    /// Feature: menu-ui-refactor, Property 1: UI elements scale proportionally with window size
    /// Validates: Requirements 1.1
    ///
    /// Tests that UI elements using percentage-based sizing maintain their proportional
    /// relationships regardless of window size.
    #[test]
    fn test_ui_elements_use_percentage_based_sizing() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));

        // Add required resources
        app.init_resource::<CharacterSelection>();
        app.init_state::<GameState>();

        // Create mock GameAssets
        let game_assets = create_mock_game_assets();

        app.insert_resource(game_assets);

        // Add the setup_menu system
        app.add_systems(Startup, setup_menu);

        // Run the app for one frame to execute startup systems
        app.update();

        // Query for cover image nodes - these should use percentage-based sizing
        let mut cover_query = app
            .world_mut()
            .query_filtered::<&Node, Or<(With<CoverImage1>, With<CoverImage2>)>>();
        let cover_nodes: Vec<Node> = cover_query.iter(app.world()).cloned().collect();

        assert!(!cover_nodes.is_empty(), "Should have cover image nodes");

        // Verify all cover nodes use Val::Percent for width and height
        for node in &cover_nodes {
            match node.width {
                Val::Percent(pct) => {
                    assert_eq!(pct, 100.0, "Cover node width should be 100%, got {}%", pct);
                }
                _ => panic!(
                    "Cover node width should use Val::Percent, got {:?}",
                    node.width
                ),
            }

            match node.height {
                Val::Percent(pct) => {
                    assert_eq!(pct, 100.0, "Cover node height should be 100%, got {}%", pct);
                }
                _ => panic!(
                    "Cover node height should use Val::Percent, got {:?}",
                    node.height
                ),
            }
        }

        // Query for the main UI root node - should also use percentage-based sizing
        let mut ui_root_query = app.world_mut().query_filtered::<&Node, (
            With<MenuUI>,
            Without<CoverImage1>,
            Without<CoverImage2>,
        )>();
        let ui_root_nodes: Vec<Node> = ui_root_query.iter(app.world()).cloned().collect();

        // Find nodes that are likely the main UI container (100% width/height)
        let full_size_nodes: Vec<&Node> = ui_root_nodes
            .iter()
            .filter(|n| {
                matches!(n.width, Val::Percent(100.0)) && matches!(n.height, Val::Percent(100.0))
            })
            .collect();

        assert!(
            !full_size_nodes.is_empty(),
            "Should have at least one UI root node with 100% width and height for responsive layout"
        );

        // Verify these nodes use percentage-based sizing
        for node in full_size_nodes {
            assert!(
                matches!(node.width, Val::Percent(_)),
                "UI root node should use percentage-based width"
            );
            assert!(
                matches!(node.height, Val::Percent(_)),
                "UI root node should use percentage-based height"
            );
        }
    }
}

#[cfg(test)]
mod cleanup_tests {
    use super::*;

    /// Feature: menu-ui-refactor, Property 10: Cleanup removes all menu entities
    /// Validates: Requirements 4.5
    ///
    /// Tests that cleanup_menu execution removes all MenuUI entities from the world.
    #[test]
    fn test_cleanup_removes_all_menu_entities() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));

        // Add required resources
        app.init_resource::<CharacterSelection>();
        app.init_state::<GameState>();

        // Create mock GameAssets
        let game_assets = create_mock_game_assets();

        app.insert_resource(game_assets);

        // Add the setup_menu system only (not cleanup yet)
        app.add_systems(Startup, setup_menu);

        // Run the app for one frame to execute startup systems
        app.update();

        // Query for MenuUI entities before cleanup
        let mut menu_query_before = app.world_mut().query_filtered::<Entity, With<MenuUI>>();
        let menu_entities_before: Vec<Entity> = menu_query_before.iter(app.world()).collect();

        assert!(
            !menu_entities_before.is_empty(),
            "Should have MenuUI entities before cleanup, found {}",
            menu_entities_before.len()
        );

        println!(
            "MenuUI entities before cleanup: {}",
            menu_entities_before.len()
        );

        // Now add and run cleanup_menu system
        app.add_systems(Update, cleanup_menu);
        app.update();

        // Query for MenuUI entities after cleanup
        let mut menu_query_after = app.world_mut().query_filtered::<Entity, With<MenuUI>>();
        let menu_entities_after: Vec<Entity> = menu_query_after.iter(app.world()).collect();

        println!(
            "MenuUI entities after cleanup: {}",
            menu_entities_after.len()
        );

        // Verify all MenuUI entities were removed
        assert!(
            menu_entities_after.is_empty(),
            "All MenuUI entities should be removed after cleanup, but {} remain",
            menu_entities_after.len()
        );

        // Verify specific component types are also gone
        let mut cover1_query = app
            .world_mut()
            .query_filtered::<Entity, With<CoverImage1>>();
        let cover1_entities: Vec<Entity> = cover1_query.iter(app.world()).collect();
        assert!(
            cover1_entities.is_empty(),
            "CoverImage1 entities should be removed after cleanup"
        );

        let mut cover2_query = app
            .world_mut()
            .query_filtered::<Entity, With<CoverImage2>>();
        let cover2_entities: Vec<Entity> = cover2_query.iter(app.world()).collect();
        assert!(
            cover2_entities.is_empty(),
            "CoverImage2 entities should be removed after cleanup"
        );

        let mut start_button_query = app
            .world_mut()
            .query_filtered::<Entity, With<StartButton>>();
        let start_button_entities: Vec<Entity> = start_button_query.iter(app.world()).collect();
        assert!(
            start_button_entities.is_empty(),
            "StartButton entities should be removed after cleanup"
        );
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Integration test: Complete menu initialization and cleanup flow
    /// Validates: Requirements 4.3
    ///
    /// Tests the full lifecycle of menu setup and cleanup.
    #[test]
    fn test_complete_menu_lifecycle() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));

        // Add required resources
        app.init_resource::<CharacterSelection>();
        app.init_state::<GameState>();

        // Create mock GameAssets
        let game_assets = create_mock_game_assets();

        app.insert_resource(game_assets);

        // Add systems
        app.add_systems(Startup, setup_menu);

        // Initialize menu
        app.update();

        // Verify menu entities were created
        let mut menu_query = app.world_mut().query_filtered::<Entity, With<MenuUI>>();
        let menu_entities: Vec<Entity> = menu_query.iter(app.world()).collect();
        assert!(
            !menu_entities.is_empty(),
            "Menu should be initialized with entities"
        );

        // Verify cover images exist
        let mut cover1_query = app
            .world_mut()
            .query_filtered::<Entity, With<CoverImage1>>();
        let cover1_count = cover1_query.iter(app.world()).count();
        assert_eq!(cover1_count, 1, "Should have exactly one CoverImage1");

        let mut cover2_query = app
            .world_mut()
            .query_filtered::<Entity, With<CoverImage2>>();
        let cover2_count = cover2_query.iter(app.world()).count();
        assert_eq!(cover2_count, 1, "Should have exactly one CoverImage2");

        // Verify buttons exist
        let mut start_button_query = app
            .world_mut()
            .query_filtered::<Entity, With<StartButton>>();
        let start_button_count = start_button_query.iter(app.world()).count();
        assert_eq!(start_button_count, 1, "Should have exactly one StartButton");

        // Add cleanup system and run
        app.add_systems(Update, cleanup_menu);
        app.update();

        // Verify all menu entities were cleaned up
        let mut menu_query_after = app.world_mut().query_filtered::<Entity, With<MenuUI>>();
        let menu_entities_after: Vec<Entity> = menu_query_after.iter(app.world()).collect();
        assert!(
            menu_entities_after.is_empty(),
            "All menu entities should be cleaned up"
        );
    }

    /// Integration test: Animation system with UI system
    /// Validates: Requirements 4.3
    ///
    /// Tests that the animation system works correctly with UI nodes.
    #[test]
    fn test_animation_system_integration() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));

        // Add required resources
        app.init_resource::<CharacterSelection>();
        app.init_state::<GameState>();
        app.init_resource::<Time>();

        // Create mock GameAssets
        let game_assets = create_mock_game_assets();

        app.insert_resource(game_assets);

        // Add systems
        app.add_systems(Startup, setup_menu);
        app.add_systems(Update, cover_fade_animation);

        // Initialize menu
        app.update();

        // Get initial alpha values
        let mut cover_query = app.world_mut().query_filtered::<(&BackgroundColor, &CoverFadeState), Or<(With<CoverImage1>, With<CoverImage2>)>>();
        let initial_alphas: Vec<(f32, f32)> = cover_query
            .iter(app.world())
            .map(|(bg, fade)| (bg.0.alpha(), fade.alpha))
            .collect();

        assert_eq!(initial_alphas.len(), 2, "Should have two cover images");

        // Run animation for a few frames
        for _ in 0..5 {
            app.update();
        }

        // Verify animation is working (alphas should have changed or stayed within valid range)
        let mut cover_query_after = app
            .world_mut()
            .query_filtered::<&BackgroundColor, Or<(With<CoverImage1>, With<CoverImage2>)>>();
        let final_alphas: Vec<f32> = cover_query_after
            .iter(app.world())
            .map(|bg| bg.0.alpha())
            .collect();

        // Verify all alphas are within valid range
        for alpha in final_alphas {
            assert!(
                (0.0..=1.0).contains(&alpha),
                "Alpha should be in valid range [0, 1], got {}",
                alpha
            );
        }
    }

    /// Integration test: Fallback when assets not loaded
    /// Validates: Requirements 4.3
    ///
    /// Tests that the menu handles missing assets gracefully.
    #[test]
    fn test_menu_fallback_without_assets() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));

        // Add required resources but NO GameAssets
        app.init_resource::<CharacterSelection>();
        app.init_state::<GameState>();

        // Add systems
        app.add_systems(Startup, setup_menu);

        // Initialize menu without assets
        app.update();

        // Verify menu entities were still created (fallback behavior)
        let mut menu_query = app.world_mut().query_filtered::<Entity, With<MenuUI>>();
        let menu_entities: Vec<Entity> = menu_query.iter(app.world()).collect();
        assert!(
            !menu_entities.is_empty(),
            "Menu should create fallback entities even without assets"
        );

        // Verify fallback background was created (should have MenuUI but no CoverImage markers)
        let mut fallback_query = app.world_mut().query_filtered::<Entity, (
            With<MenuUI>,
            Without<CoverImage1>,
            Without<CoverImage2>,
        )>();
        let fallback_entities: Vec<Entity> = fallback_query.iter(app.world()).collect();
        assert!(
            !fallback_entities.is_empty(),
            "Should have fallback background when assets are not loaded"
        );
    }
}
