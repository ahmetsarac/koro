import type { NextResponse } from "next/server";

import type { AuthTokens } from "@/lib/auth/backend";
import {
  ACCESS_TOKEN_COOKIE_NAME,
  ACCESS_TOKEN_MAX_AGE_SECONDS,
  REFRESH_TOKEN_COOKIE_NAME,
  REFRESH_TOKEN_MAX_AGE_SECONDS,
} from "@/lib/auth/constants";

const secure = process.env.NODE_ENV === "production";

export { ACCESS_TOKEN_COOKIE_NAME, REFRESH_TOKEN_COOKIE_NAME };

export function setAuthCookies(response: NextResponse, tokens: AuthTokens) {
  response.cookies.set({
    name: ACCESS_TOKEN_COOKIE_NAME,
    value: tokens.access_token,
    httpOnly: true,
    sameSite: "lax",
    secure,
    path: "/",
    maxAge: ACCESS_TOKEN_MAX_AGE_SECONDS,
  });

  response.cookies.set({
    name: REFRESH_TOKEN_COOKIE_NAME,
    value: tokens.refresh_token,
    httpOnly: true,
    sameSite: "lax",
    secure,
    path: "/",
    maxAge: REFRESH_TOKEN_MAX_AGE_SECONDS,
  });
}

export function clearAuthCookies(response: NextResponse) {
  response.cookies.set({
    name: ACCESS_TOKEN_COOKIE_NAME,
    value: "",
    httpOnly: true,
    sameSite: "lax",
    secure,
    path: "/",
    maxAge: 0,
  });

  response.cookies.set({
    name: REFRESH_TOKEN_COOKIE_NAME,
    value: "",
    httpOnly: true,
    sameSite: "lax",
    secure,
    path: "/",
    maxAge: 0,
  });
}
