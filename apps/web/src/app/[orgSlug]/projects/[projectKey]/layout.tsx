import { RecordProjectView } from "@/components/record-project-view"

export default async function ProjectKeyLayout({
  children,
  params,
}: {
  children: React.ReactNode
  params: Promise<{ orgSlug: string; projectKey: string }>
}) {
  const { orgSlug, projectKey } = await params

  return (
    <>
      <RecordProjectView orgSlug={orgSlug} projectKey={projectKey} />
      {children}
    </>
  )
}
