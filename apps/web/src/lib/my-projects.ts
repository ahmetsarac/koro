import {
  projectListResponseSchema,
  type ProjectListResponse,
} from "@/app/[orgSlug]/projects/data/schema"

export type FetchMyProjectsParams = {
  orgSlug: string
  limit?: number
  offset?: number
  q?: string
}

function buildSearchParams(params: FetchMyProjectsParams) {
  const searchParams = new URLSearchParams()
  searchParams.set("org_slug", params.orgSlug)

  const { orgSlug: _orgSlug, ...rest } = params
  Object.entries(rest).forEach(([key, value]) => {
    if (value !== undefined && value !== "") {
      searchParams.set(key, String(value))
    }
  })

  return searchParams
}

export async function fetchMyProjects(
  params: FetchMyProjectsParams
): Promise<ProjectListResponse> {
  const searchParams = buildSearchParams(params)
  const query = searchParams.toString()

  const response = await fetch(`/api/my-projects${query ? `?${query}` : ""}`, {
    cache: "no-store",
    credentials: "same-origin",
  })

  if (!response.ok) {
    throw new Error("Failed to fetch my projects.")
  }

  const data = await response.json()
  return projectListResponseSchema.parse(data)
}
