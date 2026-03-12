import { type Metadata } from "next"

import { columns } from "../components/columns"
import { DataTable } from "../components/data-table"

export const metadata: Metadata = {
  title: "Created Issues",
  description: "Issues created by you.",
}

export default function CreatedIssuesPage() {
  return <DataTable columns={columns} filterType="created" />
}
