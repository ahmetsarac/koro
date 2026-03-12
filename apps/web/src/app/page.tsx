import { redirect } from "next/navigation";

export default function Home() {
  redirect("/org/acme/my-issues");
}
