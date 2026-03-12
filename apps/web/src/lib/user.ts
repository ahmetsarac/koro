import { z } from "zod"

export const userOrganizationSchema = z.object({
  id: z.string().uuid(),
  name: z.string(),
  slug: z.string(),
  role: z.string(),
})

export const meResponseSchema = z.object({
  id: z.string().uuid(),
  email: z.string(),
  name: z.string(),
  organizations: z.array(userOrganizationSchema),
})

export type UserOrganization = z.infer<typeof userOrganizationSchema>
export type MeResponse = z.infer<typeof meResponseSchema>

export async function fetchMe(): Promise<MeResponse> {
  const response = await fetch("/api/me", {
    cache: "no-store",
    credentials: "same-origin",
  })

  if (!response.ok) {
    throw new Error("Failed to fetch user info")
  }

  const data = await response.json()
  return meResponseSchema.parse(data)
}
