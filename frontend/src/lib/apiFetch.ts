export class UnauthorizedError extends Error {
  constructor() {
    super("Unauthorized");
    this.name = "UnauthorizedError";
  }
}

function buildHeaders(creds: string, extra?: HeadersInit): HeadersInit {
  return {
    Authorization: `Basic ${creds}`,
    "Content-Type": "application/json",
    ...(extra as Record<string, string>),
  };
}

async function attemptFetch(
  path: string,
  creds: string,
  options?: RequestInit
): Promise<Response> {
  const res = await fetch(path, {
    ...options,
    headers: buildHeaders(creds, options?.headers),
  });
  if (res.status === 401) throw new UnauthorizedError();
  return res;
}

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export async function apiFetch(
  path: string,
  creds: string,
  options?: RequestInit
): Promise<Response> {
  try {
    const res = await attemptFetch(path, creds, options);
    if (res.ok) return res;
    // non-401, non-ok: retry once
    await delay(1000);
    const retry = await attemptFetch(path, creds, options);
    if (retry.ok) return retry;
    throw new Error(`HTTP ${retry.status}`);
  } catch (err) {
    if (err instanceof UnauthorizedError) throw err;
    // network error path: retry once
    await delay(1000);
    const retry = await attemptFetch(path, creds, options);
    if (retry.ok) return retry;
    throw err;
  }
}
