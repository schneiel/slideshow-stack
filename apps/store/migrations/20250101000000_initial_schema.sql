-- Initial database schema for display server
-- Created: 2025-01-01 00:00:00 UTC

-- Create migrations tracking table first
CREATE TABLE IF NOT EXISTS schema_migrations (
    version TEXT PRIMARY KEY,
    executed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Slideshows table
CREATE TABLE IF NOT EXISTS slideshows (
    id TEXT PRIMARY KEY,                    -- UUID v4
    name TEXT NOT NULL,                     -- Slideshow name
    description TEXT,                       -- Optional description
    interval_seconds INTEGER NOT NULL CHECK (interval_seconds >= 1 AND interval_seconds <= 30),
    loop_enabled BOOLEAN DEFAULT 1,         -- Whether to loop when reaching end
    shuffle BOOLEAN DEFAULT 0,              -- Whether to shuffle media order
    auto_start BOOLEAN DEFAULT 0,           -- Whether to auto-start when created
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Slideshow media junction table (many-to-many relationship)
CREATE TABLE IF NOT EXISTS slideshow_media (
    id TEXT PRIMARY KEY,                    -- UUID v4
    slideshow_id TEXT NOT NULL,             -- Foreign key to slideshows.id
    media_id TEXT NOT NULL,                 -- Media filename (references media directory)
    position INTEGER NOT NULL,              -- Display order position
    FOREIGN KEY (slideshow_id) REFERENCES slideshows(id) ON DELETE CASCADE,
    UNIQUE(slideshow_id, position)          -- Ensure unique positions per slideshow
);

-- Indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_slideshow_media_slideshow_id ON slideshow_media(slideshow_id);
CREATE INDEX IF NOT EXISTS idx_slideshow_media_media_id ON slideshow_media(media_id);
CREATE INDEX IF NOT EXISTS idx_slideshows_created_at ON slideshows(created_at);

-- Trigger to automatically update updated_at timestamp
CREATE TRIGGER IF NOT EXISTS update_slideshows_updated_at
    AFTER UPDATE ON slideshows
    FOR EACH ROW
BEGIN
    UPDATE slideshows SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;