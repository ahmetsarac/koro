import { NextRequest, NextResponse } from "next/server"

import { getApiBaseUrl } from "@/lib/api/backend"
import { ACCESS_TOKEN_COOKIE_NAME } from "@/lib/auth/constants"

export async function GET(
  request: NextRequest,
  { params }: { params: Promise<{ orgSlug: string; projectKey: string }> }
) {
  try {
    const { orgSlug, projectKey } = await params
    const apiUrl = new URL(
      `${getApiBaseUrl()}/orgs/${orgSlug}/projects/${projectKey}`
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
    console.error("Failed to proxy project request", error)

    return NextResponse.json(
      { message: "Project request failed." },
      { status: 500 }
    )
  }
}

export async function PATCH(
  request: NextRequest,
  { params }: { params: Promise<{ orgSlug: string; projectKey: string }> }
) {
  try {
    const { orgSlug, projectKey } = await params
    const accessToken = request.cookies.get(ACCESS_TOKEN_COOKIE_NAME)?.value
    if (!accessToken) {
      return NextResponse.json({ message: "Unauthorized" }, { status: 401 })
    }

    const body = (await request.json()) as { name?: string }
    const name =
      typeof body?.name === "string" ? body.name.trim() : ""
    if (!name) {
      return NextResponse.json({ message: "name is required" }, { status: 400 })
    }

    const response = await fetch(
      `${getApiBaseUrl()}/orgs/${orgSlug}/projects/${projectKey}`,
      {
        method: "PATCH",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${accessToken}`,
        },
        body: JSON.stringify({ name }),
      }
    )

    const text = await response.text()
    if (!response.ok) {
      return NextResponse.json(
        { message: text || "Failed to update project" },
        { status: response.status }
      )
    }
    return new NextResponse(text, {
      status: 200,
      headers: { "Content-Type": "application/json" },
    })
  } catch (error) {
    console.error("Failed to patch project", error)
    return NextResponse.json(
      { message: "Failed to update project" },
      { status: 500 }
    )
  }
}
