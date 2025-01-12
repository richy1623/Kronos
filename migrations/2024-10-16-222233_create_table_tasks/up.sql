CREATE TABLE
    task (
        id INTEGER PRIMARY KEY NOT NULL,
        name TEXT UNIQUE NOT NULL,
        last_used INTEGER NOT NULL DEFAULT (STRFTIME ('%s', 'now')) -- Unix timestamp
    );

-- Create an index on name for faster lookup
CREATE INDEX idx_task_name ON task (name);

-- Optional: Create an index on last_used for faster last-used queries
CREATE INDEX idx_task_last_used ON task (last_used);
