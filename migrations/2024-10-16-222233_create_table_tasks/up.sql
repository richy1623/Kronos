-- Your SQL goes here
CREATE TABLE
    task (
        id INTEGER PRIMARY KEY NOT NULL,
        name TEXT UNIQUE NOT NULL
    );

-- Create an index on name for faster lookup
CREATE INDEX idx_task_name ON task (name);
