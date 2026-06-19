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

export type GroupKey = "none" | "date" | "type";

export interface DuplicateGroup {
  hash: string;
  paths: string[];
}

export interface TrashEntry {
  id: string;
  original_path: string;
  stored_path: string;
  deleted_at: number;
}

export interface HistoryEntry {
  folder: string;
  last_item: string | null;
  opened_at: number;
}
