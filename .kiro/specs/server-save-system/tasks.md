# Implementation Plan

- [ ] 1. Extend network protocol with save/load packets
  - Add new packet types to `src/protocol.rs` for save/load operations
  - Implement serialization/deserialization for `CompleteGameState` in packets
  - Add request/response packet types: `SaveGameRequest`, `SaveGameResponse`, `LoadGameRequest`, `LoadGameResponse`, `ListSavesRequest`, `ListSavesResponse`, `DeleteSaveRequest`, `DeleteSaveResponse`, `RenameSaveRequest`, `RenameSaveResponse`
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_

- [ ] 1.1 Write property test for protocol serialization
  - **Property 24: Server responses match protocol**
  - **Validates: Requirements 7.2, 7.4**

- [ ] 2. Create state conversion layer
  - Create `src/database/state_converter.rs`
  - Implement `StateConverter::to_database_format` to convert `CompleteGameState` to `GameData`
  - Implement `StateConverter::from_database_format` to convert `GameData` to `CompleteGameState`
  - Implement `StateConverter::calculate_checksum` for integrity verification
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 6.5_

- [ ] 2.1 Write property test for state conversion round-trip
  - **Property 7: Save and load round-trip preserves complete state**
  - **Validates: Requirements 2.3, 3.1, 3.2, 3.3, 3.4**

- [ ] 2.2 Write property test for checksum consistency
  - **Property 23: Checksums detect corruption**
  - **Validates: Requirements 6.5**

- [ ] 3. Update database schema
  - Add unique constraint on `save_games` table for `(player_id, save_name)`
  - Add index on `save_games(player_id, updated_at DESC)` for faster lookups
  - Add `checksum` column to `save_games` table
  - Create database migration script
  - _Requirements: 2.1, 6.5_

- [ ] 4. Implement server-side save handler
  - Create `src/systems/server_save_handler.rs`
  - Implement `ServerSaveHandler` struct with database connection
  - Implement `handle_save_request` method with transaction support
  - Implement `handle_load_request` method with checksum verification
  - Implement `handle_list_saves` method with ordering
  - Implement `handle_delete_save` method
  - Implement `handle_rename_save` method
  - _Requirements: 1.1, 2.1, 2.2, 2.3, 2.4, 2.5, 3.5, 6.1, 6.3, 6.5_

- [ ] 4.1 Write property test for save operation timing
  - **Property 1: Save operation completes within time limit**
  - **Validates: Requirements 1.1**

- [ ] 4.2 Write property test for save slot creation
  - **Property 5: Save slot creation generates unique identifiers**
  - **Validates: Requirements 2.1**

- [ ] 4.3 Write property test for save list ordering
  - **Property 6: Save slots are ordered by update time**
  - **Validates: Requirements 2.2**

- [ ] 4.4 Write property test for save deletion
  - **Property 9: Save deletion removes data and confirms**
  - **Validates: Requirements 2.4**

- [ ] 4.5 Write property test for save rename
  - **Property 10: Save rename updates database**
  - **Validates: Requirements 2.5**

- [ ] 4.6 Write property test for transaction usage
  - **Property 19: Database operations use transactions**
  - **Validates: Requirements 6.1**

- [ ] 4.7 Write property test for save verification
  - **Property 21: Save acknowledgment requires verification**
  - **Validates: Requirements 6.3**

- [ ] 5. Implement auto-save system on server
  - Add auto-save timer to server main loop
  - Implement `auto_save_active_sessions` method in `ServerSaveHandler`
  - Track active game sessions for auto-save
  - _Requirements: 1.4_

- [ ] 5.1 Write property test for auto-save intervals
  - **Property 3: Auto-save occurs at regular intervals**
  - **Validates: Requirements 1.4**

- [ ] 6. Implement retry logic for failed saves
  - Add retry counter to save operations
  - Implement exponential backoff for retries
  - Log errors after 3 failed attempts
  - _Requirements: 1.5_

- [ ] 6.1 Write property test for retry logic
  - **Property 4: Failed saves trigger retry logic**
  - **Validates: Requirements 1.5**

- [ ] 7. Implement connection interruption handling
  - Add connection state tracking to server
  - Preserve last known GameState on disconnection
  - Implement reconnection logic with state restoration
  - _Requirements: 1.2, 1.3_

- [ ] 7.1 Write property test for state preservation on disconnect
  - **Property 2: Connection interruption preserves state**
  - **Validates: Requirements 1.2, 1.3**

- [ ] 8. Integrate save handler into server network loop
  - Update `src/bin/server.rs` to handle save/load packets
  - Route save/load requests to `ServerSaveHandler`
  - Send responses back to clients via WebSocket
  - _Requirements: 3.5, 7.2, 7.4, 7.5_

- [ ] 8.1 Write property test for state transmission
  - **Property 8: Loaded state is transmitted to client**
  - **Validates: Requirements 3.5**

- [ ] 8.2 Write property test for load protocol
  - **Property 25: Load protocol is request-response**
  - **Validates: Requirements 7.5**

