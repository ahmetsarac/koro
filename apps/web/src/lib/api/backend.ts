export function getApiBaseUrl() {
  const apiBaseUrl =
    typeof window === "undefined"
      ? (process.env.API_URL ?? process.env.NEXT_PUBLIC_API_URL)
      : process.env.NEXT_PUBLIC_API_URL

  if (!apiBaseUrl) {
    throw new Error(
      "API base URL is not configured (set NEXT_PUBLIC_API_URL; for server-side in Docker also API_URL).",
    )
  }

  return apiBaseUrl.replace(/\/$/, "")
}
