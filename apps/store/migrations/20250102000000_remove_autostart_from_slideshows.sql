-- Migration: Remove autostart from slideshows
-- Created: 2026-01-16 00:00:00 UTC
-- Description: Autostart is no longer a slideshow property, but a separate system configuration
-- Note: This migration may fail if already applied, which is acceptable

-- Remove autostart column from slideshows table
ALTER TABLE slideshows DROP COLUMN auto_start;
