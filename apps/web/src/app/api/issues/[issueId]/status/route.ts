import { NextRequest, NextResponse } from "next/server"

import { getApiBaseUrl } from "@/lib/api/backend"
import { ACCESS_TOKEN_COOKIE_NAME } from "@/lib/auth/constants"

export async function PATCH(
  request: NextRequest,
  { params }: { params: Promise<{ issueId: string }> }
) {
  try {
    const { issueId } = await params
    const body = await request.json()

    const apiUrl = new URL(`${getApiBaseUrl()}/issues/${issueId}/status`)

    const accessToken = request.cookies.get(ACCESS_TOKEN_COOKIE_NAME)?.value
    const headers: HeadersInit = {
      "Content-Type": "application/json",
    }
    if (accessToken) {
      headers["Authorization"] = `Bearer ${accessToken}`
    }

    const response = await fetch(apiUrl, {
      method: "PATCH",
      cache: "no-store",
      headers,
      body: JSON.stringify(body),
    })

    const responseBody = await response.text()

    return new NextResponse(responseBody, {
      status: response.status,
      headers: {
        "content-type":
          response.headers.get("content-type") ?? "application/json",
      },
    })
  } catch (error) {
    console.error("Failed to proxy status update request", error)

    return NextResponse.json(
      { message: "Status update request failed." },
      { status: 500 }
    )
  }
}
