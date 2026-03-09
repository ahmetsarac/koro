BEGIN;

CREATE TABLE demo_tasks (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  status TEXT NOT NULL
    CHECK (status IN ('backlog', 'todo', 'in progress', 'done', 'canceled')),
  label TEXT NOT NULL
    CHECK (label IN ('bug', 'feature', 'documentation')),
  priority TEXT NOT NULL
    CHECK (priority IN ('low', 'medium', 'high')),
  sort_order INTEGER NOT NULL UNIQUE CHECK (sort_order >= 0),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_demo_tasks_sort_order ON demo_tasks(sort_order);
CREATE INDEX idx_demo_tasks_status ON demo_tasks(status);
CREATE INDEX idx_demo_tasks_priority ON demo_tasks(priority);
CREATE INDEX idx_demo_tasks_created_at ON demo_tasks(created_at DESC);

CREATE TRIGGER trg_demo_tasks_updated_at
BEFORE UPDATE ON demo_tasks
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

COMMIT;
