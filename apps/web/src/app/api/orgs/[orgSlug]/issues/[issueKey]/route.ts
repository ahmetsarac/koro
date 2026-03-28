import { NextRequest, NextResponse } from "next/server"

import { getApiBaseUrl } from "@/lib/api/backend"
import { ACCESS_TOKEN_COOKIE_NAME } from "@/lib/auth/constants"

export async function GET(
  request: NextRequest,
  { params }: { params: Promise<{ orgSlug: string; issueKey: string }> }
) {
  try {
    const { orgSlug, issueKey } = await params

    const apiUrl = new URL(
      `${getApiBaseUrl()}/orgs/${orgSlug}/issues/${issueKey}`
    )

    const accessToken = request.cookies.get(ACCESS_TOKEN_COOKIE_NAME)?.value
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
    console.error("Failed to proxy issue request", error)

    return NextResponse.json(
      { message: "Issue request failed." },
      { status: 500 }
    )
  }
}

export async function PATCH(
  request: NextRequest,
  { params }: { params: Promise<{ orgSlug: string; issueKey: string }> }
) {
  try {
    const { orgSlug, issueKey } = await params

    const apiUrl = `${getApiBaseUrl()}/orgs/${orgSlug}/issues/${issueKey}`

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
    console.error("Failed to update issue", error)

    return NextResponse.json(
      { message: "Failed to update issue" },
      { status: 500 }
    )
  }
}

export async function DELETE(
  _request: NextRequest,
  { params }: { params: Promise<{ orgSlug: string; issueKey: string }> }
) {
  try {
    const { orgSlug, issueKey } = await params

    const accessToken = _request.cookies.get(ACCESS_TOKEN_COOKIE_NAME)?.value
    if (!accessToken) {
      return NextResponse.json({ message: "Unauthorized" }, { status: 401 })
    }

    const response = await fetch(
      `${getApiBaseUrl()}/orgs/${orgSlug}/issues/${issueKey}`,
      {
        method: "DELETE",
        cache: "no-store",
        headers: { Authorization: `Bearer ${accessToken}` },
      }
    )

    if (!response.ok) {
      const text = (await response.text()).trim()
      return NextResponse.json(
        { message: text.length > 0 ? text : "Delete failed." },
        { status: response.status }
      )
    }

    return new NextResponse(null, { status: 204 })
  } catch (error) {
    console.error("Failed to delete issue", error)
    return NextResponse.json(
      { message: "Failed to delete issue" },
      { status: 500 }
    )
  }
}
