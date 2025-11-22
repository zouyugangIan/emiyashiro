# Design Document

## Overview

This design document outlines the architecture for migrating the game save system from local JSON file storage to a centralized server-side PostgreSQL database. The system will support multiplayer online gameplay similar to DNF, Dota2, LoL, and CS2, where all player data is managed server-side for consistency, security, and cross-device accessibility.

The design integrates the existing `CompleteGameState` structure from `pause_save.rs` with the PostgreSQL database schema in `database/models.rs`, while removing redundant local file operations from `save.rs`. Communication between client and server will use WebSocket with binary serialization via bincode.

## Architecture

### High-Level Architecture

```
┌─────────────────┐         WebSocket          ┌─────────────────┐
│                 │    (bincode protocol)       │                 │
│  Game Client    │◄───────────────────────────►│  Game Server    │
│  (Bevy ECS)     │                             │  (Bevy ECS)     │
│                 │                             │                 │
└─────────────────┘                             └────────┬────────┘
                                                         │
                                                         │ sqlx
                                                         ▼
                                                ┌─────────────────┐
                                                │   PostgreSQL    │
                                                │    Database     │
                                                │                 │
                                                │  - players      │
                                                │  - save_games   │
                                                │  - game_sessions│
                                                │  - player_actions│
                                                └─────────────────┘
```

### Component Responsibilities

**Client Responsibilities:**
- Capture local game state (player position, velocity, animation state)
- Send save/load requests to server via WebSocket
- Receive and apply game state from server
- Display save slot UI and management interface
- Handle local migration of legacy JSON saves

**Server Responsibilities:**
- Authenticate and manage player sessions
- Process save/load requests from clients
- Perform all database operations (CRUD on save_games table)
- Broadcast game state updates to clients
- Maintain data integrity with transactions
- Auto-save player progress periodically

**Database Responsibilities:**
- Persist player accounts, save slots, and game states
- Maintain referential integrity between tables
- Provide ACID guarantees for transactions
- Store session history and player actions

## Components and Interfaces

### 1. Network Protocol Extension

Extend `src/protocol.rs` with new packet types for save/load operations:

```rust
// Server → Client packets
pub enum GamePacket {
    // ... existing variants ...
    
    /// Response to save request
    SaveGameResponse {
        success: bool,
        save_id: Option<Uuid>,
        error_message: Option<String>,
    },
    
    /// Response to load request
    LoadGameResponse {
        success: bool,
        game_state: Option<CompleteGameState>,
        error_message: Option<String>,
    },
    
    /// List of available save slots
    ListSavesResponse {
        saves: Vec<SaveFileMetadata>,
    },
    
    /// Confirmation of save deletion
    DeleteSaveResponse {
        success: bool,
        error_message: Option<String>,
    },
}

// Client → Server packets
pub enum PlayerAction {
    // ... existing variants ...
    
    /// Request to save current game state
    SaveGameRequest {
        save_name: String,
    },
    
    /// Request to load a save slot
    LoadGameRequest {
        save_name: String,
    },
    
    /// Request list of available saves
    ListSavesRequest,
    
    /// Request to delete a save slot
    DeleteSaveRequest {
        save_name: String,
    },
    
    /// Request to rename a save slot
    RenameSaveRequest {
        old_name: String,
        new_name: String,
    },
}
```

### 2. Server-Side Save Handler

Create `src/systems/server_save_handler.rs`:

```rust
pub struct ServerSaveHandler {
    database: Arc<Database>,
}

impl ServerSaveHandler {
    /// Process save request from client
    pub async fn handle_save_request(
        &self,
        player_id: Uuid,
        save_name: String,
        game_state: CompleteGameState,
    ) -> Result<Uuid, SaveError>;
    
    /// Process load request from client
    pub async fn handle_load_request(
        &self,
        player_id: Uuid,
        save_name: String,
    ) -> Result<CompleteGameState, SaveError>;
    
    /// List all saves for a player
    pub async fn handle_list_saves(
        &self,
        player_id: Uuid,
    ) -> Result<Vec<SaveFileMetadata>, SaveError>;
    
    /// Delete a save slot
    pub async fn handle_delete_save(
        &self,
        player_id: Uuid,
        save_name: String,
    ) -> Result<(), SaveError>;
    
    /// Auto-save system (runs every 30 seconds)
    pub async fn auto_save_active_sessions(
        &self,
    ) -> Result<(), SaveError>;
}
```

