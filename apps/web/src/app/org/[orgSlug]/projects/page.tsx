import { DataTable } from "./components/data-table"
import { columns } from "./components/columns"

export default function ProjectsPage() {
  return (
    <section className="flex min-h-0 flex-1 flex-col gap-4">
      <h1 className="text-2xl font-semibold">Projects</h1>
      <DataTable columns={columns} />
    </section>
  )
}
