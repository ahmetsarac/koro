import { MyIssuesTabs } from "./components/my-issues-tabs"

export default function MyIssuesLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <section className="flex min-h-0 flex-col gap-4 h-[calc(100svh-4.5rem)]">
      <h1 className="text-2xl font-semibold">My Issues</h1>
      <MyIssuesTabs />
      {children}
    </section>
  )
}
