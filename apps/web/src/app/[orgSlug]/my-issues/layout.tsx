import { cookies } from "next/headers"

import { MyIssuesViewProvider } from "./components/my-issues-view-context"
import { MyIssuesTabs } from "./components/my-issues-tabs"
import { NewIssueModalProvider } from "@/components/issues/new-issue-modal-context"
import { MY_ISSUES_VIEW_COOKIE } from "./constants"

export default async function MyIssuesLayout({
  children,
  params,
}: {
  children: React.ReactNode
  params: Promise<{ orgSlug: string }>
}) {
  const { orgSlug } = await params
  const cookieStore = await cookies()
  const viewCookie = cookieStore.get(MY_ISSUES_VIEW_COOKIE)?.value
  const initialView = viewCookie === "board" ? "board" : "list"

  return (
    <NewIssueModalProvider orgSlug={orgSlug}>
      <MyIssuesViewProvider initialView={initialView}>
        <section className="flex min-h-0 flex-col gap-4 h-[calc(100svh-4.5rem)] w-[calc(100svw-var(--sidebar-width)-3rem)]">
          <h1 className="text-2xl font-semibold">My Issues</h1>
          <MyIssuesTabs />
          {children}
        </section>
      </MyIssuesViewProvider>
    </NewIssueModalProvider>
  )
}
