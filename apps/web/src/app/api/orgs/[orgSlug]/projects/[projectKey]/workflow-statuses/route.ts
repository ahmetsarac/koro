import { NextRequest, NextResponse } from "next/server"

import { getApiBaseUrl } from "@/lib/api/backend"
import { ACCESS_TOKEN_COOKIE_NAME } from "@/lib/auth/constants"

function authHeaders(request: NextRequest): HeadersInit {
  const accessToken = request.cookies.get(ACCESS_TOKEN_COOKIE_NAME)?.value
  const headers: HeadersInit = {}
  if (accessToken) {
    headers["Authorization"] = `Bearer ${accessToken}`
  }
  return headers
}

export async function GET(
  request: NextRequest,
  { params }: { params: Promise<{ orgSlug: string; projectKey: string }> }
) {
  try {
    const { orgSlug, projectKey } = await params
    const apiUrl = new URL(
      `${getApiBaseUrl()}/orgs/${orgSlug}/projects/${projectKey}/workflow-statuses`
    )
    const response = await fetch(apiUrl, {
      cache: "no-store",
      headers: authHeaders(request),
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
    console.error("Failed to proxy workflow-statuses GET", error)
    return NextResponse.json({ message: "Request failed." }, { status: 500 })
  }
}

export async function POST(
  request: NextRequest,
  { params }: { params: Promise<{ orgSlug: string; projectKey: string }> }
) {
  try {
    const { orgSlug, projectKey } = await params
    const body = await request.text()
    const apiUrl = new URL(
      `${getApiBaseUrl()}/orgs/${orgSlug}/projects/${projectKey}/workflow-statuses`
    )
    const headers: HeadersInit = {
      ...authHeaders(request),
      "Content-Type": "application/json",
    }
    const response = await fetch(apiUrl, {
      method: "POST",
      cache: "no-store",
      headers,
      body,
    })
    const resBody = await response.text()
    return new NextResponse(resBody, {
      status: response.status,
      headers: {
        "content-type":
          response.headers.get("content-type") ?? "application/json",
      },
    })
  } catch (error) {
    console.error("Failed to proxy workflow-statuses POST", error)
    return NextResponse.json({ message: "Request failed." }, { status: 500 })
  }
}
