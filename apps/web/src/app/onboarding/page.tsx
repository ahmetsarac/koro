import { cookies } from "next/headers"
import { redirect } from "next/navigation"

import { getApiBaseUrl } from "@/lib/api/backend"
import { ACCESS_TOKEN_COOKIE_NAME } from "@/lib/auth/constants"
import { OnboardingFlow } from "@/components/onboarding/onboarding-flow"

async function getMe() {
  const cookieStore = await cookies()
  const accessToken = cookieStore.get(ACCESS_TOKEN_COOKIE_NAME)?.value
  if (!accessToken) return null
  try {
    const res = await fetch(`${getApiBaseUrl()}/me`, {
      cache: "no-store",
      headers: { Authorization: `Bearer ${accessToken}` },
    })
    if (!res.ok) return null
    return res.json() as Promise<{ organizations: { slug: string }[] }>
  } catch {
    return null
  }
}

export default async function OnboardingPage() {
  const me = await getMe()
  if (!me) {
    redirect("/login")
  }
  if (me.organizations?.length > 0) {
    redirect(`/${me.organizations[0].slug}/my-issues`)
  }
  return (
    <div className="flex min-h-screen items-center justify-center bg-muted/30 p-4">
      <OnboardingFlow />
    </div>
  )
}
