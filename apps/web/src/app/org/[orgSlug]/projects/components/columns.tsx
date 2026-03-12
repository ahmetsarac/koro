"use client"

import { type ColumnDef } from "@tanstack/react-table"
import Link from "next/link"
import { FolderKanban, Users, FileText } from "lucide-react"

import { projectRoles } from "../data/data"
import { type Project } from "../data/schema"
import { DataTableColumnHeader } from "./data-table-column-header"

export const columns: ColumnDef<Project>[] = [
  {
    accessorKey: "project_key",
    id: "key",
    size: 100,
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title="Key" />
    ),
    cell: ({ row }) => (
      <Link
        href={`/org/${row.original.org_slug}/projects/${row.original.project_key}`}
        className="flex items-center gap-2 font-mono text-xs hover:underline"
      >
        <FolderKanban className="size-4 text-muted-foreground" />
        {row.original.project_key}
      </Link>
    ),
    enableSorting: false,
    enableHiding: false,
  },
  {
    accessorKey: "name",
    size: 300,
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title="Name" />
    ),
    cell: ({ row }) => (
      <Link
        href={`/org/${row.original.org_slug}/projects/${row.original.project_key}`}
        className="max-w-[400px] truncate font-medium hover:underline"
      >
        {row.getValue("name")}
      </Link>
    ),
    enableSorting: false,
  },
  {
    accessorKey: "org_name",
    size: 180,
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title="Organization" />
    ),
    cell: ({ row }) => (
      <span className="text-muted-foreground">
        {row.getValue("org_name")}
      </span>
    ),
    enableSorting: false,
  },
  {
    accessorKey: "issue_count",
    size: 100,
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title="Issues" />
    ),
    cell: ({ row }) => (
      <div className="flex items-center gap-1.5 text-muted-foreground">
        <FileText className="size-3.5" />
        <span>{row.getValue("issue_count")}</span>
      </div>
    ),
    enableSorting: false,
  },
  {
    accessorKey: "member_count",
    size: 100,
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title="Members" />
    ),
    cell: ({ row }) => (
      <div className="flex items-center gap-1.5 text-muted-foreground">
        <Users className="size-3.5" />
        <span>{row.getValue("member_count")}</span>
      </div>
    ),
    enableSorting: false,
  },
  {
    accessorKey: "my_role",
    size: 140,
    header: ({ column }) => (
      <DataTableColumnHeader column={column} title="My Role" />
    ),
    cell: ({ row }) => {
      const role = projectRoles.find(
        (r) => r.value === row.getValue("my_role")
      )

      if (!role) {
        return <span className="text-muted-foreground">{row.getValue("my_role")}</span>
      }

      return (
        <div className="flex items-center gap-2">
          {role.icon && (
            <role.icon className="size-4 text-muted-foreground" />
          )}
          <span>{role.label}</span>
        </div>
      )
    },
    enableSorting: false,
  },
]
