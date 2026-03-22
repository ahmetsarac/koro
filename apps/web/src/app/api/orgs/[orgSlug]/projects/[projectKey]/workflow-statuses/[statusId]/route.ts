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

export async function PATCH(
  request: NextRequest,
  {
    params,
  }: {
    params: Promise<{ orgSlug: string; projectKey: string; statusId: string }>
  }
) {
  try {
    const { orgSlug, projectKey, statusId } = await params
    const body = await request.text()
    const apiUrl = new URL(
      `${getApiBaseUrl()}/orgs/${orgSlug}/projects/${projectKey}/workflow-statuses/${statusId}`
    )
    const headers: HeadersInit = {
      ...authHeaders(request),
      "Content-Type": "application/json",
    }
    const response = await fetch(apiUrl, {
      method: "PATCH",
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
    console.error("Failed to proxy workflow-status PATCH", error)
    return NextResponse.json({ message: "Request failed." }, { status: 500 })
  }
}

export async function DELETE(
  request: NextRequest,
  {
    params,
  }: {
    params: Promise<{ orgSlug: string; projectKey: string; statusId: string }>
  }
) {
  try {
    const { orgSlug, projectKey, statusId } = await params
    const { searchParams } = new URL(request.url)
    const reassignTo = searchParams.get("reassign_to")
    if (!reassignTo) {
      return NextResponse.json(
        { message: "reassign_to query parameter required" },
        { status: 400 }
      )
    }
    const apiUrl = new URL(
      `${getApiBaseUrl()}/orgs/${orgSlug}/projects/${projectKey}/workflow-statuses/${statusId}`
    )
    apiUrl.searchParams.set("reassign_to", reassignTo)
    const response = await fetch(apiUrl, {
      method: "DELETE",
      cache: "no-store",
      headers: authHeaders(request),
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
    console.error("Failed to proxy workflow-status DELETE", error)
    return NextResponse.json({ message: "Request failed." }, { status: 500 })
  }
}
