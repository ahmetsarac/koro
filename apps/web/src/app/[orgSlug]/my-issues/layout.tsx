import { cookies } from "next/headers"

import { MyIssuesViewProvider } from "./components/my-issues-view-context"
import { MyIssuesTabs } from "./components/my-issues-tabs"
import { MY_ISSUES_VIEW_COOKIE } from "./constants"
import { MyIssuesLayoutContent } from "./my-issues-layout-content"
import { MyIssuesNewIssueProvider } from "./my-issues-new-issue-provider"

export default async function MyIssuesLayout({
  children,
  params,
}: {
  children: React.ReactNode
  params: Promise<{ orgSlug: string }>
}) {
  const { orgSlug } = await params
  const cookieStore = await cookies()
  const viewCookie = cookieStore.get(MY_ISSUES_VIEW_COOKIE)
  const initialView = viewCookie?.value === "board" ? "board" : "list"

  return (
    <MyIssuesNewIssueProvider orgSlug={orgSlug}>
      <MyIssuesViewProvider initialView={initialView}>
        <MyIssuesLayoutContent>
          <h1 className="text-2xl font-semibold">My Issues</h1>
          <MyIssuesTabs />
          {children}
        </MyIssuesLayoutContent>
      </MyIssuesViewProvider>
    </MyIssuesNewIssueProvider>
  )
}
