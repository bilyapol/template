import { useEffect, useState } from "react";
import { useParams, Link } from "react-router-dom";
import { fetchDoc, fetchDocs, type DocContent, type DocEntry } from "@/api/docs";
import { Spinner } from "@/components/ui/Spinner";
import Markdown from "react-markdown";

export function DocPage() {
  const { slug } = useParams<{ slug: string }>();
  const [doc, setDoc] = useState<DocContent | null>(null);
  const [docs, setDocs] = useState<DocEntry[]>([]);
  // Slug whose fetch has finished; loading until it matches the current slug.
  const [loadedSlug, setLoadedSlug] = useState<string | null>(null);
  const loading = loadedSlug !== slug;

  useEffect(() => {
    fetchDocs().then(setDocs);
  }, []);

  useEffect(() => {
    if (!slug) return;
    fetchDoc(slug)
      .then(setDoc)
      .catch(() => setDoc(null))
      .finally(() => setLoadedSlug(slug));
  }, [slug]);

  if (loading) {
    return (
      <div className="flex items-center justify-center pt-20">
        <Spinner className="w-8 h-8 text-accent" />
      </div>
    );
  }

  return (
    <div className="max-w-5xl mx-auto px-6 pt-10 pb-16 flex gap-8">
      {/* Sidebar */}
      <nav className="hidden md:block w-48 shrink-0 pt-2">
        <Link
          to="/"
          className="text-xs text-text-muted hover:text-text mb-4 block"
        >
          &larr; Home
        </Link>
        <h3 className="text-xs font-semibold text-text-muted uppercase tracking-wider mb-3">
          Docs
        </h3>
        <ul className="space-y-1">
          {docs.map((d) => (
            <li key={d.slug}>
              <Link
                to={`/docs/${d.slug}`}
                className={`block text-sm py-1 px-2 rounded ${
                  d.slug === slug
                    ? "bg-accent-dim text-accent font-medium"
                    : "text-text-secondary hover:text-text"
                }`}
              >
                {d.title}
              </Link>
            </li>
          ))}
        </ul>
      </nav>

      {/* Content */}
      <article className="flex-1 min-w-0 prose prose-invert prose-sm max-w-none">
        {doc ? (
          <Markdown
            components={{
              h1: ({ children }) => (
                <h1 className="text-2xl font-bold text-text mb-4 mt-0">{children}</h1>
              ),
              h2: ({ children }) => (
                <h2 className="text-lg font-semibold text-text mt-8 mb-3 pb-1 border-b border-border-subtle">
                  {children}
                </h2>
              ),
              h3: ({ children }) => (
                <h3 className="text-base font-semibold text-text mt-6 mb-2">{children}</h3>
              ),
              p: ({ children }) => (
                <p className="text-sm text-text-secondary leading-relaxed mb-3">{children}</p>
              ),
              li: ({ children }) => (
                <li className="text-sm text-text-secondary leading-relaxed">{children}</li>
              ),
              code: ({ className, children }) => {
                const isBlock = className?.includes("language-");
                if (isBlock) {
                  return (
                    <code className="block bg-bg border border-border-subtle rounded-lg p-4 text-xs font-mono text-text overflow-x-auto my-3">
                      {children}
                    </code>
                  );
                }
                return (
                  <code className="bg-surface px-1.5 py-0.5 rounded text-xs font-mono text-accent">
                    {children}
                  </code>
                );
              },
              pre: ({ children }) => <pre className="my-0 bg-transparent">{children}</pre>,
              table: ({ children }) => (
                <table className="w-full text-sm border-collapse my-4">{children}</table>
              ),
              th: ({ children }) => (
                <th className="text-left text-xs font-semibold text-text-muted uppercase tracking-wider py-2 px-3 border-b border-border">
                  {children}
                </th>
              ),
              td: ({ children }) => (
                <td className="text-sm text-text-secondary py-2 px-3 border-b border-border-subtle">
                  {children}
                </td>
              ),
              a: ({ href, children }) => (
                <a
                  href={href}
                  className="text-accent hover:text-accent-hover underline"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  {children}
                </a>
              ),
              strong: ({ children }) => (
                <strong className="text-text font-semibold">{children}</strong>
              ),
            }}
          >
            {doc.content}
          </Markdown>
        ) : (
          <p className="text-text-muted">Document not found.</p>
        )}
      </article>
    </div>
  );
}
