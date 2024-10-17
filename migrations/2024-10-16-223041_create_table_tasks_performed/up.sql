-- Your SQL goes here
CREATE TABLE
    task_performed (
        date TEXT NOT NULL, -- Store dates as 'YYYY-MM-DD'
        task_id INTEGER NOT NULL,
        time_spent INTEGER NOT NULL,
        PRIMARY KEY (date, task_id),
        FOREIGN KEY (task_id) REFERENCES Tasks (id)
    );

-- Create an index on task_id for faster lookup
CREATE INDEX idx_task_performed_task_id ON task_performed (task_id);

-- Create an index on date for faster date-based queries
CREATE INDEX idx_task_performed_date ON task_performed (date);