### 3. Client-Side Save Manager

Refactor `src/systems/pause_save.rs` to communicate with server:

```rust
pub struct ClientSaveManager {
    network: Res<NetworkResource>,
    pending_requests: HashMap<RequestId, PendingRequest>,
}

impl ClientSaveManager {
    /// Send save request to server
    pub fn request_save(
        &mut self,
        save_name: String,
        game_state: CompleteGameState,
    ) -> RequestId;
    
    /// Send load request to server
    pub fn request_load(
        &mut self,
        save_name: String,
    ) -> RequestId;
    
    /// Send list saves request to server
    pub fn request_list_saves(&mut self) -> RequestId;
    
    /// Process responses from server
    pub fn process_save_responses(
        &mut self,
        packets: Vec<GamePacket>,
    );
}
```

### 4. Database Schema Updates

The existing `save_games` table in PostgreSQL already supports the required structure. We will enhance it with additional indexes and constraints:

```sql
-- Add unique constraint for player_id + save_name
ALTER TABLE save_games 
ADD CONSTRAINT unique_player_save_name UNIQUE (player_id, save_name);

-- Add index for faster lookups
CREATE INDEX idx_save_games_player_updated 
ON save_games(player_id, updated_at DESC);

-- Add checksum column for data integrity
ALTER TABLE save_games 
ADD COLUMN checksum VARCHAR(64);
```

### 5. State Conversion Layer

Create `src/database/state_converter.rs` to convert between `CompleteGameState` and `GameData`:

```rust
pub struct StateConverter;

impl StateConverter {
    /// Convert CompleteGameState to database GameData format
    pub fn to_database_format(
        state: &CompleteGameState,
    ) -> GameData;
    
    /// Convert database GameData to CompleteGameState
    pub fn from_database_format(
        data: &GameData,
        metadata: &SaveGame,
    ) -> CompleteGameState;
    
    /// Calculate checksum for integrity verification
    pub fn calculate_checksum(
        state: &CompleteGameState,
    ) -> String;
}
```

## Data Models

### CompleteGameState (Enhanced)

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CompleteGameState {
    // Player state
    pub player_position: Vec3,
    pub player_velocity: Velocity,
    pub player_grounded: bool,
    pub player_crouching: bool,
    pub player_animation_state: String,
    
    // Camera state
    pub camera_position: Vec3,
    pub camera_target: Vec3,
    
    // Game metrics
    pub score: u32,
    pub distance_traveled: f32,
    pub jump_count: u32,
    pub play_time: f32,
    
    // Character and session info
    pub selected_character: CharacterType,
    pub player_count: PlayerCount,
    
    // Audio state
    pub music_position: f32,
    pub music_playing: bool,
    pub audio_volume: f32,
    
    // Entities (for future expansion)
    pub entities_snapshot: Vec<EntitySnapshot>,
    
    // Metadata
    pub save_timestamp: DateTime<Utc>,
    pub session_id: Option<Uuid>,  // NEW: Link to game session
}
```

### SaveFileMetadata (Enhanced)

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SaveFileMetadata {
    pub id: Uuid,  // NEW: Database ID
    pub name: String,
    pub score: u32,
    pub distance: f32,
    pub play_time: f32,
    pub character: CharacterType,  // NEW: Show character in save list
    pub save_timestamp: DateTime<Utc>,
    pub checksum: String,  // NEW: For integrity verification
}
```

### Request/Response Types

