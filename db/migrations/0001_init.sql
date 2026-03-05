-- 0001_init.sql
-- Koro core schema (secure bootstrap + SaaS-ready multi-tenant)

BEGIN;

-- UUID generator
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ===== Enums as TEXT + CHECK (MVP-friendly) =====
-- (Postgres ENUM da olur ama migration degisiklikleri daha zor olur. TEXT + CHECK daha esnek.)

-- ===== users =====
CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  email TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,

  -- Auth
  password_hash TEXT, -- NULL olabilir (ileride SSO/magic link gibi modlar icin)
  is_active BOOLEAN NOT NULL DEFAULT TRUE,

  -- Platform-level role (for secure bootstrap + ops)
  -- Only a very small set of users should be platform_admin
  platform_role TEXT NOT NULL DEFAULT 'user'
    CHECK (platform_role IN ('user', 'platform_admin')),

  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
 
-- ===== organizations =====
CREATE TABLE organizations (
  id UUID PRIMARY KEY,
  name TEXT NOT NULL,
  slug TEXT NOT NULL UNIQUE,
  
  created_by UUID REFERENCES users(id),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ===== org_members =====
CREATE TABLE org_members (
  org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  org_role TEXT NOT NULL
    CHECK (org_role IN ('org_admin', 'org_member')),
  
  joined_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (org_id, user_id)
);

CREATE INDEX idx_org_members_user ON org_members(user_id);

-- ===== projects =====
CREATE TABLE projects (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,

  project_key TEXT NOT NULL, -- "APP", "PAY"
  name TEXT NOT NULL,
  description TEXT,

  next_issue_seq INTEGER NOT NULL DEFAULT 1 CHECK (next_issue_seq >= 1),

  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

  UNIQUE(org_id, project_key)
);

CREATE INDEX idx_projects_org  ON projects(org_id);

-- ===== project_members =====
CREATE TABLE project_members (
  project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  project_role TEXT NOT NULL
    CHECK (project_role IN ('project_manager', 'business_analyst', 'developer', 'viewer')),

  joined_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (project_id, user_id)
);

CREATE INDEX idx_project_members_user ON project_members(user_id);

-- ===== issues =====
CREATE TABLE issues (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
  project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,

  -- Human key parts: project_key + key_seq => APP-123
  key_seq INTEGER NOT NULL CHECK (key_seq >= 1),

  title TEXT NOT NULL,
  description TEXT,

  status TEXT NOT NULL DEFAULT 'backlog'
    CHECK (status IN ('backlog', 'todo', 'in_progress', 'blocked', 'done')),
  priority TEXT NOT NULL DEFAULT 'p2'
    CHECK (priority IN ('p0', 'p1', 'p2', 'p3')),

  reporter_id UUID REFERENCES users(id) ON DELETE SET NULL,
  assignee_id UUID REFERENCES users(id) ON DELETE SET NULL,

  -- Subtasks
  parent_issue_id UUID REFERENCES users(id) ON DELETE SET NULL,

  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

  UNIQUE(project_id, key_seq)
);

CREATE INDEX idx_issues_project_status ON issues(project_id, status);
CREATE INDEX idx_issues_project_updated ON issues(project_id, updated_at DESC);
CREATE INDEX idx_issues_org ON issues(org_id);
CREATE INDEX idx_issues_parent ON issues(parent_issue_id);

-- ===== comments =====
CREATE TABLE comments (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
  project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  issue_id UUID NOT NULL REFERENCES issues(id) ON DELETE CASCADE,

  author_id UUID REFERENCES users(id) ON DELETE SET NULL,
  body TEXT NOT NULL,

  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_comments_issue_created ON comments(issue_id, created_at);

-- ===== issue_relations =====
-- Store only one direction as source->target with type.
-- UI/API can expose both "blocking" and "blocked_by" views.
CREATE TABLE issue_relations (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,

  source_issue_id UUID NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
  target_issue_id UUID NOT NULL REFERENCES issues(id) ON DELETE CASCADE,

  relation_type TEXT NOT NULL,
    CHECK (relation_type IN ('blocks', 'relates_to', 'duplicate')),
  
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

  CONSTRAINT chk_relation_not_self CHECK (source_issue_id <> target_issue_id),
  CONSTRAINT uq_relation UNIQUE (source_issue_id, target_issue_id, relation_type)
);

CREATE INDEX idx_relations_source ON issue_relations(source_issue_id);
CREATE INDEX idx_relations_target ON issue_relations(target_issue_id);

-- ===== weekly_updates =====
CREATE TABLE weekly_updates (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
  project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  author_id UUID REFERENCES users(id) ON DELETE SET NULL,

  week_start DATE NOT NULL, -- monday date prefferred

  last_week TEXT,
  this_week TEXT,
  blockers TEXT,

  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

  -- One update per author per project per week
  UNIQUE (project_id, author_id, week_start)
);

CREATE INDEX idx_weekly_project_week ON weekly_updates(project_id, week_start DESC);

-- ===== invites (org-scoped) =====
-- Secure bootstrap + controlled access.
-- Token should be generated server-side; store only token_hash.
CREATE TABLE user_invites (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,

  email TEXT NOT NULL,
  invited_role TEXT NOT NULL
    CHECK (invited_role IN ('org_admin', 'org_member')),

  token_hash TEXT NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  used_at TIMESTAMPTZ,

  invited_by UUID REFERENCES users(id) ON DELETE SET NULL,

  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

  CONSTRAINT uq_invite_active UNIQUE (org_id, email, token_hash)
);

CREATE INDEX idx_invites_org_email ON user_invites(org_id, email);
CREATE INDEX idx_invites_expires ON user_invites(expires_at);

-- ===== issue_events =====
CREATE TABLE issue_events (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
  issue_id UUID NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
  actor_id UUID REFERENCES users(id) ON DELETE SET NULL,
  event_type TEXT NOT NULL,
  payload JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_issue_events_issue_created
ON issue_events(issue_id, created_at);

-- ===== updated_at trigger =====
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = now();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_users_updated_at
BEFORE UPDATE ON users
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_orgs_updated_at
BEFORE UPDATE ON organizations
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_projects_updated_at
BEFORE UPDATE ON projects
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_issues_updated_at
BEFORE UPDATE ON issues
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_weekly_updates_updated_at
BEFORE UPDATE ON weekly_updates
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

COMMIT;