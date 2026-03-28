-- Per-user last time they viewed project-scoped UI (sidebar ordering).

BEGIN;

ALTER TABLE project_members
  ADD COLUMN IF NOT EXISTS viewed_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS idx_project_members_user_viewed
  ON project_members (user_id, viewed_at DESC);

COMMIT;
