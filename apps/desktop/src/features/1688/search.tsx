/**
 * 1688 商品搜索页 — 关键词搜索 + 结果列表。
 *
 * 搜索栏使用共享 {@link ChannelSearchToolbar}（无 PageGlowCard 光晕）；
 * 结果列表用普通边框卡片，光晕仅留给大块内容卡片场景。
 *
 * @author Xiaoman
 * @created 2026-08-22
 */

import { useEffect, useMemo, useState } from "react";
import {
  Loading,
  PageScaffold,
  toast,
} from "@desk/ui";
import { ExternalLink } from "@desk/ui/icons";
import { accountList, type XianyuAccount } from "@desk/platform/ipc/account";
import {
  ali1688Search,
  type Ali1688SearchOffer,
  type Ali1688SearchResponse,
} from "@desk/platform/ipc/ali1688-search";
import { ChannelSearchToolbar } from "@feature/component/channel-search";

const OWNER_ID = 1;

function formatPrice(offer: Ali1688SearchOffer): string {
  const text = offer.price?.text?.trim();
  if (text) return text;
  const min = offer.price?.min;
  if (typeof min === "number") return `¥${min}`;
  return "—";
}

function formatLocation(offer: Ali1688SearchOffer): string {
  const province = offer.location?.province?.trim();
  const city = offer.location?.city?.trim();
  if (province && city) return `${province} · ${city}`;
  return province || city || "—";
}

function toAccountOptions(accounts: XianyuAccount[]) {
  return accounts.map((account) => ({
    id: account.account_id,
    label: account.display_name || account.login_id || account.account_id,
  }));
}

/**
 * 1688 商品搜索页。
 */
export function Ali1688SearchPage() {
  const [accounts, setAccounts] = useState<XianyuAccount[]>([]);
  const [accountId, setAccountId] = useState("");
  const [keyword, setKeyword] = useState("");
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<Ali1688SearchResponse | null>(null);

  const accountOptions = useMemo(() => toAccountOptions(accounts), [accounts]);

  useEffect(() => {
    let cancelled = false;
    void accountList(OWNER_ID)
      .then((list) => {
        if (cancelled) return;
        const aliAccounts = list.filter(
          (item) => (item.platform ?? "xianyu") === "ali1688" && item.status === "active",
        );
        setAccounts(aliAccounts);
        if (aliAccounts.length > 0) {
          setAccountId((current) => current || aliAccounts[0]!.account_id);
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
      toast.error("请先添加并选择 1688 账号");
      return;
    }
    setLoading(true);
    setResult(null);
    try {
      const data = await ali1688Search({
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
      title="商品搜索"
      subtitle="使用 Camoufox 指纹浏览器在 1688 搜索批发商品（会弹出浏览器窗口）"
      toolbar={
        <ChannelSearchToolbar
          accountLabel="1688 账号"
          accounts={accountOptions}
          accountId={accountId}
          onAccountIdChange={setAccountId}
          keyword={keyword}
          onKeywordChange={setKeyword}
          onSearch={() => void handleSearch()}
          loading={loading}
          keywordPlaceholder="例如：苹果17pro"
        />
      }
    >
      {accounts.length === 0 ? (
        <p className="text-sm text-muted-foreground">请先在「账号管理」扫码登录 1688 账号。</p>
      ) : null}

      {loading ? (
        <div className="flex justify-center py-12">
          <Loading text="正在启动指纹浏览器并搜索…" />
        </div>
      ) : null}

      {result && result.offers.length > 0 ? (
        <div className="space-y-3">
          <p className="text-sm text-muted-foreground">
            「{result.keyword}」共 {result.totalBeforeFilter} 条命中，展示 {result.total} 条
          </p>
          <ul className="space-y-3">
            {result.offers.map((offer) => (
              <li key={offer.offerId}>
                <article className="flex gap-3 rounded-xl border border-border bg-card p-3 sm:gap-4 sm:p-4">
                  {offer.image ? (
                    <img
                      src={offer.image}
                      alt=""
                      className="size-20 shrink-0 rounded-md bg-muted object-cover sm:size-24"
                      loading="lazy"
                    />
                  ) : (
                    <div className="size-20 shrink-0 rounded-md bg-muted sm:size-24" />
                  )}
                  <div className="min-w-0 flex-1 space-y-1.5">
                    <a
                      href={offer.url}
                      target="_blank"
                      rel="noreferrer"
                      className="line-clamp-2 text-sm font-medium leading-snug hover:underline"
                    >
                      {offer.title}
                    </a>
                    <div className="flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-muted-foreground">
                      <span className="text-base font-semibold text-foreground">
                        {formatPrice(offer)}
                      </span>
                      <span>{offer.supplier?.name ?? "—"}</span>
                      <span>{formatLocation(offer)}</span>
                      {offer.turnover ? <span>成交 {offer.turnover}</span> : null}
                    </div>
                    {offer.tags && offer.tags.length > 0 ? (
                      <div className="flex flex-wrap gap-1">
                        {offer.tags.slice(0, 4).map((tag) => (
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
                    href={offer.url}
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
