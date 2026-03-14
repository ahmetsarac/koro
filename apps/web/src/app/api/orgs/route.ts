import { NextRequest, NextResponse } from "next/server"

import { getApiBaseUrl } from "@/lib/api/backend"
import { ACCESS_TOKEN_COOKIE_NAME } from "@/lib/auth/constants"

export async function POST(request: NextRequest) {
  try {
    const accessToken = request.cookies.get(ACCESS_TOKEN_COOKIE_NAME)?.value
    if (!accessToken) {
      return NextResponse.json({ message: "Unauthorized" }, { status: 401 })
    }

    const body = await request.json() as { name?: string; slug?: string }
    if (!body?.name || !body?.slug) {
      return NextResponse.json(
        { message: "name and slug are required" },
        { status: 400 },
      )
    }

    const response = await fetch(`${getApiBaseUrl()}/orgs`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${accessToken}`,
      },
      body: JSON.stringify({ name: body.name, slug: body.slug }),
    })

    const data = await response.text()
    if (!response.ok) {
      return NextResponse.json(
        { message: data || "Failed to create organization" },
        { status: response.status },
      )
    }
    return new NextResponse(data, {
      status: 201,
      headers: { "Content-Type": "application/json" },
    })
  } catch (error) {
    console.error("Create org failed", error)
    return NextResponse.json(
      { message: "Failed to create organization" },
      { status: 500 },
    )
  }
}
