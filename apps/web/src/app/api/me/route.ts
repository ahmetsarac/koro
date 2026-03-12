import { NextRequest, NextResponse } from "next/server"

import { getApiBaseUrl } from "@/lib/api/backend"
import { ACCESS_TOKEN_COOKIE_NAME } from "@/lib/auth/constants"

export async function GET(request: NextRequest) {
  try {
    const apiUrl = new URL(`${getApiBaseUrl()}/me`)

    const accessToken = request.cookies.get(ACCESS_TOKEN_COOKIE_NAME)?.value
    if (!accessToken) {
      return NextResponse.json({ message: "Unauthorized" }, { status: 401 })
    }

    const response = await fetch(apiUrl, {
      cache: "no-store",
      headers: {
        Authorization: `Bearer ${accessToken}`,
      },
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
    console.error("Failed to fetch user info", error)

    return NextResponse.json(
      { message: "Failed to fetch user info." },
      { status: 500 }
    )
  }
}