```rust
#[derive(Debug, Clone)]
pub struct PendingRequest {
    pub request_type: RequestType,
    pub timestamp: Instant,
    pub timeout: Duration,
}

#[derive(Debug, Clone)]
pub enum RequestType {
    Save { save_name: String },
    Load { save_name: String },
    ListSaves,
    Delete { save_name: String },
}

pub type RequestId = u64;

#[derive(Debug)]
pub enum SaveError {
    DatabaseError(String),
    NetworkError(String),
    ValidationError(String),
    NotFound(String),
    Timeout,
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*


### Property Reflection

After analyzing all acceptance criteria, several properties can be consolidated to eliminate redundancy:

**Consolidations:**
- Properties 3.1-3.4 all test round-trip consistency for different parts of GameState. These can be combined into a single comprehensive round-trip property.
- Properties 7.1, 7.2, 7.3, 7.4, 7.5 all test protocol compliance. These can be combined into properties that test request-response pairs.
- Properties 4.1, 4.2, 4.3 all test the refactoring goal of moving operations to the server. These can be combined into a single property about client-server responsibility.

**Retained Properties:**
- Auto-save timing (1.4) and manual save timing (1.1) are distinct behaviors
- Retry logic (1.5) is separate from basic save operations
- Concurrent operations (6.2) tests different behavior than single operations
- Migration properties (8.x) are distinct from normal save/load operations

### Correctness Properties

Property 1: Save operation completes within time limit
*For any* valid GameState and save_name, when a save operation is triggered, the Server should persist the data to PostgreSQL and respond within 5 seconds
**Validates: Requirements 1.1**

Property 2: Connection interruption preserves state
*For any* active game session, if the connection is interrupted at any point, the last known GameState should remain in PostgreSQL and be retrievable
**Validates: Requirements 1.2, 1.3**

Property 3: Auto-save occurs at regular intervals
*For any* active game session lasting longer than 30 seconds, the Server should perform automatic saves at 30-second intervals
**Validates: Requirements 1.4**

Property 4: Failed saves trigger retry logic
*For any* save operation that encounters a database error, the Server should retry up to 3 times before logging an error and responding with failure
**Validates: Requirements 1.5**

Property 5: Save slot creation generates unique identifiers
*For any* save_name provided by a player, creating a save slot should result in a database record with a unique UUID and a valid timestamp
**Validates: Requirements 2.1**

Property 6: Save slots are ordered by update time
*For any* player with multiple save slots, requesting the save list should return slots ordered by updated_at in descending order (most recent first)
**Validates: Requirements 2.2**

Property 7: Save and load round-trip preserves complete state
*For any* CompleteGameState (including player_position, player_velocity, player_animation_state, camera_position, camera_target, game statistics, selected_character, and audio settings), saving then loading should return an equivalent state
**Validates: Requirements 2.3, 3.1, 3.2, 3.3, 3.4**

Property 8: Loaded state is transmitted to client
*For any* successful load operation, the Server should transmit the complete GameState to the Client via a LoadGameResponse WebSocket packet
**Validates: Requirements 3.5**

Property 9: Save deletion removes data and confirms
*For any* existing save slot, deleting it should remove the record from PostgreSQL and send a DeleteSaveResponse with success=true to the Client
**Validates: Requirements 2.4**

Property 10: Save rename updates database
*For any* existing save slot and valid new_name, renaming should update the save_name field in PostgreSQL while preserving all other data
**Validates: Requirements 2.5**

Property 11: Client does not write local files
*For any* save or load operation after refactoring, the Client should not create, modify, or delete any local save files
**Validates: Requirements 4.1**

Property 12: Client communicates via WebSocket
*For any* save, load, list, or delete operation, the Client should send the corresponding request packet (SaveGameRequest, LoadGameRequest, ListSavesRequest, DeleteSaveRequest) via WebSocket
**Validates: Requirements 4.2, 7.1, 7.3**

Property 13: Server handles all database operations
*For any* save/load operation, all database queries should originate from the Server process, not the Client process
**Validates: Requirements 4.3**

Property 14: Game session is created on start
*For any* player starting a game, the Server should create a new GameSession record in PostgreSQL with the player_id, character_type, and start_time
**Validates: Requirements 5.1**

Property 15: Player actions are logged
*For any* player action (Move, Jump, Attack), the Server should create a PlayerAction record in PostgreSQL with the action_type, timestamp, and player position
**Validates: Requirements 5.2**

Property 16: Session finalization updates database
*For any* game session that ends, the Server should update the GameSession record with end_time and final statistics (distance_traveled, jump_count, play_time, score)
**Validates: Requirements 5.3**

Property 17: Session history is ordered chronologically
*For any* player with multiple game sessions, requesting session history should return GameSession records ordered by start_time in descending order
**Validates: Requirements 5.4**

Property 18: Session statistics update in real-time
*For any* active game session, the Server should update the GameSession statistics in PostgreSQL as gameplay progresses
**Validates: Requirements 5.5**

Property 19: Database operations use transactions
*For any* database write operation (save, delete, update), the Server should wrap the operation in a PostgreSQL transaction to ensure atomicity
**Validates: Requirements 6.1**

Property 20: Concurrent saves are serialized
*For any* two concurrent save operations for the same player, the Server should use database locks to ensure both operations complete without data loss or corruption
**Validates: Requirements 6.2**

Property 21: Save acknowledgment requires verification
*For any* save operation, the Server should only send a SaveGameResponse with success=true after verifying the data exists in PostgreSQL
**Validates: Requirements 6.3**

Property 22: Database reconnection queues operations
*For any* database connection failure, the Server should queue pending operations and execute them after successfully reconnecting
**Validates: Requirements 6.4**

Property 23: Checksums detect corruption
*For any* saved GameState, the Server should calculate and store a checksum, and when loading, verify the checksum matches to detect corruption
**Validates: Requirements 6.5**

Property 24: Server responses match protocol
*For any* save operation, the Server should respond with a SaveGameResponse packet containing success status and save metadata; for any list operation, the Server should respond with a ListSavesResponse containing all SaveFileMetadata
**Validates: Requirements 7.2, 7.4**

Property 25: Load protocol is request-response
*For any* load operation, the Client should send a LoadGameRequest with save_name, and the Server should respond with a LoadGameResponse containing the complete GameState
**Validates: Requirements 7.5**

Property 26: Migration transmits local data
*For any* local save file that is migrated, the Client should transmit the complete SaveFileData to the Server via WebSocket
**Validates: Requirements 8.2**

Property 27: Migrated data preserves timestamps
*For any* migrated save, the Server should store it in PostgreSQL with the original save_timestamp from the local file
**Validates: Requirements 8.3**

Property 28: Failed migration preserves local files
*For any* migration operation that fails, the Client should not delete or modify the local save files, and should allow the user to retry
**Validates: Requirements 8.5**

## Error Handling

### Error Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SaveError {
    /// Database connection or query failed
    DatabaseError { message: String, retry_count: u32 },
    
    /// Network communication failed
    NetworkError { message: String },
    
    /// Invalid input data
    ValidationError { field: String, message: String },
    
    /// Requested resource not found
    NotFound { resource: String, identifier: String },
    
    /// Operation timed out
    Timeout { operation: String, duration_ms: u64 },
    
    /// Checksum mismatch indicating corruption
    CorruptionDetected { save_name: String, expected: String, actual: String },
    
    /// Concurrent modification conflict
    ConcurrencyConflict { save_name: String },
}
```

