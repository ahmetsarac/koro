"use client"

import * as React from "react"
import { NewIssueModal } from "./new-issue-modal"

function looksLikeUuid(s: string): boolean {
  return /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(s)
}

const NewIssueModalContext = React.createContext<{
  openNewIssueModal: (columnHint?: string) => void
} | null>(null)

export function useNewIssueModal() {
  return React.useContext(NewIssueModalContext)
}

export function NewIssueModalProvider({
  orgSlug,
  children,
}: {
  orgSlug: string
  children: React.ReactNode
}) {
  const [open, setOpen] = React.useState(false)
  const [initialWorkflowStatusId, setInitialWorkflowStatusId] = React.useState<
    string | undefined
  >(undefined)
  const [initialWorkflowCategory, setInitialWorkflowCategory] = React.useState<
    string | undefined
  >(undefined)

  const value = React.useMemo(
    () => ({
      openNewIssueModal: (hint?: string) => {
        if (!hint) {
          setInitialWorkflowStatusId(undefined)
          setInitialWorkflowCategory(undefined)
        } else if (looksLikeUuid(hint)) {
          setInitialWorkflowStatusId(hint)
          setInitialWorkflowCategory(undefined)
        } else {
          setInitialWorkflowStatusId(undefined)
          setInitialWorkflowCategory(hint)
        }
        setOpen(true)
      },
    }),
    []
  )

  const handleOpenChange = React.useCallback((next: boolean) => {
    setOpen(next)
    if (!next) {
      setInitialWorkflowStatusId(undefined)
      setInitialWorkflowCategory(undefined)
    }
  }, [])

  return (
    <NewIssueModalContext.Provider value={value}>
      {children}
      <NewIssueModal
        open={open}
        onOpenChange={handleOpenChange}
        orgSlug={orgSlug}
        initialProjectKey={undefined}
        initialWorkflowStatusId={initialWorkflowStatusId}
        initialWorkflowCategory={initialWorkflowCategory}
      />
    </NewIssueModalContext.Provider>
  )
}
