export type AuthTokens = {
  access_token: string;
  refresh_token: string;
};

export class AuthApiError extends Error {
  constructor(
    message: string,
    public readonly status: number,
  ) {
    super(message);
    this.name = "AuthApiError";
  }
}

function getApiBaseUrl() {
  const apiBaseUrl = process.env.NEXT_PUBLIC_API_URL;

  if (!apiBaseUrl) {
    throw new Error("NEXT_PUBLIC_API_URL is not configured.");
  }

  return apiBaseUrl.replace(/\/$/, "");
}

async function requestAuthTokens(
  path: string,
  body: Record<string, string>,
  unauthorizedMessage: string,
) {
  const response = await fetch(`${getApiBaseUrl()}${path}`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
    },
    body: JSON.stringify(body),
    cache: "no-store",
  });

  if (!response.ok) {
    throw new AuthApiError(
      response.status === 401 ? unauthorizedMessage : "Auth request failed.",
      response.status,
    );
  }

  const data = (await response.json()) as Partial<AuthTokens>;

  if (
    typeof data.access_token !== "string" ||
    typeof data.refresh_token !== "string"
  ) {
    throw new Error("Auth response is invalid.");
  }

  return {
    access_token: data.access_token,
    refresh_token: data.refresh_token,
  } satisfies AuthTokens;
}

export async function loginWithApi(email: string, password: string) {
  return requestAuthTokens(
    "/login",
    {
      email,
      password,
    },
    "Invalid credentials.",
  );
}

export async function refreshWithApi(refreshToken: string) {
  return requestAuthTokens(
    "/refresh",
    {
      refresh_token: refreshToken,
    },
    "Refresh token is invalid.",
  );
}
