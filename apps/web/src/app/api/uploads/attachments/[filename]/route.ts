import { NextRequest, NextResponse } from "next/server"

import { getApiBaseUrl } from "@/lib/api/backend"

export async function GET(
  request: NextRequest,
  { params }: { params: Promise<{ filename: string }> },
) {
  try {
    const { filename } = await params
    if (!filename) {
      return NextResponse.json({ message: "Bad request" }, { status: 400 })
    }

    const apiUrl = `${getApiBaseUrl()}/uploads/attachments/${encodeURIComponent(filename)}`

    const response = await fetch(apiUrl, {
      cache: "no-store",
    })

    if (!response.ok) {
      return new NextResponse(null, { status: response.status })
    }

    const blob = await response.blob()
    const contentType =
      response.headers.get("content-type") ?? "application/octet-stream"

    return new NextResponse(blob, {
      status: 200,
      headers: {
        "content-type": contentType,
        "cache-control": response.headers.get("cache-control") ?? "public, max-age=31536000",
      },
    })
  } catch (error) {
    console.error("Failed to fetch attachment", error)
    return NextResponse.json(
      { message: "Failed to fetch attachment." },
      { status: 500 },
    )
  }
}