- [ ] 9. Checkpoint - Ensure server-side tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 10. Refactor client save manager
  - Update `src/systems/pause_save.rs` to use WebSocket communication
  - Remove local file write operations from `capture_game_state`
  - Implement `ClientSaveManager::request_save` to send WebSocket packets
  - Implement `ClientSaveManager::request_load` to send WebSocket packets
  - Implement `ClientSaveManager::request_list_saves` to send WebSocket packets
  - Implement `ClientSaveManager::process_save_responses` to handle server responses
  - _Requirements: 4.1, 4.2, 7.1, 7.3_

- [ ] 10.1 Write property test for client file operations
  - **Property 11: Client does not write local files**
  - **Validates: Requirements 4.1**

- [ ] 10.2 Write property test for client WebSocket communication
  - **Property 12: Client communicates via WebSocket**
  - **Validates: Requirements 4.2, 7.1, 7.3**

- [ ] 11. Update client UI for server-based saves
  - Update pause menu to request saves from server
  - Update load table to display server save slots
  - Add loading indicators for network operations
  - Handle error responses from server in UI
  - _Requirements: 2.2, 3.5_

- [ ] 12. Implement game session tracking
  - Update server to create GameSession on player join
  - Log player actions to database during gameplay
  - Update session statistics in real-time
  - Finalize session on player disconnect
  - _Requirements: 5.1, 5.2, 5.3, 5.5_

- [ ] 12.1 Write property test for session creation
  - **Property 14: Game session is created on start**
  - **Validates: Requirements 5.1**

- [ ] 12.2 Write property test for action logging
  - **Property 15: Player actions are logged**
  - **Validates: Requirements 5.2**

- [ ] 12.3 Write property test for session finalization
  - **Property 16: Session finalization updates database**
  - **Validates: Requirements 5.3**

- [ ] 12.4 Write property test for real-time statistics
  - **Property 18: Session statistics update in real-time**
  - **Validates: Requirements 5.5**

- [ ] 13. Implement session history retrieval
  - Add endpoint to retrieve player session history
  - Order sessions by start_time descending
  - Return session list to client
  - _Requirements: 5.4_

- [ ] 13.1 Write property test for session history ordering
  - **Property 17: Session history is ordered chronologically**
  - **Validates: Requirements 5.4**

- [ ] 14. Implement concurrency control
  - Add database row-level locking for save operations
  - Handle concurrent save conflicts gracefully
  - Test with multiple simultaneous save requests
  - _Requirements: 6.2_

- [ ] 14.1 Write property test for concurrent saves
  - **Property 20: Concurrent saves are serialized**
  - **Validates: Requirements 6.2**

- [ ] 15. Implement database reconnection logic
  - Add connection health monitoring
  - Queue operations during database outage
  - Execute queued operations after reconnection
  - _Requirements: 6.4_

- [ ] 15.1 Write property test for operation queueing
  - **Property 22: Database reconnection queues operations**
  - **Validates: Requirements 6.4**

- [ ] 16. Checkpoint - Ensure client-server integration tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 17. Implement local save migration
  - Add migration detection on client startup
  - Display migration prompt UI when local saves are found
  - Implement migration data transmission to server
  - Preserve original timestamps during migration
  - Archive local files after successful migration
  - _Requirements: 8.1, 8.2, 8.3, 8.4_

- [ ] 17.1 Write property test for migration data transmission
  - **Property 26: Migration transmits local data**
  - **Validates: Requirements 8.2**

- [ ] 17.2 Write property test for timestamp preservation
  - **Property 27: Migrated data preserves timestamps**
  - **Validates: Requirements 8.3**

- [ ] 18. Implement migration error handling
  - Preserve local files on migration failure
  - Allow retry of failed migrations
  - Display error messages to user
  - _Requirements: 8.5_

- [ ] 18.1 Write property test for migration failure handling
  - **Property 28: Failed migration preserves local files**
  - **Validates: Requirements 8.5**

- [ ] 19. Remove deprecated local save system
  - Mark `src/systems/save.rs` functions as deprecated
  - Remove local file operations from `pause_save.rs`
  - Update documentation to reflect server-based saves
  - Remove `SaveManager` resource if no longer needed
  - _Requirements: 4.4, 4.5_

- [ ] 20. Add comprehensive error handling
  - Implement `SaveError` enum with all error types
  - Add error logging on server
  - Display user-friendly error messages on client
  - Implement exponential backoff for retries
  - _Requirements: 1.5, 6.3, 6.4_

- [ ] 21. Implement database operation verification
  - Verify all database operations originate from server
  - Add logging for database queries
  - Ensure client never directly accesses database
  - _Requirements: 4.3_

- [ ] 21.1 Write property test for server database responsibility
  - **Property 13: Server handles all database operations**
  - **Validates: Requirements 4.3**

- [ ] 22. Final checkpoint - Run full integration test suite
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 23. Performance testing and optimization
  - Measure save operation latency
  - Test with large GameState objects
  - Verify auto-save doesn't cause frame drops
  - Test database connection pool under load
  - Optimize slow queries if needed

- [ ] 24. Documentation and cleanup
  - Update README with server setup instructions
  - Document database schema changes
  - Add code comments for complex logic
  - Create migration guide for existing players
  - Remove unused code and dependencies
