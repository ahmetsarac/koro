import { NextRequest, NextResponse } from "next/server"

import { getApiBaseUrl } from "@/lib/api/backend"
import { ACCESS_TOKEN_COOKIE_NAME } from "@/lib/auth/constants"

export async function POST(
  request: NextRequest,
  { params }: { params: Promise<{ orgSlug: string }> },
) {
  try {
    const { orgSlug } = await params
    const accessToken = request.cookies.get(ACCESS_TOKEN_COOKIE_NAME)?.value
    if (!accessToken) {
      return NextResponse.json({ message: "Unauthorized" }, { status: 401 })
    }

    const body = await request.json() as {
      project_key?: string
      name?: string
      description?: string
    }
    if (!body?.project_key || !body?.name) {
      return NextResponse.json(
        { message: "project_key and name are required" },
        { status: 400 },
      )
    }

    const response = await fetch(
      `${getApiBaseUrl()}/orgs/${orgSlug}/projects`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${accessToken}`,
        },
        body: JSON.stringify({
          project_key: body.project_key,
          name: body.name,
          description: body.description ?? null,
        }),
      },
    )

    const data = await response.text()
    if (!response.ok) {
      return NextResponse.json(
        { message: data || "Failed to create project" },
        { status: response.status },
      )
    }
    return new NextResponse(data, {
      status: 201,
      headers: { "Content-Type": "application/json" },
    })
  } catch (error) {
    console.error("Create project failed", error)
    return NextResponse.json(
      { message: "Failed to create project" },
      { status: 500 },
    )
  }
}
