import { type Metadata } from "next"

import { columns } from "./components/columns"
import { DataTable } from "./components/data-table"

export const metadata: Metadata = {
  title: "My Issues",
  description: "View and manage your assigned issues.",
}

export default function MyIssuesPage() {
  return (
    <div className="flex h-[calc(100svh-4.5rem)] min-h-0 flex-col">
      <div className="mb-4">
        <h2 className="text-2xl font-semibold tracking-tight">My Issues</h2>
        <p className="text-muted-foreground">Here's the list of your issues.</p>
      </div>
      <DataTable columns={columns} />
    </div>
  )
}
