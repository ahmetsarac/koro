import { DataTable } from "./components/data-table"
import { columns } from "./components/columns"

export default async function ProjectsPage({
  params,
}: {
  params: Promise<{ orgSlug: string }>
}) {
  const { orgSlug } = await params

  return (
    <section className="flex h-[calc(100svh-4.5rem)] flex-col gap-4">
      <h1 className="text-2xl font-semibold">Projects</h1>
      <DataTable columns={columns} orgSlug={orgSlug} />
    </section>
  )
}
