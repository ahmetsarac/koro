"use client"

import * as React from "react"
import { NewIssueModal } from "./new-issue-modal"

const NewIssueModalContext = React.createContext<{
  openNewIssueModal: () => void
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
  const value = React.useMemo(
    () => ({ openNewIssueModal: () => setOpen(true) }),
    []
  )
  return (
    <NewIssueModalContext.Provider value={value}>
      {children}
      <NewIssueModal
        open={open}
        onOpenChange={setOpen}
        orgSlug={orgSlug}
        initialProjectKey={undefined}
      />
    </NewIssueModalContext.Provider>
  )
}
