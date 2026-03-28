export function projectKeyFromIssueKey(issueKey: string): string {
  const i = issueKey.lastIndexOf("-")
  return i >= 0 ? issueKey.slice(0, i) : issueKey
}

export type IssueDetailSource =
  | { from: "my-issues" }
  | { from: "project"; projectKey?: string }

export function issueDetailHref(
  orgSlug: string,
  issueKey: string,
  source: IssueDetailSource
): string {
  const path = `/${orgSlug}/issue/${encodeURIComponent(issueKey)}`
  const q = new URLSearchParams()
  if (source.from === "my-issues") {
    q.set("from", "my-issues")
    return `${path}?${q}`
  }
  q.set("from", "project")
  q.set(
    "project",
    source.projectKey ?? projectKeyFromIssueKey(issueKey)
  )
  return `${path}?${q}`
}

export function issueDetailHrefPreserveSource(
  orgSlug: string,
  issueKey: string,
  current: Pick<URLSearchParams, "get">
): string {
  const from = current.get("from")
  const project = current.get("project")
  if (from === "my-issues") {
    return issueDetailHref(orgSlug, issueKey, { from: "my-issues" })
  }
  if (from === "project") {
    return issueDetailHref(orgSlug, issueKey, {
      from: "project",
      projectKey: project ?? undefined,
    })
  }
  return `/${orgSlug}/issue/${encodeURIComponent(issueKey)}`
}
