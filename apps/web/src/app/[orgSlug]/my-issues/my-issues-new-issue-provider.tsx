"use client"

import { NewIssueModalProvider } from "@/components/issues/new-issue-modal-context"

export function MyIssuesNewIssueProvider({
  orgSlug,
  children,
}: {
  orgSlug: string
  children: React.ReactNode
}) {
  return (
    <NewIssueModalProvider orgSlug={orgSlug}>
      {children}
    </NewIssueModalProvider>
  )
}
