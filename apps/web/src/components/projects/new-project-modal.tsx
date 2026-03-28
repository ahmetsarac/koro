"use client"

import * as React from "react"
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Label } from "@/components/ui/label"
import { Input } from "@/components/ui/input"
import { Button } from "@/components/ui/button"

export interface NewProjectModalProps {
  orgSlug: string
  open: boolean
  onOpenChange: (open: boolean) => void
  onSuccess?: () => void
}

export function NewProjectModal({
  orgSlug,
  open,
  onOpenChange,
  onSuccess,
}: NewProjectModalProps) {
  const [name, setName] = React.useState("")
  const [projectKey, setProjectKey] = React.useState("")
  const [loading, setLoading] = React.useState(false)
  const [error, setError] = React.useState<string | null>(null)

  React.useEffect(() => {
    if (!open) return
    setName("")
    setProjectKey("")
    setError(null)
    setLoading(false)
  }, [open])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    const trimmedName = name.trim()
    const key = projectKey.trim().toUpperCase()
    if (trimmedName.length === 0 || key.length < 2) {
      setError("Name and a 2–6 character project key are required.")
      return
    }

    setError(null)
    setLoading(true)
    try {
      const res = await fetch(`/api/orgs/${orgSlug}/projects`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        credentials: "same-origin",
        body: JSON.stringify({
          name: trimmedName,
          project_key: key,
          description: null,
        }),
      })
      const data = (await res.json().catch(() => ({}))) as { message?: string }
      if (!res.ok) {
        const msg =
          res.status === 403
            ? "You need org admin permission to create a project."
            : (data.message ?? "Could not create project.")
        throw new Error(msg)
      }
      onOpenChange(false)
      onSuccess?.()
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not create project.")
    } finally {
      setLoading(false)
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>New project</DialogTitle>
        </DialogHeader>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="new-project-name">Name</Label>
            <Input
              id="new-project-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g. Web application"
              autoComplete="off"
              disabled={loading}
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="new-project-key">Key</Label>
            <Input
              id="new-project-key"
              value={projectKey}
              onChange={(e) =>
                setProjectKey(
                  e.target.value.replace(/[^a-zA-Z0-9]/g, "").slice(0, 6)
                )
              }
              placeholder="APP"
              maxLength={6}
              autoComplete="off"
              disabled={loading}
              className="font-mono uppercase"
            />
            <p className="text-xs text-muted-foreground">
              2–6 letters or numbers. Used in issue keys (e.g. APP-1).
            </p>
          </div>
          {error && <p className="text-sm text-destructive">{error}</p>}
          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
              disabled={loading}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={loading}>
              {loading ? "Creating…" : "Create"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
