import type { NextRequest } from "next/server";
import { NextResponse } from "next/server";

import { AuthApiError, refreshWithApi } from "@/lib/auth/backend";
import {
  REFRESH_TOKEN_COOKIE_NAME,
  clearAuthCookies,
  setAuthCookies,
} from "@/lib/auth/cookies";

export async function POST(request: NextRequest) {
  const refreshToken = request.cookies.get(REFRESH_TOKEN_COOKIE_NAME)?.value;

  if (!refreshToken) {
    return NextResponse.json(
      { message: "Refresh token bulunamadı." },
      { status: 401 },
    );
  }

  try {
    const tokens = await refreshWithApi(refreshToken);
    const response = NextResponse.json({ ok: true });

    setAuthCookies(response, tokens);

    return response;
  } catch (error) {
    const response = NextResponse.json(
      {
        message:
          error instanceof AuthApiError && error.status === 401
            ? "Oturum süresi doldu."
            : "Refresh sırasında bir hata oluştu.",
      },
      {
        status: error instanceof AuthApiError ? error.status : 500,
      },
    );

    clearAuthCookies(response);

    return response;
  }
}
