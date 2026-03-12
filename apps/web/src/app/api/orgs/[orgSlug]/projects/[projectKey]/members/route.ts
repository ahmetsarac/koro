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
      `${getApiBaseUrl()}/orgs/${orgSlug}/projects/${projectKey}/members`
    )

    searchParams.forEach((value, key) => {
      apiUrl.searchParams.set(key, value)
    })

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
    console.error("Failed to proxy project members request", error)

    return NextResponse.json(
      { message: "Project members request failed." },
      { status: 500 }
    )
  }
}
