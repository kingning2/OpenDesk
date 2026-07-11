export type Locale = "zh-CN" | "en";

export type TranslationTree = {
  readonly [key: string]: string | TranslationTree;
};

export type Messages = TranslationTree;
