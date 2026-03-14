import { NextResponse } from "next/server"

import { AuthApiError, signupWithApi } from "@/lib/auth/backend"
import { clearAuthCookies, setAuthCookies } from "@/lib/auth/cookies"

type SignupBody = {
  email?: string
  name?: string
  password?: string
}

export async function POST(request: Request) {
  const body = (await request.json().catch(() => null)) as SignupBody | null

  if (!body?.email || !body?.name || !body?.password) {
    return NextResponse.json(
      { message: "Email, ad ve şifre zorunlu." },
      { status: 400 },
    )
  }

  if (body.password.length < 8) {
    return NextResponse.json(
      { message: "Şifre en az 8 karakter olmalı." },
      { status: 400 },
    )
  }

  try {
    const tokens = await signupWithApi(
      body.email.trim(),
      body.name.trim(),
      body.password,
    )
    const response = NextResponse.json({ ok: true })
    setAuthCookies(response, tokens)
    return response
  } catch (error) {
    const status = error instanceof AuthApiError ? error.status : 500
    const message =
      error instanceof AuthApiError && error.status === 409
        ? "Bu e-posta adresi zaten kayıtlı."
        : status === 409
          ? "Bu e-posta adresi zaten kayıtlı."
          : "Kayıt sırasında bir hata oluştu."

    const response = NextResponse.json(
      { message },
      { status },
    )
    if (status === 401 || status === 409) {
      clearAuthCookies(response)
    }
    return response
  }
}
