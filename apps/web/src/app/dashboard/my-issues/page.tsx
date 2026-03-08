import { promises as fs } from "fs"
import path from "path"
import { type Metadata } from "next"
import Image from "next/image"
import { z } from "zod"

import { columns } from "./components/columns"
import { DataTable } from "./components/data-table"
import { UserNav } from "./components/user-nav"
import { taskSchema } from "./data/schema"

export const metadata: Metadata = {
  title: "Tasks",
  description: "A task and issue tracker build using Tanstack Table.",
}

// Simulate a database read for tasks.
async function getTasks() {
  const data = await fs.readFile(
    path.join(process.cwd(), "src/app/dashboard/my-issues/data/tasks.json")
  )

  const tasks = JSON.parse(data.toString())

  return z.array(taskSchema).parse(tasks)
}

export default async function TaskPage() {
  const tasks = await getTasks()

  return (
    <div className="flex h-[calc(100svh-4.5rem)] min-h-0 flex-col">
      <div className="mb-4">
        <h2 className="text-2xl font-semibold tracking-tight">My Issues</h2>
        <p className="text-muted-foreground">Here's the list of your issues.</p>
      </div>
      <DataTable data={tasks} columns={columns} />
    </div>
  )
}
