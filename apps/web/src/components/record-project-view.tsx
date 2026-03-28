"use client"

import * as React from "react"

export function RecordProjectView({
  orgSlug,
  projectKey,
}: {
  orgSlug: string
  projectKey: string
}) {
  React.useEffect(() => {
    const key = projectKey.trim()
    if (!orgSlug || !key) return
    void fetch(`/api/orgs/${orgSlug}/projects/${encodeURIComponent(key)}/view`, {
      method: "POST",
      credentials: "same-origin",
    })
  }, [orgSlug, projectKey])

  return null
}
