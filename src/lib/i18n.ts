import { derived, writable } from "svelte/store";
import en from "./i18n/en";
import zh_CN from "./i18n/zh_CN";
import uk from "./i18n/uk";
import pl from "./i18n/pl";

export type Language = "en" | "zh-CN" | "uk" | "pl";

const STORAGE_KEY = "dp100-lab-language";

const messages: Record<Language, Record<string, string>> = {
  en: en as unknown as Record<string, string>,
  "zh-CN": zh_CN as unknown as Record<string, string>,
  uk: uk as unknown as Record<string, string>,
  pl: pl as unknown as Record<string, string>,
};

type MessageKey = keyof typeof en;

function initialLanguage(): Language {
  if (typeof localStorage === "undefined") return "en";
  const stored = localStorage.getItem(STORAGE_KEY);
  const valid: Language[] = ["en", "zh-CN", "uk", "pl"];
  return valid.includes(stored as Language) ? (stored as Language) : "en";
}

export const language = writable<Language>(initialLanguage());

language.subscribe((value) => {
  if (typeof localStorage !== "undefined") {
    localStorage.setItem(STORAGE_KEY, value);
  }
});

export const languages: { code: Language; label: string }[] = [
  { code: "en", label: "English" },
  { code: "uk", label: "Українська" },
  { code: "pl", label: "Polski" },
  { code: "zh-CN", label: "中文" },
];

export function setLanguage(value: Language) {
  language.set(value);
}

export const t = derived(language, ($language) => {
  return (key: MessageKey, params: Record<string, string | number> = {}) => {
    let text: string = messages[$language]?.[key as string] ?? messages.en[key as string] ?? (key as string);
    for (const [name, value] of Object.entries(params)) {
      text = text.replaceAll(`{${name}}`, String(value));
    }
    return text;
  };
});
