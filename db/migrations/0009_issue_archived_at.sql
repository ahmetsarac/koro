-- Archived issues: NULL = active, set = archived (hidden from board/active lists)

BEGIN;

ALTER TABLE issues
  ADD COLUMN archived_at TIMESTAMPTZ NULL;

CREATE INDEX idx_issues_project_archived
  ON issues (project_id)
  WHERE archived_at IS NOT NULL;

COMMIT;
