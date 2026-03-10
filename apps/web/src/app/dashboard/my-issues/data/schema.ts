import { z } from "zod"

// We're keeping a simple non-relational schema here.
// IRL, you will have a schema for your data models.
export const taskSchema = z.object({
  id: z.string(),
  title: z.string(),
  status: z.string(),
  label: z.string(),
  priority: z.string(),
})

export type Task = z.infer<typeof taskSchema>

/** Server-side facet counts: value -> count (e.g. { status: { backlog: 1200 } }) */
export const demoTaskFacetsSchema = z.object({
  status: z.record(z.string(), z.number().int().nonnegative()),
  label: z.record(z.string(), z.number().int().nonnegative()),
  priority: z.record(z.string(), z.number().int().nonnegative()),
})

export type DemoTaskFacets = z.infer<typeof demoTaskFacetsSchema>

export const demoTaskListResponseSchema = z.object({
  items: z.array(taskSchema),
  total: z.number().int().nonnegative(),
  limit: z.number().int().positive(),
  offset: z.number().int().nonnegative(),
  has_more: z.boolean(),
  facets: demoTaskFacetsSchema,
})

export type DemoTaskListResponse = z.infer<typeof demoTaskListResponseSchema>