### Error Handling Strategies

**Client-Side:**
- Display user-friendly error messages in UI
- Retry failed operations with exponential backoff
- Cache failed requests for retry when connection is restored
- Log errors to console for debugging

**Server-Side:**
- Retry database operations up to 3 times with exponential backoff
- Log all errors with context (player_id, operation, timestamp)
- Return structured error responses to clients
- Maintain operation queue during database outages
- Alert administrators for critical failures (database down, corruption detected)

**Database-Level:**
- Use transactions to ensure atomicity
- Implement row-level locking for concurrent operations
- Set connection pool timeouts to prevent resource exhaustion
- Enable query logging for debugging

## Testing Strategy

### Unit Testing

Unit tests will verify individual components in isolation:

**Server-Side Unit Tests:**
- `ServerSaveHandler::handle_save_request` with valid and invalid inputs
- `ServerSaveHandler::handle_load_request` with existing and non-existent saves
- `StateConverter::to_database_format` and `from_database_format` round-trip
- `StateConverter::calculate_checksum` produces consistent results
- Database operations with mocked PostgreSQL connections

**Client-Side Unit Tests:**
- `ClientSaveManager::request_save` sends correct WebSocket packet
- `ClientSaveManager::process_save_responses` handles success and error responses
- UI components display save slots correctly
- Migration detection identifies local save files

