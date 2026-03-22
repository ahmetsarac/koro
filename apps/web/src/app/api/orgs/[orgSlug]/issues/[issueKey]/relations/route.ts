import { NextRequest, NextResponse } from "next/server"

import { getApiBaseUrl } from "@/lib/api/backend"
import { ACCESS_TOKEN_COOKIE_NAME } from "@/lib/auth/constants"

export async function GET(
  request: NextRequest,
  { params }: { params: Promise<{ orgSlug: string; issueKey: string }> }
) {
  try {
    const { orgSlug, issueKey } = await params
    const apiUrl = `${getApiBaseUrl()}/orgs/${orgSlug}/issues/${issueKey}/relations`
    const accessToken = request.cookies.get(ACCESS_TOKEN_COOKIE_NAME)?.value
    const headers: HeadersInit = {}
    if (accessToken) headers["Authorization"] = `Bearer ${accessToken}`

    const response = await fetch(apiUrl, { cache: "no-store", headers })
    const body = await response.text()
    return new NextResponse(body, {
      status: response.status,
      headers: {
        "content-type":
          response.headers.get("content-type") ?? "application/json",
      },
    })
  } catch (error) {
    console.error("Failed to proxy relations GET", error)
    return NextResponse.json(
      { message: "Relations request failed." },
      { status: 500 }
    )
  }
}

export async function POST(
  request: NextRequest,
  { params }: { params: Promise<{ orgSlug: string; issueKey: string }> }
) {
  try {
    const { orgSlug, issueKey } = await params
    const apiUrl = `${getApiBaseUrl()}/orgs/${orgSlug}/issues/${issueKey}/relations`
    const accessToken = request.cookies.get(ACCESS_TOKEN_COOKIE_NAME)?.value
    const headers: HeadersInit = { "Content-Type": "application/json" }
    if (accessToken) headers["Authorization"] = `Bearer ${accessToken}`

    const reqBody = await request.text()
    const response = await fetch(apiUrl, {
      method: "POST",
      cache: "no-store",
      headers,
      body: reqBody,
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
    console.error("Failed to proxy relations POST", error)
    return NextResponse.json(
      { message: "Failed to create relation" },
      { status: 500 }
    )
  }
}
