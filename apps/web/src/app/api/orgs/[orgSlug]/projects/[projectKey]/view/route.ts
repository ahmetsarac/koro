import { NextRequest, NextResponse } from "next/server"

import { getApiBaseUrl } from "@/lib/api/backend"
import { ACCESS_TOKEN_COOKIE_NAME } from "@/lib/auth/constants"

export async function POST(
  request: NextRequest,
  { params }: { params: Promise<{ orgSlug: string; projectKey: string }> }
) {
  try {
    const { orgSlug, projectKey } = await params
    const accessToken = request.cookies.get(ACCESS_TOKEN_COOKIE_NAME)?.value
    if (!accessToken) {
      return NextResponse.json({ message: "Unauthorized" }, { status: 401 })
    }

    const apiUrl = `${getApiBaseUrl()}/orgs/${orgSlug}/projects/${projectKey}/view`
    const response = await fetch(apiUrl, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${accessToken}`,
      },
    })

    if (response.status === 204) {
      return new NextResponse(null, { status: 204 })
    }

    const text = await response.text()
    return new NextResponse(text || null, {
      status: response.status,
      headers: {
        "content-type":
          response.headers.get("content-type") ?? "application/json",
      },
    })
  } catch (error) {
    console.error("Failed to proxy project view", error)
    return NextResponse.json({ message: "Request failed." }, { status: 500 })
  }
}
