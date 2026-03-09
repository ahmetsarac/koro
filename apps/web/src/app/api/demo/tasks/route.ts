import { NextRequest, NextResponse } from "next/server"

import { getApiBaseUrl } from "@/lib/api/backend"

export async function GET(request: NextRequest) {
  try {
    const apiUrl = new URL(`${getApiBaseUrl()}/demo/tasks`)

    request.nextUrl.searchParams.forEach((value, key) => {
      apiUrl.searchParams.set(key, value)
    })

    const response = await fetch(apiUrl, {
      cache: "no-store",
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
    console.error("Failed to proxy demo tasks request", error)

    return NextResponse.json(
      { message: "Demo tasks request failed." },
      { status: 500 }
    )
  }
}
