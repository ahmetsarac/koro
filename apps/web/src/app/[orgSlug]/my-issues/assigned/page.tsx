"use client"

import { use } from "react"

import { createColumns } from "../components/columns"
import { DataTable } from "../components/data-table"

export default function AssignedIssuesPage({
  params,
}: {
  params: Promise<{ orgSlug: string }>
}) {
  const { orgSlug } = use(params)
  const columns = createColumns(orgSlug)
  return (
    <div className="flex min-h-0 flex-1 flex-col w-[calc(100svw-20rem)]">
      <DataTable orgSlug={orgSlug} columns={columns} filterType="assigned" />
    </div>
  )
}
