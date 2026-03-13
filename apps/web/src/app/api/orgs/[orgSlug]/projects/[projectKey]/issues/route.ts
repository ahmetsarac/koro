import { NextRequest, NextResponse } from "next/server"

import { getApiBaseUrl } from "@/lib/api/backend"
import { ACCESS_TOKEN_COOKIE_NAME } from "@/lib/auth/constants"

export async function GET(
  request: NextRequest,
  { params }: { params: Promise<{ orgSlug: string; projectKey: string }> }
) {
  try {
    const { orgSlug, projectKey } = await params
    const { searchParams } = new URL(request.url)

    const apiUrl = new URL(
      `${getApiBaseUrl()}/orgs/${orgSlug}/projects/${projectKey}/issues`
    )

    searchParams.forEach((value, key) => {
      apiUrl.searchParams.set(key, value)
    })

    const { cookies } = await import("next/headers")
    const accessToken = (await cookies()).get(ACCESS_TOKEN_COOKIE_NAME)?.value
    const headers: HeadersInit = {}
    if (accessToken) {
      headers["Authorization"] = `Bearer ${accessToken}`
    }

    const response = await fetch(apiUrl, {
      cache: "no-store",
      headers,
    })

    const body = await response.text()

    return new NextResponse(body, {
      status: response.status,
      headers: {
        "content-type":
          response.headers.get("content-type") ?? "application/json",
      },
    })
  } catch (error) {
    console.error("Failed to proxy project issues request", error)

    return NextResponse.json(
      { message: "Project issues request failed." },
      { status: 500 }
    )
  }
}

export async function POST(
  request: NextRequest,
  { params }: { params: Promise<{ orgSlug: string; projectKey: string }> }
) {
  try {
    const { orgSlug, projectKey } = await params
    const { cookies } = await import("next/headers")
    const accessToken = (await cookies()).get(ACCESS_TOKEN_COOKIE_NAME)?.value
    const headers: HeadersInit = {
      "Content-Type": "application/json",
    }
    if (accessToken) {
      headers["Authorization"] = `Bearer ${accessToken}`
    }

    const projectRes = await fetch(
      `${getApiBaseUrl()}/orgs/${orgSlug}/projects/${projectKey}`,
      { cache: "no-store", headers }
    )
    if (!projectRes.ok) {
      return NextResponse.json(
        { message: "Project not found" },
        { status: 404 }
      )
    }
    const project = (await projectRes.json()) as { id: string }
    const body = await request.json() as {
      title: string
      description?: string
      status?: string
      priority?: string
      assignee_id?: string | null
    }

    const createRes = await fetch(
      `${getApiBaseUrl()}/projects/${project.id}/issues`,
      {
        method: "POST",
        headers,
        body: JSON.stringify({
          title: body.title,
          description: body.description ?? null,
          status: body.status ?? "backlog",
          priority: body.priority ?? "medium",
          assignee_id: body.assignee_id ?? null,
        }),
      }
    )

    const resBody = await createRes.text()
    return new NextResponse(resBody, {
      status: createRes.status,
      headers: {
        "content-type":
          createRes.headers.get("content-type") ?? "application/json",
      },
    })
  } catch (error) {
    console.error("Failed to create issue", error)
    return NextResponse.json(
      { message: "Failed to create issue" },
      { status: 500 }
    )
  }
}
