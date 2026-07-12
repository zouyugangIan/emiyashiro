CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS players (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS game_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    character_type VARCHAR(20) NOT NULL,
    start_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    end_time TIMESTAMPTZ,
    distance_traveled REAL NOT NULL DEFAULT 0.0,
    jump_count INTEGER NOT NULL DEFAULT 0,
    play_time REAL NOT NULL DEFAULT 0.0,
    score INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS player_actions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES game_sessions(id) ON DELETE CASCADE,
    action_type VARCHAR(20) NOT NULL,
    action_data JSONB,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    player_position_x REAL,
    player_position_y REAL
);

CREATE TABLE IF NOT EXISTS save_games (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    save_name VARCHAR(100) NOT NULL,
    game_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT save_games_player_save_unique UNIQUE (player_id, save_name)
);

-- 为旧版运行时建表产生的数据库补齐 upsert 所需的唯一约束。
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'save_games_player_save_unique'
          AND conrelid = 'save_games'::regclass
    ) THEN
        ALTER TABLE save_games
            ADD CONSTRAINT save_games_player_save_unique UNIQUE (player_id, save_name);
    END IF;
END
$$;

CREATE INDEX IF NOT EXISTS game_sessions_player_id_idx ON game_sessions(player_id);
CREATE INDEX IF NOT EXISTS player_actions_session_id_idx ON player_actions(session_id);
CREATE INDEX IF NOT EXISTS save_games_player_id_updated_at_idx
    ON save_games(player_id, updated_at DESC);
