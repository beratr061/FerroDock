function pageNonce(): string | undefined {
  const meta = document.querySelector<HTMLMetaElement>('meta[property="csp-nonce"]');
  return meta?.nonce || undefined;
}

export function withStyleNonce<T>(create: () => T): T {
  const nonce = pageNonce();
  if (!nonce) {
    return create();
  }
  const original = document.createElement;
  document.createElement = function createWithNonce(this: Document, tag: string, options?: ElementCreationOptions) {
    const element = original.call(this, tag, options);
    if (tag.toLowerCase() === "style") {
      element.nonce = nonce;
    }
    return element;
  } as typeof document.createElement;
  try {
    return create();
  } finally {
    document.createElement = original;
  }
}
