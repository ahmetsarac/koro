import { redirect } from "next/navigation"

export default async function MyIssuesPage({
  params,
}: {
  params: Promise<{ orgSlug: string }>
}) {
  const { orgSlug } = await params
  redirect(`/${orgSlug}/my-issues/assigned`)
}