**Protocol Unit Tests:**
- Serialization and deserialization of all packet types
- Packet size limits are respected
- Invalid packets are rejected gracefully

### Property-Based Testing

Property-based tests will verify universal properties across many randomly generated inputs using the `proptest` crate (already in dev-dependencies):

**Property Test Configuration:**
- Minimum 100 iterations per property test
- Use custom generators for `CompleteGameState`, `SaveFileMetadata`, and packet types
- Test with edge cases: empty strings, maximum values, special characters

**Key Property Tests:**
- Property 7: Round-trip consistency (save then load returns equivalent state)
- Property 6: Save list ordering (multiple saves are always sorted correctly)
- Property 20: Concurrent save serialization (no data loss under concurrent operations)
- Property 23: Checksum integrity (corruption is always detected)
- Property 4: Retry logic (failures trigger exactly 3 retries)

**Property Test Tagging:**
Each property-based test will include a comment tag referencing the design document:
```rust
// **Feature: server-save-system, Property 7: Save and load round-trip preserves complete state**
// **Validates: Requirements 2.3, 3.1, 3.2, 3.3, 3.4**
#[test]
fn prop_save_load_roundtrip() { ... }
```

### Integration Testing

Integration tests will verify end-to-end workflows:

- Start server, connect client, perform save/load cycle
- Simulate network interruption during save operation
- Test auto-save triggers at 30-second intervals
- Verify database state after multiple concurrent operations
- Test migration of legacy local saves to server

### Performance Testing

- Measure save operation latency (target: < 5 seconds)
- Test with large GameState objects (many entities)
- Verify auto-save does not cause frame drops
- Test database connection pool under load (100+ concurrent clients)

## Implementation Notes

### Migration Strategy

1. **Phase 1: Add server-side save handlers** (no breaking changes)
   - Implement `ServerSaveHandler` and database operations
   - Add new protocol packets
   - Server can handle both old and new save formats

2. **Phase 2: Update client to use server saves**
   - Refactor `ClientSaveManager` to send WebSocket requests
   - Remove local file write operations
   - Add migration UI for legacy saves

3. **Phase 3: Deprecate local save system**
   - Mark `src/systems/save.rs` as deprecated
   - Remove local file operations from `pause_save.rs`
   - Update documentation

### Database Optimization

- Use connection pooling (already configured in sqlx)
- Create indexes on frequently queried columns (player_id, updated_at)
- Use prepared statements to prevent SQL injection
- Implement database query caching for read-heavy operations (list saves)

### Security Considerations

- Validate all client inputs (save_name length, character whitelist)
- Use parameterized queries to prevent SQL injection
- Implement rate limiting for save operations (max 1 save per 5 seconds per player)
- Authenticate players before allowing save/load operations
- Encrypt sensitive data in database (if needed in future)

### Scalability Considerations

- Database connection pool size should scale with concurrent players
- Consider sharding save_games table by player_id for very large player bases
- Implement caching layer (Redis) for frequently accessed saves
- Use database replication for read scalability
- Monitor database performance metrics (query time, connection pool usage)
