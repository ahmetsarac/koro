-- organizations.id INSERT'te verilmediğinde gen_random_uuid() üretilsin (users vb. ile aynı davranış).
ALTER TABLE organizations
  ALTER COLUMN id SET DEFAULT gen_random_uuid();
