import {
  issueListResponseSchema,
  type IssueListResponse,
} from "@/app/dashboard/my-issues/data/schema"

export type IssueSortBy =
  | "created_at"
  | "updated_at"
  | "key_seq"
  | "title"
  | "status"
  | "priority"

export type IssueSortDir = "asc" | "desc"

export type FetchMyIssuesParams = {
  limit?: number
  offset?: number
  cursor?: string | null
  q?: string
  status?: string | string[]
  priority?: string | string[]
  sort_by?: IssueSortBy
  sort_dir?: IssueSortDir
}

function buildSearchParams(params: FetchMyIssuesParams) {
  const searchParams = new URLSearchParams()

  const arrayKeys = ["status", "priority"] as const
  for (const key of arrayKeys) {
    const value = params[key]
    if (value === undefined) continue
    const arr = Array.isArray(value) ? value : [value]
    const joined = arr.filter(Boolean).join(",")
    if (joined) searchParams.set(key, joined)
  }

  const { status: _s, priority: _p, cursor, offset, ...rest } = params
  if (cursor) {
    searchParams.set("cursor", cursor)
  } else if (offset !== undefined) {
    searchParams.set("offset", String(offset))
  }
  Object.entries(rest).forEach(([key, value]) => {
    if (value !== undefined && value !== "") {
      searchParams.set(key, String(value))
    }
  })

  return searchParams
}

export async function fetchMyIssues(
  params: FetchMyIssuesParams = {}
): Promise<IssueListResponse> {
  const searchParams = buildSearchParams(params)
  const query = searchParams.toString()

  const response = await fetch(`/api/my-issues${query ? `?${query}` : ""}`, {
    cache: "no-store",
    credentials: "same-origin",
  })

  if (!response.ok) {
    throw new Error("Failed to fetch my issues.")
  }

  const data = await response.json()
  return issueListResponseSchema.parse(data)
}
