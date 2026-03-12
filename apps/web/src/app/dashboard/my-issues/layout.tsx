import { MyIssuesTabs } from "./components/my-issues-tabs"

export default function MyIssuesLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <div className="flex h-[calc(100svh-4.5rem)] min-h-0 flex-col">
      <div className="mb-4">
        <h2 className="text-2xl font-semibold tracking-tight">My Issues</h2>
        <p className="text-muted-foreground">View and manage your issues.</p>
      </div>

      <div className="flex min-h-0 flex-1 flex-col gap-4">
        <MyIssuesTabs />
        {children}
      </div>
    </div>
  )
}
