-- Your SQL goes here
CREATE TABLE
    TasksPerformed (
        id INTEGER PRIMARY KEY,
        task_id INTEGER,
        date TEXT, -- Store dates as 'YYYY-MM-DD'
        time_spent INTEGER,
        FOREIGN KEY (task_id) REFERENCES Tasks (id)
    );

-- Create an index on task_id for faster lookup
CREATE INDEX idx_task_id ON TasksPerformed (task_id);

-- Create an index on date for faster date-based queries
CREATE INDEX idx_date ON TasksPerformed (date);
