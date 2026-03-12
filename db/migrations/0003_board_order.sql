-- Add board_order column for kanban board ordering within status columns
ALTER TABLE issues ADD COLUMN board_order INTEGER NOT NULL DEFAULT 0;

-- Create index for efficient board queries
CREATE INDEX idx_issues_board_order ON issues(project_id, status, board_order);
