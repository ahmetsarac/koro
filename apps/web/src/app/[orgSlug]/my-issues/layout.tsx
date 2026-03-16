"use client"

import * as React from "react"
import { usePathname } from "next/navigation"

import { MyIssuesViewProvider } from "./components/my-issues-view-context"
import { MyIssuesTabs } from "./components/my-issues-tabs"
import { NewIssueModalProvider } from "@/components/issues/new-issue-modal-context"
import { useSidebar } from "@/components/ui/sidebar"
import { MY_ISSUES_VIEW_COOKIE } from "./constants"

export default function MyIssuesLayout({
  children,
}: {
  children: React.ReactNode
}) {
  const pathname = usePathname()
  const orgSlug = pathname.split("/")[1]

  // Get initial view from cookie on mount
  const [initialView, setInitialView] = React.useState<"list" | "board">(() => {
    if (typeof document === "undefined") return "list"
    const cookies = document.cookie.split(";")
    const viewCookie = cookies.find(c => c.trim().startsWith(`${MY_ISSUES_VIEW_COOKIE}=`))
    return viewCookie?.trim().split("=")[1] === "board" ? "board" : "list"
  })

  return (
    <NewIssueModalProvider orgSlug={orgSlug}>
      <MyIssuesViewProvider initialView={initialView}>
        <MyIssuesLayoutContent>
          <h1 className="text-2xl font-semibold">My Issues</h1>
          <MyIssuesTabs />
          {children}
        </MyIssuesLayoutContent>
      </MyIssuesViewProvider>
    </NewIssueModalProvider>
  )
}

function MyIssuesLayoutContent({ children }: { children: React.ReactNode }) {
  const { state } = useSidebar()

  const sidebarWidth = state === "expanded" ? "var(--sidebar-width)" : "var(--sidebar-width-icon)"

  return (
    <section className="flex min-h-0 flex-col gap-4 h-[calc(100svh-4.5rem)] w-[calc(100svh-var(--sidebar-width)-3rem)]" style={{ width: `calc(100svw - ${sidebarWidth} - 3rem)` }}>
      {children}
    </section>
  )
}
