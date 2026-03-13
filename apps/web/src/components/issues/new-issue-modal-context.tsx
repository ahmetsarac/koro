"use client"

import * as React from "react"
import { NewIssueModal } from "./new-issue-modal"

const NewIssueModalContext = React.createContext<{
  openNewIssueModal: (initialStatus?: string) => void
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
  const [initialStatus, setInitialStatus] = React.useState<string | undefined>(undefined)
  const value = React.useMemo(
    () => ({
      openNewIssueModal: (status?: string) => {
        setInitialStatus(status)
        setOpen(true)
      },
    }),
    []
  )
  const handleOpenChange = React.useCallback((next: boolean) => {
    setOpen(next)
    if (!next) setInitialStatus(undefined)
  }, [])
  return (
    <NewIssueModalContext.Provider value={value}>
      {children}
      <NewIssueModal
        open={open}
        onOpenChange={handleOpenChange}
        orgSlug={orgSlug}
        initialProjectKey={undefined}
        initialStatus={initialStatus}
      />
    </NewIssueModalContext.Provider>
  )
}
