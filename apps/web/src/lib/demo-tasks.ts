import {
  demoTaskListResponseSchema,
  type DemoTaskListResponse,
} from "@/app/dashboard/my-issues/data/schema"

type FetchDemoTasksParams = {
  limit?: number
  offset?: number
  q?: string
  status?: string
  priority?: string
}

function buildSearchParams(params: FetchDemoTasksParams) {
  const searchParams = new URLSearchParams()

  Object.entries(params).forEach(([key, value]) => {
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
