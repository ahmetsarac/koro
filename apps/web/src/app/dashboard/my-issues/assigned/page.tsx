import { type Metadata } from "next"

import { columns } from "../components/columns"
import { DataTable } from "../components/data-table"

export const metadata: Metadata = {
  title: "Assigned Issues",
  description: "Issues assigned to you.",
}

export default function AssignedIssuesPage() {
  return <DataTable columns={columns} filterType="assigned" />
}
