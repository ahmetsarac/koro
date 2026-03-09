export function getApiBaseUrl() {
  const apiBaseUrl = process.env.NEXT_PUBLIC_API_URL

  if (!apiBaseUrl) {
    throw new Error("NEXT_PUBLIC_API_URL is not configured.")
  }

  return apiBaseUrl.replace(/\/$/, "")
}
