import { cookies } from "next/headers"

import { getApiBaseUrl } from "@/lib/api/backend"
import { ACCESS_TOKEN_COOKIE_NAME } from "@/lib/auth/constants"
import { meResponseSchema } from "@/lib/user"

import { WorkspaceSettingsForm } from "./workspace-settings-form"

export default async function WorkspaceSettingsPage({
  params,
}: {
  params: Promise<{ orgSlug: string }>
}) {
  const { orgSlug } = await params

  const cookieStore = await cookies()
  const accessToken = cookieStore.get(ACCESS_TOKEN_COOKIE_NAME)?.value

  if (!accessToken) {
    return (
      <section className="space-y-2">
        <h1 className="text-2xl font-semibold">Workspace settings</h1>
        <p className="text-sm text-muted-foreground">Sign in to manage this workspace.</p>
      </section>
    )
  }

  const meResponse = await fetch(`${getApiBaseUrl()}/me`, {
    cache: "no-store",
    headers: { Authorization: `Bearer ${accessToken}` },
  })

  if (!meResponse.ok) {
    return (
      <section className="space-y-2">
        <h1 className="text-2xl font-semibold">Workspace settings</h1>
        <p className="text-sm text-muted-foreground">Could not load workspace.</p>
      </section>
    )
  }

  const me = meResponseSchema.parse(await meResponse.json())
  const org = me.organizations.find((o) => o.slug === orgSlug)

  if (!org) {
    return (
      <section className="space-y-2">
        <h1 className="text-2xl font-semibold">Workspace settings</h1>
        <p className="text-sm text-muted-foreground">
          You don&apos;t have access to this workspace.
        </p>
      </section>
    )
  }

  const canEdit = org.role === "org_admin"

  return (
    <section className="space-y-6">
      <div className="space-y-1">
        <h1 className="text-2xl font-semibold">Workspace settings</h1>
        <p className="text-sm text-muted-foreground">
          <span className="font-mono text-foreground">{orgSlug}</span>
        </p>
      </div>
      <WorkspaceSettingsForm
        orgSlug={orgSlug}
        initialName={org.name}
        canEdit={canEdit}
      />
    </section>
  )
}
