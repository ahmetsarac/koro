-- Add 'none' to priority allowed values and set as default for new issues

BEGIN;

ALTER TABLE issues DROP CONSTRAINT IF EXISTS issues_priority_check;
ALTER TABLE issues ADD CONSTRAINT issues_priority_check
  CHECK (priority IN ('none', 'critical', 'high', 'medium', 'low'));

ALTER TABLE issues ALTER COLUMN priority SET DEFAULT 'none';

COMMIT;
