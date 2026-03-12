import { NextRequest, NextResponse } from "next/server"

import { getApiBaseUrl } from "@/lib/api/backend"
import { ACCESS_TOKEN_COOKIE_NAME } from "@/lib/auth/constants"

export async function PATCH(
  request: NextRequest,
  { params }: { params: Promise<{ issueId: string }> }
) {
  try {
    const { issueId } = await params

    const apiUrl = `${getApiBaseUrl()}/issues/${issueId}/status`

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
    console.error("Failed to update issue status", error)

    return NextResponse.json(
      { message: "Failed to update issue status" },
      { status: 500 }
    )
  }
}
