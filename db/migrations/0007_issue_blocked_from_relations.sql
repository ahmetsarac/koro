-- Blocked is derived from issue_relations (type `blocks` → target), not a stored flag.
ALTER TABLE issues DROP COLUMN IF EXISTS is_blocked;
