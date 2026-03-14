import { cookies } from "next/headers"
import { redirect } from "next/navigation"

import { getApiBaseUrl } from "@/lib/api/backend"
import { ACCESS_TOKEN_COOKIE_NAME } from "@/lib/auth/constants"

async function getDefaultOrg(): Promise<string | null> {
  const cookieStore = await cookies()
  const accessToken = cookieStore.get(ACCESS_TOKEN_COOKIE_NAME)?.value

  if (!accessToken) {
    return null
  }

  try {
    const response = await fetch(`${getApiBaseUrl()}/me`, {
      cache: "no-store",
      headers: {
        Authorization: `Bearer ${accessToken}`,
      },
    })

    if (!response.ok) {
      return null
    }

    const data = await response.json()
    const orgs = data.organizations as { slug: string }[]

    if (orgs && orgs.length > 0) {
      return orgs[0].slug
    }

    return null
  } catch {
    return null
  }
}

export default async function Home() {
  const defaultOrg = await getDefaultOrg()

  if (defaultOrg) {
    redirect(`/${defaultOrg}/my-issues`)
  }
  // No org: either unauthenticated (onboarding will redirect to login) or new user (show onboarding)
  redirect("/onboarding")
}
