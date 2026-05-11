"use client"

import { type Row } from "@tanstack/react-table"
import { MoreHorizontal } from "lucide-react"

import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import {
  useConfirmDialog,
  useAlertDialog,
} from "@/components/ui/confirm-dialog"

import { issueSchema } from "../data/schema"

interface DataTableRowActionsProps<TData> {
  row: Row<TData>
}

export function DataTableRowActions<TData>({
  row,
}: DataTableRowActionsProps<TData>) {
  const issue = issueSchema.parse(row.original)
  const { confirm, dialog: confirmDialog } = useConfirmDialog()
  const { alert, dialog: alertDialog } = useAlertDialog()

  const handleArchive = async () => {
    const ok = await confirm({
      title: "Archive Issue",
      description: `Are you sure you want to archive ${issue.display_key}?`,
      confirmLabel: "Archive",
    })
    if (!ok) return
    try {
      const res = await fetch("/api/my-issues/bulk", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        credentials: "same-origin",
        body: JSON.stringify({ archive: { issue_ids: [issue.id] } }),
      })
      if (!res.ok) {
        const data = await res.json().catch(() => ({})) as { message?: string }
        throw new Error(
          typeof data?.message === "string" ? data.message : "Archive failed."
        )
      }
      const projectKey = issue.display_key.split("-")[0] || ""
      window.dispatchEvent(
        new CustomEvent("koro:issue-archived", {
          detail: { issueId: issue.id, projectKey, delta: -1 },
        })
      )
    } catch (e) {
      await alert({
        title: "Error",
        description:
          e instanceof Error ? e.message : "Could not archive issue.",
      })
    }
  }

  return (
    <>
      {confirmDialog}
      {alertDialog}
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            variant="ghost"
            size="icon"
            className="size-8 data-[state=open]:bg-muted"
          >
            <MoreHorizontal />
            <span className="sr-only">Open menu</span>
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-[160px]">
          <DropdownMenuItem onClick={handleArchive}>Archive</DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </>
  )
}
