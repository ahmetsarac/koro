export type TokenType = "access" | "refresh";

export type TokenClaims = {
  sub: string;
  exp: number;
  token_type: TokenType;
};

const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder();

function decodeBase64Url(value: string) {
  const base64 = value.replace(/-/g, "+").replace(/_/g, "/");
  const padding = "=".repeat((4 - (base64.length % 4)) % 4);
  const binary = atob(`${base64}${padding}`);
  const bytes = new Uint8Array(binary.length);

  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }

  return bytes;
}

function parseSegment<T>(segment: string) {
  const json = textDecoder.decode(decodeBase64Url(segment));
  return JSON.parse(json) as T;
}

async function importSecret(secret: string) {
  return crypto.subtle.importKey(
    "raw",
    textEncoder.encode(secret),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["verify"],
  );
}

function isValidClaims(value: unknown): value is TokenClaims {
  if (!value || typeof value !== "object") {
    return false;
  }

  const claims = value as Partial<TokenClaims>;

  return (
    typeof claims.sub === "string" &&
    typeof claims.exp === "number" &&
    (claims.token_type === "access" || claims.token_type === "refresh")
  );
}

export async function verifyJwt(
  token: string,
  secret: string,
  expectedType: TokenType,
) {
  try {
    const [headerSegment, payloadSegment, signatureSegment] = token.split(".");

    if (!headerSegment || !payloadSegment || !signatureSegment) {
      return null;
    }

    const header = parseSegment<{ alg?: string; typ?: string }>(headerSegment);

    if (header.alg !== "HS256" || header.typ !== "JWT") {
      return null;
    }

    const key = await importSecret(secret);
    const isValidSignature = await crypto.subtle.verify(
      "HMAC",
      key,
      decodeBase64Url(signatureSegment),
      textEncoder.encode(`${headerSegment}.${payloadSegment}`),
    );

    if (!isValidSignature) {
      return null;
    }

    const claims = parseSegment<unknown>(payloadSegment);

    if (!isValidClaims(claims)) {
      return null;
    }

    if (claims.token_type !== expectedType) {
      return null;
    }

    if (claims.exp <= Math.floor(Date.now() / 1000)) {
      return null;
    }

    return claims;
  } catch {
    return null;
  }
}
