import { Button } from "./button";

export interface PaginationProps {
  page: number;
  totalPages: number;
  onPageChange: (page: number) => void;
}

/** 通用分页控件（上一页 / 页码 / 下一页）；单页时返回 null。 */
export function Pagination({ page, totalPages, onPageChange }: PaginationProps) {
  if (totalPages <= 1) {
    return null;
  }
  return (
    <div className="flex items-center justify-end gap-2">
      <Button
        size="sm"
        variant="outline"
        disabled={page <= 1}
        onClick={() => onPageChange(Math.max(1, page - 1))}
      >
        上一页
      </Button>
      <span className="text-[length:var(--text-sm)] text-muted-foreground">
        {page} / {totalPages}
      </span>
      <Button
        size="sm"
        variant="outline"
        disabled={page >= totalPages}
        onClick={() => onPageChange(Math.min(totalPages, page + 1))}
      >
        下一页
      </Button>
    </div>
  );
}
