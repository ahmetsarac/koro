import { NextRequest, NextResponse } from "next/server"

import { getApiBaseUrl } from "@/lib/api/backend"
import { ACCESS_TOKEN_COOKIE_NAME } from "@/lib/auth/constants"

export async function PATCH(
  request: NextRequest,
  { params }: { params: Promise<{ orgSlug: string }> },
) {
  try {
    const { orgSlug } = await params
    const accessToken = request.cookies.get(ACCESS_TOKEN_COOKIE_NAME)?.value
    if (!accessToken) {
      return NextResponse.json({ message: "Unauthorized" }, { status: 401 })
    }

    const body = (await request.json()) as { name?: string }
    const name = typeof body?.name === "string" ? body.name.trim() : ""
    if (!name) {
      return NextResponse.json({ message: "name is required" }, { status: 400 })
    }

    const response = await fetch(`${getApiBaseUrl()}/orgs/${orgSlug}`, {
      method: "PATCH",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${accessToken}`,
      },
      body: JSON.stringify({ name }),
    })

    const text = await response.text()
    if (!response.ok) {
      return NextResponse.json(
        { message: text || "Failed to update organization" },
        { status: response.status },
      )
    }

    return new NextResponse(text, {
      status: 200,
      headers: {
        "Content-Type": "application/json",
      },
    })
  } catch (error) {
    console.error("Patch org failed", error)
    return NextResponse.json(
      { message: "Failed to update organization" },
      { status: 500 },
    )
  }
}
