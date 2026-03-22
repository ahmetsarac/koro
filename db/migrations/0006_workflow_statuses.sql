-- Workflow statuses per project (Linear-style category + custom labels)
CREATE TABLE project_workflow_statuses (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  category TEXT NOT NULL CHECK (category IN ('backlog', 'unstarted', 'started', 'completed', 'canceled')),
  name TEXT NOT NULL,
  slug TEXT NOT NULL,
  position INTEGER NOT NULL,
  is_default BOOLEAN NOT NULL DEFAULT false,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE(project_id, slug)
);

CREATE INDEX idx_project_workflow_statuses_project ON project_workflow_statuses(project_id);
CREATE INDEX idx_project_workflow_statuses_project_position ON project_workflow_statuses(project_id, position);

CREATE UNIQUE INDEX idx_project_workflow_one_default
  ON project_workflow_statuses (project_id)
  WHERE is_default;

-- Seed default workflow per existing project
INSERT INTO project_workflow_statuses (project_id, category, name, slug, position, is_default)
SELECT p.id, v.category, v.name, v.slug, v.position, v.is_default
FROM projects p
CROSS JOIN (
  VALUES
    ('backlog', 'Backlog', 'backlog', 0, true),
    ('unstarted', 'Todo', 'todo', 1, false),
    ('started', 'In Progress', 'in_progress', 2, false),
    ('completed', 'Done', 'done', 3, false),
    ('canceled', 'Canceled', 'canceled', 4, false)
) AS v(category, name, slug, position, is_default);

ALTER TABLE issues ADD COLUMN workflow_status_id UUID REFERENCES project_workflow_statuses(id);
ALTER TABLE issues ADD COLUMN is_blocked BOOLEAN NOT NULL DEFAULT false;

UPDATE issues i
SET workflow_status_id = pws.id
FROM project_workflow_statuses pws
WHERE pws.project_id = i.project_id
  AND pws.slug = CASE i.status
    WHEN 'backlog' THEN 'backlog'
    WHEN 'todo' THEN 'todo'
    WHEN 'in_progress' THEN 'in_progress'
    WHEN 'blocked' THEN 'in_progress'
    WHEN 'done' THEN 'done'
    ELSE 'backlog'
  END;

UPDATE issues SET is_blocked = true WHERE status = 'blocked';

ALTER TABLE issues ALTER COLUMN workflow_status_id SET NOT NULL;

DROP INDEX IF EXISTS idx_issues_project_status;
DROP INDEX IF EXISTS idx_issues_board_order;

ALTER TABLE issues DROP COLUMN status;

CREATE INDEX idx_issues_project_workflow ON issues(project_id, workflow_status_id);
CREATE INDEX idx_issues_board_order ON issues(project_id, workflow_status_id, board_order);
