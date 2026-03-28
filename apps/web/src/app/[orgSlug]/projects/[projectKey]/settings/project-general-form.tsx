"use client"

import * as React from "react"
import { useRouter } from "next/navigation"
import { z } from "zod"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"

import { ProjectDeleteSection } from "./project-delete-section"

const patchResponseSchema = z.object({
  id: z.string().uuid(),
  project_key: z.string(),
  name: z.string(),
})

export function ProjectGeneralSettingsForm({
  orgSlug,
  projectKey,
  initialName,
  canEdit,
}: {
  orgSlug: string
  projectKey: string
  initialName: string
  canEdit: boolean
}) {
  const router = useRouter()
  const [name, setName] = React.useState(initialName)
  const [saving, setSaving] = React.useState(false)
  const [error, setError] = React.useState<string | null>(null)
  const [savedAt, setSavedAt] = React.useState<number | null>(null)

  React.useEffect(() => {
    setName(initialName)
  }, [initialName])

  const trimmed = name.trim()
  const dirty = trimmed !== initialName.trim()
  const canSave = canEdit && trimmed.length > 0 && dirty && !saving

  async function handleSave(e: React.FormEvent) {
    e.preventDefault()
    if (!canSave) return
    setSaving(true)
    setError(null)
    try {
      const res = await fetch(`/api/orgs/${orgSlug}/projects/${projectKey}`, {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        credentials: "same-origin",
        body: JSON.stringify({ name: trimmed }),
      })
      const data = await res.json().catch(() => ({})) as { message?: string }
      if (!res.ok) {
        const msg =
          typeof data?.message === "string" && data.message.length > 0
            ? data.message
            : res.status === 403
              ? "You don't have permission to change project settings."
              : "Could not save project name."
        throw new Error(msg)
      }
      const parsed = patchResponseSchema.parse(data)
      setName(parsed.name)
      setSavedAt(Date.now())
      router.refresh()
    } catch (err) {
      setError(err instanceof Error ? err.message : "Save failed.")
    } finally {
      setSaving(false)
    }
  }

  return (
    <section className="space-y-4">
      <div>
        <h2 className="text-lg font-medium">General</h2>
        <p className="text-sm text-muted-foreground">
          Basic details for this project.
        </p>
      </div>
      <form onSubmit={handleSave} className="space-y-4">
        <div className="space-y-2">
          <Label htmlFor="project-settings-name">Project name</Label>
          <Input
            id="project-settings-name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            disabled={!canEdit || saving}
            autoComplete="off"
            maxLength={200}
          />
          {!canEdit && (
            <p className="text-xs text-muted-foreground">
              Only project managers and organization admins can rename a project.
            </p>
          )}
        </div>
        {error && <p className="text-sm text-destructive">{error}</p>}
        {savedAt !== null && !error && !dirty && (
          <p className="text-sm text-muted-foreground">Saved.</p>
        )}
        {canEdit && (
          <Button type="submit" disabled={!canSave}>
            {saving ? "Saving…" : "Save"}
          </Button>
        )}
      </form>
      <ProjectDeleteSection
        orgSlug={orgSlug}
        projectKey={projectKey}
        projectName={initialName}
        canDelete={canEdit}
      />
    </section>
  )
}
