"use client"

import * as React from "react"
import { useRouter } from "next/navigation"

import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { clearAllIssuesCaches } from "@/lib/cache/issues-cache"

export function ProjectDeleteSection({
  orgSlug,
  projectKey,
  projectName,
  canDelete,
}: {
  orgSlug: string
  projectKey: string
  projectName: string
  canDelete: boolean
}) {
  const router = useRouter()
  const [open, setOpen] = React.useState(false)
  const [confirmName, setConfirmName] = React.useState("")
  const [confirmKey, setConfirmKey] = React.useState("")
  const [deleting, setDeleting] = React.useState(false)
  const [error, setError] = React.useState<string | null>(null)

  const keyExpected = projectKey.trim().toUpperCase()
  const nameExpected = projectName.trim()

  React.useEffect(() => {
    if (!open) return
    setConfirmName("")
    setConfirmKey("")
    setError(null)
    setDeleting(false)
  }, [open])

  const nameOk = confirmName.trim() === nameExpected
  const keyOk = confirmKey.trim().toUpperCase() === keyExpected
  const canSubmit = canDelete && nameOk && keyOk && !deleting

  async function handleDelete() {
    if (!canSubmit) return
    setDeleting(true)
    setError(null)
    try {
      const res = await fetch(
        `/api/orgs/${orgSlug}/projects/${projectKey}`,
        {
          method: "DELETE",
          headers: { "Content-Type": "application/json" },
          credentials: "same-origin",
          body: JSON.stringify({
            confirm_name: confirmName.trim(),
            confirm_project_key: confirmKey.trim(),
          }),
        }
      )
      if (!res.ok) {
        const data = await res.json().catch(() => ({})) as { message?: string }
        throw new Error(
          typeof data?.message === "string" && data.message.length > 0
            ? data.message
            : res.status === 403
              ? "You don't have permission to delete this project."
              : "Could not delete project."
        )
      }
      clearAllIssuesCaches()
      setOpen(false)
      router.replace(`/${orgSlug}/projects`)
      router.refresh()
    } catch (err) {
      setError(err instanceof Error ? err.message : "Delete failed.")
    } finally {
      setDeleting(false)
    }
  }

  if (!canDelete) {
    return null
  }

  return (
    <section className="space-y-4 border-t border-destructive/20 pt-8">
      <div>
        <h2 className="text-lg font-medium text-destructive">Danger zone</h2>
        <p className="text-sm text-muted-foreground">
          Permanently delete this project and all issues, comments, and related
          data. This cannot be undone.
        </p>
      </div>
      <Button
        type="button"
        variant="destructive"
        onClick={() => setOpen(true)}
      >
        Delete this project
      </Button>

      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent
          className="sm:max-w-md"
          showCloseButton={!deleting}
        >
          <DialogHeader>
            <DialogTitle>Delete this project?</DialogTitle>
            <DialogDescription asChild>
              <span className="block space-y-2 text-xs/relaxed text-muted-foreground">
                <span>
                  This will permanently delete{" "}
                  <span className="font-medium text-foreground">
                    {projectName}
                  </span>{" "}
                  (<span className="font-mono">{keyExpected}</span>) and all
                  issues and other data in it.
                </span>
              </span>
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="delete-confirm-name">
                Type the project name{" "}
                <span className="font-mono font-medium text-foreground">
                  {nameExpected}
                </span>{" "}
                to confirm
              </Label>
              <Input
                id="delete-confirm-name"
                value={confirmName}
                onChange={(e) => setConfirmName(e.target.value)}
                autoComplete="off"
                disabled={deleting}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="delete-confirm-key">
                Type the project key{" "}
                <span className="font-mono font-medium text-foreground">
                  {keyExpected}
                </span>{" "}
                to confirm
              </Label>
              <Input
                id="delete-confirm-key"
                value={confirmKey}
                onChange={(e) => setConfirmKey(e.target.value)}
                autoComplete="off"
                disabled={deleting}
              />
            </div>
            {error && (
              <p className="text-sm text-destructive">{error}</p>
            )}
          </div>
          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => setOpen(false)}
              disabled={deleting}
            >
              Cancel
            </Button>
            <Button
              type="button"
              variant="destructive"
              disabled={!canSubmit}
              onClick={handleDelete}
            >
              {deleting ? "Deleting…" : "Delete project"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </section>
  )
}
