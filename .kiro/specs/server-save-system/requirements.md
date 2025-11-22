# Requirements Document

## Introduction

This document specifies the requirements for integrating the game save system into the server-side PostgreSQL database for a multiplayer online game. The system will migrate from local JSON file storage to centralized server-based storage, enabling persistent player data across sessions and devices, similar to games like DNF, Dota2, LoL, and CS2.

## Glossary

- **Client**: The game application running on the player's device
- **Server**: The backend application managing game state and database operations
- **GameState**: The complete snapshot of a player's game progress including position, stats, and character data
- **SaveSlot**: A named storage location for a player's game progress
- **PlayerSession**: A single gameplay session from start to end
- **WebSocket**: The real-time communication protocol between Client and Server
- **PostgreSQL**: The relational database system storing all persistent game data

## Requirements

### Requirement 1

**User Story:** As a player, I want my game progress automatically saved to the server, so that I can resume from any device without manual save operations

#### Acceptance Criteria

1. WHEN a player completes a gameplay action THEN the Server SHALL persist the updated GameState to PostgreSQL within 5 seconds
2. WHEN a player's connection is interrupted THEN the Server SHALL preserve the last known GameState in PostgreSQL
3. WHEN a player reconnects THEN the Server SHALL retrieve the most recent GameState from PostgreSQL and transmit it to the Client
4. WHILE a player is actively playing THEN the Server SHALL automatically save GameState every 30 seconds
5. WHEN a GameState save operation fails THEN the Server SHALL retry up to 3 times before logging an error

### Requirement 2

**User Story:** As a player, I want to manage multiple save slots, so that I can maintain different game progressions simultaneously

#### Acceptance Criteria

1. WHEN a player creates a new save slot THEN the Server SHALL store it in PostgreSQL with a unique identifier and timestamp
2. WHEN a player requests their save slots THEN the Server SHALL return all SaveSlots ordered by most recent update time
3. WHEN a player selects a save slot THEN the Server SHALL load the associated GameState from PostgreSQL
4. WHEN a player deletes a save slot THEN the Server SHALL remove it from PostgreSQL and confirm deletion to the Client
5. WHEN a player renames a save slot THEN the Server SHALL update the save_name field in PostgreSQL

### Requirement 3

**User Story:** As a player, I want my character position and state restored exactly when loading a save, so that I can continue from where I left off

#### Acceptance Criteria

1. WHEN a save slot is loaded THEN the Server SHALL restore player_position, player_velocity, and player_animation_state from PostgreSQL
2. WHEN a save slot is loaded THEN the Server SHALL restore camera_position and camera_target from PostgreSQL
3. WHEN a save slot is loaded THEN the Server SHALL restore game statistics including distance_traveled, jump_count, and play_time from PostgreSQL
4. WHEN a save slot is loaded THEN the Server SHALL restore selected_character and audio settings from PostgreSQL
5. WHEN GameState restoration completes THEN the Server SHALL transmit the complete state to the Client via WebSocket

### Requirement 4

**User Story:** As a developer, I want to remove local JSON file save operations from the client, so that all persistence is centralized on the server

#### Acceptance Criteria

1. WHEN the system is refactored THEN the Client SHALL NOT write any save data to local files
2. WHEN the system is refactored THEN the Client SHALL communicate all save/load requests to the Server via WebSocket
3. WHEN the system is refactored THEN the Server SHALL handle all database operations for save/load functionality
4. WHEN the refactoring is complete THEN the systems in src/systems/save.rs SHALL be removed or marked as deprecated
5. WHEN the refactoring is complete THEN the pause_save.rs local file operations SHALL be replaced with server API calls

### Requirement 5

**User Story:** As a player, I want my game session statistics tracked, so that I can review my gameplay history

#### Acceptance Criteria

1. WHEN a player starts a game THEN the Server SHALL create a new GameSession record in PostgreSQL
2. WHEN a player performs actions THEN the Server SHALL log PlayerAction records with timestamps and positions to PostgreSQL
3. WHEN a player ends a session THEN the Server SHALL update the GameSession end_time and final statistics in PostgreSQL
4. WHEN a player requests session history THEN the Server SHALL retrieve GameSession records from PostgreSQL ordered by start_time
5. WHILE a session is active THEN the Server SHALL update session statistics in real-time

### Requirement 6

**User Story:** As a system administrator, I want data integrity guarantees, so that player progress is never lost due to system failures

#### Acceptance Criteria

1. WHEN a database write operation occurs THEN the Server SHALL use PostgreSQL transactions to ensure atomicity
2. WHEN concurrent save operations occur for the same player THEN the Server SHALL use database locks to prevent race conditions
3. WHEN a save operation completes THEN the Server SHALL verify the data was written successfully before acknowledging to the Client
4. IF a database connection fails THEN the Server SHALL attempt to reconnect and queue pending operations
5. WHEN critical data is saved THEN the Server SHALL include checksums for corruption detection

### Requirement 7

**User Story:** As a developer, I want a unified protocol for save/load operations, so that Client and Server communicate efficiently

#### Acceptance Criteria

1. WHEN the Client requests a save THEN the Client SHALL send a SaveGameRequest packet via WebSocket containing the save_name
2. WHEN the Server completes a save THEN the Server SHALL send a SaveGameResponse packet with success status and save metadata
3. WHEN the Client requests available saves THEN the Client SHALL send a ListSavesRequest packet via WebSocket
4. WHEN the Server responds to list saves THEN the Server SHALL send a ListSavesResponse packet containing all SaveFileMetadata
5. WHEN the Client requests to load a save THEN the Client SHALL send a LoadGameRequest packet with the save_name, and the Server SHALL respond with LoadGameResponse containing the complete GameState

### Requirement 8

**User Story:** As a player, I want seamless migration from local saves to server saves, so that my existing progress is not lost

#### Acceptance Criteria

1. WHEN the system detects local save files THEN the Client SHALL offer to upload them to the Server
2. WHEN a player confirms migration THEN the Client SHALL transmit local SaveFileData to the Server via WebSocket
3. WHEN the Server receives migrated data THEN the Server SHALL convert and store it in PostgreSQL with the original timestamps
4. WHEN migration completes THEN the Client SHALL archive local save files and display a confirmation message
5. IF migration fails THEN the Client SHALL preserve local files and allow retry
