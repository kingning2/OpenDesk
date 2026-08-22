/**
 * 闲鱼商品搜索页 — 关键词搜索 + 结果列表。
 *
 * 搜索栏使用共享 {@link ChannelSearchToolbar}；Sidecar 拦截
 * `mtop.taobao.idlemtopsearch.pc.search`（对齐 ai-goofish-monitor）。
 *
 * @author Xiaoman
 * @created 2026-08-22
 */

import { useEffect, useMemo, useState } from "react";
import { Loading, PageScaffold, toast } from "@desk/ui";
import { ExternalLink } from "@desk/ui/icons";
import { accountList, type XianyuAccount } from "@desk/platform/ipc/account";
import {
  xianyuSearch,
  type XianyuSearchItem,
  type XianyuSearchResponse,
} from "@desk/platform/ipc/xianyu-search";
import { ChannelSearchToolbar } from "@feature/component/channel-search";

const OWNER_ID = 1;

function formatPrice(item: XianyuSearchItem): string {
  const text = item.price?.text?.trim();
  if (text) return text;
  return "—";
}

function toAccountOptions(accounts: XianyuAccount[]) {
  return accounts.map((account) => ({
    id: account.account_id,
    label: account.display_name || account.login_id || account.account_id,
  }));
}

/**
 * 闲鱼商品搜索页。
 */
export function XianyuSearchPage() {
  const [accounts, setAccounts] = useState<XianyuAccount[]>([]);
  const [accountId, setAccountId] = useState("");
  const [keyword, setKeyword] = useState("");
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<XianyuSearchResponse | null>(null);

  const accountOptions = useMemo(() => toAccountOptions(accounts), [accounts]);

  useEffect(() => {
    let cancelled = false;
    void accountList(OWNER_ID)
      .then((list) => {
        if (cancelled) return;
        const xianyuAccounts = list.filter(
          (item) => (item.platform ?? "xianyu") === "xianyu" && item.status === "active",
        );
        setAccounts(xianyuAccounts);
        if (xianyuAccounts.length > 0) {
          setAccountId((current) => current || xianyuAccounts[0]!.account_id);
        }
      })
      .catch((error) => {
        if (!cancelled) {
          toast.error(error instanceof Error ? error.message : String(error));
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  async function handleSearch() {
    const kw = keyword.trim();
    if (!kw) {
      toast.error("请输入搜索关键词");
      return;
    }
    if (!accountId) {
      toast.error("请先添加并选择闲鱼账号");
      return;
    }
    setLoading(true);
    setResult(null);
    try {
      const data = await xianyuSearch({
        ownerId: OWNER_ID,
        accountId,
        keyword: kw,
        maxResults: 20,
        headed: true,
      });
      setResult(data);
      if (!data.ok) {
        toast.error(data.detail || "搜索未返回结果");
      } else {
        toast.success(`找到 ${data.total} 条结果`);
      }
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setLoading(false);
    }
  }

  return (
    <PageScaffold
      title="闲鱼商品搜索"
      subtitle="使用指纹浏览器在闲鱼搜索二手商品（会弹出浏览器窗口）"
      toolbar={
        <ChannelSearchToolbar
          accountLabel="闲鱼账号"
          accounts={accountOptions}
          accountId={accountId}
          onAccountIdChange={setAccountId}
          keyword={keyword}
          onKeywordChange={setKeyword}
          onSearch={() => void handleSearch()}
          loading={loading}
          keywordPlaceholder="例如：iPhone 15"
          searchLabel="搜索"
        />
      }
    >
      {accounts.length === 0 ? (
        <p className="text-sm text-muted-foreground">请先在「账号管理」扫码登录闲鱼账号。</p>
      ) : null}

      {loading ? (
        <div className="flex justify-center py-12">
          <Loading text="正在启动浏览器并搜索…" />
        </div>
      ) : null}

      {result && result.offers.length > 0 ? (
        <div className="space-y-3">
          <p className="text-sm text-muted-foreground">
            「{result.keyword}」共 {result.totalBeforeFilter} 条命中，展示 {result.total} 条
          </p>
          <ul className="space-y-3">
            {result.offers.map((item) => (
              <li key={item.itemId}>
                <article className="flex gap-3 rounded-xl border border-border bg-card p-3 sm:gap-4 sm:p-4">
                  {item.image ? (
                    <img
                      src={item.image}
                      alt=""
                      className="size-20 shrink-0 rounded-md bg-muted object-cover sm:size-24"
                      loading="lazy"
                    />
                  ) : (
                    <div className="size-20 shrink-0 rounded-md bg-muted sm:size-24" />
                  )}
                  <div className="min-w-0 flex-1 space-y-1.5">
                    <a
                      href={item.url}
                      target="_blank"
                      rel="noreferrer"
                      className="line-clamp-2 text-sm font-medium leading-snug hover:underline"
                    >
                      {item.title}
                    </a>
                    <div className="flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-muted-foreground">
                      <span className="text-base font-semibold text-foreground">
                        {formatPrice(item)}
                      </span>
                      {item.seller?.name ? <span>{item.seller.name}</span> : null}
                      {item.location ? <span>{item.location}</span> : null}
                      {item.wantCount ? <span>{item.wantCount} 人想要</span> : null}
                      {item.publishedAt ? <span>{item.publishedAt}</span> : null}
                    </div>
                    {item.tags && item.tags.length > 0 ? (
                      <div className="flex flex-wrap gap-1">
                        {item.tags.slice(0, 4).map((tag) => (
                          <span
                            key={tag}
                            className="rounded-full bg-muted px-2 py-0.5 text-[10px] text-muted-foreground"
                          >
                            {tag}
                          </span>
                        ))}
                      </div>
                    ) : null}
                  </div>
                  <a
                    href={item.url}
                    target="_blank"
                    rel="noreferrer"
                    className="hidden shrink-0 self-start text-muted-foreground hover:text-foreground sm:block"
                    aria-label="在新窗口打开"
                  >
                    <ExternalLink className="size-4" />
                  </a>
                </article>
              </li>
            ))}
          </ul>
        </div>
      ) : null}

      {result && result.offers.length === 0 && !loading ? (
        <p className="text-sm text-muted-foreground">{result.detail || "暂无结果"}</p>
      ) : null}
    </PageScaffold>
  );
}
