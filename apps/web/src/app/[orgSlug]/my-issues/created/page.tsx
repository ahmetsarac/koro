"use client"

import { use } from "react"

import { createColumns } from "../components/columns"
import { DataTable } from "../components/data-table"

export default function CreatedIssuesPage({
  params,
}: {
  params: Promise<{ orgSlug: string }>
}) {
  const { orgSlug } = use(params)
  const columns = createColumns(orgSlug)

  return <DataTable orgSlug={orgSlug} columns={columns} filterType="created" />
}
