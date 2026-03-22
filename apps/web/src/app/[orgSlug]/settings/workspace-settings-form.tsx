"use client"

import * as React from "react"
import { useRouter } from "next/navigation"
import { z } from "zod"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"

const patchOrgResponseSchema = z.object({
  id: z.string().uuid(),
  name: z.string(),
  slug: z.string(),
})

export function WorkspaceSettingsForm({
  orgSlug,
  initialName,
  canEdit,
}: {
  orgSlug: string
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
      const res = await fetch(`/api/orgs/${orgSlug}`, {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        credentials: "same-origin",
        body: JSON.stringify({ name: trimmed }),
      })
      const data = await res.json().catch(() => ({}))
      if (!res.ok) {
        const msg =
          typeof data?.message === "string"
            ? data.message
            : "Could not save workspace name."
        throw new Error(msg)
      }
      const parsed = patchOrgResponseSchema.parse(data)
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
    <form onSubmit={handleSave} className="max-w-md space-y-4">
      <div className="space-y-2">
        <Label htmlFor="workspace-name">Organization name</Label>
        <Input
          id="workspace-name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          disabled={!canEdit || saving}
          autoComplete="organization"
          maxLength={200}
        />
        {!canEdit && (
          <p className="text-xs text-muted-foreground">
            Only organization admins can change the workspace name.
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
  )
}
