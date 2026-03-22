import { NextRequest, NextResponse } from "next/server"

import { getApiBaseUrl } from "@/lib/api/backend"
import { ACCESS_TOKEN_COOKIE_NAME } from "@/lib/auth/constants"

export async function DELETE(
  request: NextRequest,
  {
    params,
  }: {
    params: Promise<{
      orgSlug: string
      issueKey: string
      relationId: string
    }>
  }
) {
  try {
    const { orgSlug, issueKey, relationId } = await params
    const apiUrl = `${getApiBaseUrl()}/orgs/${orgSlug}/issues/${issueKey}/relations/${relationId}`
    const accessToken = request.cookies.get(ACCESS_TOKEN_COOKIE_NAME)?.value
    const headers: HeadersInit = {}
    if (accessToken) headers["Authorization"] = `Bearer ${accessToken}`

    const response = await fetch(apiUrl, {
      method: "DELETE",
      cache: "no-store",
      headers,
    })

    if (response.status === 204) {
      return new NextResponse(null, { status: 204 })
    }

    const body = await response.text()
    return new NextResponse(body, {
      status: response.status,
      headers: {
        "content-type":
          response.headers.get("content-type") ?? "application/json",
      },
    })
  } catch (error) {
    console.error("Failed to proxy relation DELETE", error)
    return NextResponse.json(
      { message: "Failed to remove relation" },
      { status: 500 }
    )
  }
}
