import { NextRequest, NextResponse } from "next/server"

import { getApiBaseUrl } from "@/lib/api/backend"
import { ACCESS_TOKEN_COOKIE_NAME } from "@/lib/auth/constants"

export async function PATCH(
  request: NextRequest,
  { params }: { params: Promise<{ orgSlug: string; issueKey: string }> }
) {
  try {
    const { orgSlug, issueKey } = await params

    const apiUrl = `${getApiBaseUrl()}/orgs/${orgSlug}/issues/${issueKey}/assignee`

    const accessToken = request.cookies.get(ACCESS_TOKEN_COOKIE_NAME)?.value
    const headers: HeadersInit = {
      "Content-Type": "application/json",
    }
    if (accessToken) {
      headers["Authorization"] = `Bearer ${accessToken}`
    }

    const reqBody = await request.json()

    const response = await fetch(apiUrl, {
      method: "PATCH",
      cache: "no-store",
      headers,
      body: JSON.stringify(reqBody),
    })

    // 204 No Content has no body; reading it can cause issues in some runtimes
    const body =
      response.status === 204 ? "" : await response.text()

    if (response.status >= 400) {
      console.error(
        "[assignee PATCH] backend returned",
        response.status,
        body || "(no body)"
      )
    }

    // NextResponse with status 204 can throw "Invalid response status" in some runtimes; treat 204 as 200
    const status = response.status === 204 ? 200 : response.status
    return new NextResponse(body, {
      status,
      headers: {
        "content-type":
          response.headers.get("content-type") ?? "application/json",
      },
    })
  } catch (error) {
    console.error("Failed to assign issue (proxy error)", error)

    return NextResponse.json(
      { message: "Failed to assign issue" },
      { status: 500 }
    )
  }
}

export async function DELETE(
  request: NextRequest,
  { params }: { params: Promise<{ orgSlug: string; issueKey: string }> }
) {
  try {
    const { orgSlug, issueKey } = await params

    const apiUrl = `${getApiBaseUrl()}/orgs/${orgSlug}/issues/${issueKey}/assignee`

    const accessToken = request.cookies.get(ACCESS_TOKEN_COOKIE_NAME)?.value
    const headers: HeadersInit = {}
    if (accessToken) {
      headers["Authorization"] = `Bearer ${accessToken}`
    }

    const response = await fetch(apiUrl, {
      method: "DELETE",
      cache: "no-store",
      headers,
    })

    // 204 No Content has no body; reading it can cause issues in some runtimes
    const body =
      response.status === 204 ? "" : await response.text()

    if (response.status >= 400) {
      console.error(
        "[assignee DELETE] backend returned",
        response.status,
        body || "(no body)"
      )
    }

    // NextResponse with status 204 can throw "Invalid response status" in some runtimes; treat 204 as 200
    const status = response.status === 204 ? 200 : response.status
    return new NextResponse(body, {
      status,
      headers: {
        "content-type":
          response.headers.get("content-type") ?? "application/json",
      },
    })
  } catch (error) {
    console.error("Failed to unassign issue (proxy error)", error)

    return NextResponse.json(
      { message: "Failed to unassign issue" },
      { status: 500 }
    )
  }
}
