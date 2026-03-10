import {
  demoTaskListResponseSchema,
  type DemoTaskListResponse,
} from "@/app/dashboard/my-issues/data/schema"

/** Backend sort_by: sort_order | created_at | id | title | status | label | priority */
export type DemoTaskSortBy =
  | "sort_order"
  | "created_at"
  | "id"
  | "title"
  | "status"
  | "label"
  | "priority"

export type DemoTaskSortDir = "asc" | "desc"

export type FetchDemoTasksParams = {
  limit?: number
  offset?: number
  q?: string
  /** Single value or multiple (backend accepts comma-separated) */
  status?: string | string[]
  label?: string | string[]
  priority?: string | string[]
  sort_by?: DemoTaskSortBy
  sort_dir?: DemoTaskSortDir
}

function buildSearchParams(params: FetchDemoTasksParams) {
  const searchParams = new URLSearchParams()

  const arrayKeys = ["status", "label", "priority"] as const
  for (const key of arrayKeys) {
    const value = params[key]
    if (value === undefined) continue
    const arr = Array.isArray(value) ? value : [value]
    const joined = arr.filter(Boolean).join(",")
    if (joined) searchParams.set(key, joined)
  }

  const { status: _s, label: _l, priority: _p, ...rest } = params
  Object.entries(rest).forEach(([key, value]) => {
    if (value !== undefined && value !== "") {
      searchParams.set(key, String(value))
    }
  })

  return searchParams
}

export async function fetchDemoTasks(
  params: FetchDemoTasksParams = {}
): Promise<DemoTaskListResponse> {
  const searchParams = buildSearchParams(params)
  const query = searchParams.toString()
  const response = await fetch(`/api/demo/tasks${query ? `?${query}` : ""}`, {
    cache: "no-store",
  })

  if (!response.ok) {
    throw new Error("Failed to fetch demo tasks.")
  }

  const data = await response.json()
  return demoTaskListResponseSchema.parse(data)
}
