import { apiFetch } from "./client";

export interface DocEntry {
  slug: string;
  title: string;
}

export interface DocContent {
  slug: string;
  title: string;
  content: string;
}

export function fetchDocs(): Promise<DocEntry[]> {
  return apiFetch<DocEntry[]>("/api/docs");
}

export function fetchDoc(slug: string): Promise<DocContent> {
  return apiFetch<DocContent>(`/api/docs/${slug}`);
}
