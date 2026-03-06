import { NextResponse } from "next/server";

import { AuthApiError, loginWithApi } from "@/lib/auth/backend";
import { clearAuthCookies, setAuthCookies } from "@/lib/auth/cookies";

type LoginBody = {
  email?: string;
  password?: string;
};

export async function POST(request: Request) {
  const body = (await request.json().catch(() => null)) as LoginBody | null;

  if (!body?.email || !body?.password) {
    return NextResponse.json(
      { message: "Email ve password zorunlu." },
      { status: 400 },
    );
  }

  try {
    const tokens = await loginWithApi(body.email, body.password);
    const response = NextResponse.json({ ok: true });

    setAuthCookies(response, tokens);

    return response;
  } catch (error) {
    const response = NextResponse.json(
      {
        message:
          error instanceof AuthApiError && error.status === 401
            ? "Email veya password hatalı."
            : "Giriş sırasında bir hata oluştu.",
      },
      {
        status: error instanceof AuthApiError ? error.status : 500,
      },
    );

    clearAuthCookies(response);

    return response;
  }
}
