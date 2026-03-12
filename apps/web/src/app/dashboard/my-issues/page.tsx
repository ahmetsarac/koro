import { redirect } from "next/navigation"

export default function MyIssuesPage() {
  redirect("/dashboard/my-issues/assigned")
}
