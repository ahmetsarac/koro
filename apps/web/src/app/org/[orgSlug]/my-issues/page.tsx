import { redirect } from "next/navigation"

export default async function MyIssuesPage({
  params,
}: {
  params: Promise<{ orgSlug: string }>
}) {
  const { orgSlug } = await params
  redirect(`/org/${orgSlug}/my-issues/assigned`)
}
