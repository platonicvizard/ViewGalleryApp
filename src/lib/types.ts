export type MediaKind = "image" | "video";

export interface MediaEntry {
  path: string;
  name: string;
  kind: MediaKind;
  ext: string;
  size: number;
  modified: number;
}

export type KindFilter = "all" | MediaKind;

export type SortKey = "name" | "date" | "size";
export type SortDir = "asc" | "desc";
