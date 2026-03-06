import type { NextRequest } from "next/server";
import { NextResponse } from "next/server";

import { AuthApiError, refreshWithApi } from "@/lib/auth/backend";
import { DASHBOARD_HOME } from "@/lib/auth/constants";
import {
  ACCESS_TOKEN_COOKIE_NAME,
  REFRESH_TOKEN_COOKIE_NAME,
  clearAuthCookies,
  setAuthCookies,
} from "@/lib/auth/cookies";
import { verifyJwt } from "@/lib/auth/jwt";

function getJwtSecret() {
  console.log(process.env);
  const secret = process.env.JWT_SECRET;

  if (!secret) {
    throw new Error("JWT_SECRET is not configured.");
  }

  return secret;
}

function buildLoginUrl(request: NextRequest) {
  const loginUrl = new URL("/login", request.url);
  const nextPath = `${request.nextUrl.pathname}${request.nextUrl.search}`;

  if (nextPath !== DASHBOARD_HOME) {
    loginUrl.searchParams.set("next", nextPath);
  }

  return loginUrl;
}

export async function proxy(request: NextRequest) {
  const pathname = request.nextUrl.pathname;
  const accessToken = request.cookies.get(ACCESS_TOKEN_COOKIE_NAME)?.value;
  const refreshToken = request.cookies.get(REFRESH_TOKEN_COOKIE_NAME)?.value;
  const secret = getJwtSecret();

  const accessClaims = accessToken
    ? await verifyJwt(accessToken, secret, "access")
    : null;

  if (pathname === "/login") {
    if (accessClaims) {
      return NextResponse.redirect(new URL(DASHBOARD_HOME, request.url));
    }

    if (refreshToken) {
      const refreshClaims = await verifyJwt(refreshToken, secret, "refresh");

      if (refreshClaims) {
        try {
          const tokens = await refreshWithApi(refreshToken);
          const response = NextResponse.redirect(
            new URL(DASHBOARD_HOME, request.url),
          );

          setAuthCookies(response, tokens);

          return response;
        } catch (error) {
          if (error instanceof AuthApiError && error.status === 401) {
            const response = NextResponse.next();
            clearAuthCookies(response);
            return response;
          }

          throw error;
        }
      }
    }

    return NextResponse.next();
  }

  if (accessClaims) {
    return NextResponse.next();
  }

  if (refreshToken) {
    const refreshClaims = await verifyJwt(refreshToken, secret, "refresh");

    if (refreshClaims) {
      try {
        const tokens = await refreshWithApi(refreshToken);
        const response = NextResponse.next();

        setAuthCookies(response, tokens);

        return response;
      } catch (error) {
        if (!(error instanceof AuthApiError) || error.status !== 401) {
          throw error;
        }
      }
    }
  }

  const response = NextResponse.redirect(buildLoginUrl(request));
  clearAuthCookies(response);

  return response;
}

export const config = {
  matcher: ["/login", "/dashboard", "/dashboard/:path*"],
};
