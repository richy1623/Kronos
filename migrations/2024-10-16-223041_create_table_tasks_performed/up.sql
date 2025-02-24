CREATE TABLE
    task_performed (
        date TEXT NOT NULL, -- Store dates as 'YYYY-MM-DD'
        task_id INTEGER NOT NULL,
        time_spent INTEGER NOT NULL DEFAULT 0, -- Set default to 0
        -- is_synced_to_server BOOLEAN NOT NULL DEFAULT FALSE,
        PRIMARY KEY (date, task_id),
        FOREIGN KEY (task_id) REFERENCES Task (id) ON DELETE CASCADE
    );

-- Create an index on task_id for faster lookup
CREATE INDEX idx_task_performed_task_id ON task_performed (task_id);

-- Create an index on date for faster date-based queries
CREATE INDEX idx_task_performed_date ON task_performed (date);

-- -- Create an index on is_synced_to_server for faster queries for syncing
-- CREATE INDEX idx_task_performed_synced ON task (is_synced_to_server);
