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

    const body = await response.text()

    return new NextResponse(body, {
      status: response.status,
      headers: {
        "content-type":
          response.headers.get("content-type") ?? "application/json",
      },
    })
  } catch (error) {
    console.error("Failed to assign issue", error)

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

    const body = await response.text()

    return new NextResponse(body, {
      status: response.status,
      headers: {
        "content-type":
          response.headers.get("content-type") ?? "application/json",
      },
    })
  } catch (error) {
    console.error("Failed to unassign issue", error)

    return NextResponse.json(
      { message: "Failed to unassign issue" },
      { status: 500 }
    )
  }
}
