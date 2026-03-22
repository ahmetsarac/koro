import { z } from "zod"

export const statusFacetEntrySchema = z.object({
  workflow_status_id: z.string().uuid(),
  name: z.string(),
  slug: z.string(),
  category: z.string(),
  count: z.number().int().nonnegative(),
})

export type StatusFacetEntry = z.infer<typeof statusFacetEntrySchema>

export const issueSchema = z.object({
  id: z.string().uuid(),
  project_id: z.string().uuid(),
  display_key: z.string(),
  title: z.string(),
  status: z.string(),
  workflow_status_id: z.string().uuid(),
  status_name: z.string(),
  status_category: z.string(),
  is_blocked: z.boolean(),
  priority: z.string(),
})

export type Issue = z.infer<typeof issueSchema>

export const issueFacetsSchema = z.object({
  status: z.array(statusFacetEntrySchema),
  priority: z.record(z.string(), z.number().int().nonnegative()),
})

export type IssueFacets = z.infer<typeof issueFacetsSchema>

export const issueListResponseSchema = z.object({
  items: z.array(issueSchema),
  total: z.number().int().nonnegative(),
  limit: z.number().int().positive(),
  offset: z.number().int().nonnegative(),
  has_more: z.boolean(),
  next_cursor: z.string().nullable().optional(),
  sort_by: z.string(),
  sort_dir: z.string(),
  facets: issueFacetsSchema,
})

export type IssueListResponse = z.infer<typeof issueListResponseSchema>
