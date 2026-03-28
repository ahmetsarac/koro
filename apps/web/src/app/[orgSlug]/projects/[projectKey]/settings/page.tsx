import { cookies } from "next/headers"

import { getApiBaseUrl } from "@/lib/api/backend"
import { ACCESS_TOKEN_COOKIE_NAME } from "@/lib/auth/constants"
import { meResponseSchema } from "@/lib/user"

import { ProjectGeneralSettingsForm } from "./project-general-form"

export default async function ProjectGeneralSettingsPage({
  params,
}: {
  params: Promise<{ orgSlug: string; projectKey: string }>
}) {
  const { orgSlug, projectKey } = await params

  const cookieStore = await cookies()
  const accessToken = cookieStore.get(ACCESS_TOKEN_COOKIE_NAME)?.value

  if (!accessToken) {
    return (
      <p className="text-sm text-muted-foreground">
        Sign in to manage project settings.
      </p>
    )
  }

  const headers = { Authorization: `Bearer ${accessToken}` }

  const [meResponse, projectResponse] = await Promise.all([
    fetch(`${getApiBaseUrl()}/me`, { cache: "no-store", headers }),
    fetch(`${getApiBaseUrl()}/orgs/${orgSlug}/projects/${projectKey}`, {
      cache: "no-store",
      headers,
    }),
  ])

  if (!meResponse.ok || !projectResponse.ok) {
    return (
      <p className="text-sm text-muted-foreground">
        Could not load project settings.
      </p>
    )
  }

  const me = meResponseSchema.parse(await meResponse.json())
  const project = (await projectResponse.json()) as {
    name: string
    my_role: string
  }

  const org = me.organizations.find((o) => o.slug === orgSlug)
  const canEdit =
    org?.role === "org_admin" || project.my_role === "project_manager"

  return (
    <ProjectGeneralSettingsForm
      orgSlug={orgSlug}
      projectKey={projectKey}
      initialName={project.name}
      canEdit={canEdit}
    />
  )
}
