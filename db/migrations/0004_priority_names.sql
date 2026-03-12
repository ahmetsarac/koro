-- 0004_priority_names.sql
-- Change priority values from p0/p1/p2/p3 to meaningful names

BEGIN;

-- Update existing data
UPDATE issues SET priority = 'critical' WHERE priority = 'p0';
UPDATE issues SET priority = 'high' WHERE priority = 'p1';
UPDATE issues SET priority = 'medium' WHERE priority = 'p2';
UPDATE issues SET priority = 'low' WHERE priority = 'p3';

-- Drop old constraint and add new one
ALTER TABLE issues DROP CONSTRAINT IF EXISTS issues_priority_check;
ALTER TABLE issues ADD CONSTRAINT issues_priority_check 
  CHECK (priority IN ('critical', 'high', 'medium', 'low'));

-- Update default
ALTER TABLE issues ALTER COLUMN priority SET DEFAULT 'medium';

COMMIT;
