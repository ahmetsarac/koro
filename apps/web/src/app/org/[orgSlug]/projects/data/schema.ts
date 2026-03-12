import { z } from "zod"

export const projectSchema = z.object({
  id: z.string().uuid(),
  project_key: z.string(),
  name: z.string(),
  description: z.string().nullable(),
  org_id: z.string().uuid(),
  org_name: z.string(),
  org_slug: z.string(),
  issue_count: z.number().int().nonnegative(),
  member_count: z.number().int().nonnegative(),
  my_role: z.string(),
  created_at: z.string(),
})

export type Project = z.infer<typeof projectSchema>

export const projectListResponseSchema = z.object({
  items: z.array(projectSchema),
  total: z.number().int().nonnegative(),
  limit: z.number().int().positive(),
  offset: z.number().int().nonnegative(),
  has_more: z.boolean(),
})

export type ProjectListResponse = z.infer<typeof projectListResponseSchema>
