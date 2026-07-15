import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import en from './locales/en.json';
import zh from './locales/zh.json';

const STORAGE_KEY = 'tauri-template-app-lang';

function getStoredLang(): string {
  if (typeof localStorage === 'undefined') return 'zh';
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === 'zh' || stored === 'en') return stored;
  // Fallback to system language
  if (typeof navigator !== 'undefined') {
    const sysLang = navigator.language.toLowerCase();
    if (sysLang.startsWith('zh')) return 'zh';
  }
  return 'zh';
}

function setStoredLang(lang: string) {
  try {
    localStorage.setItem(STORAGE_KEY, lang);
  } catch {
    // ignore
  }
}

i18n
  .use(initReactI18next)
  .init({
    resources: {
      zh: { translation: zh },
      en: { translation: en },
    },
    lng: getStoredLang(),
    fallbackLng: 'en',
    interpolation: {
      escapeValue: false, // React already escapes
    },
  });

export function getLocaleName(lang: string): string {
  return lang === 'zh' ? '中文' : 'English';
}

export function toggleLang(): string {
  const next = i18n.language === 'zh' ? 'en' : 'zh';
  i18n.changeLanguage(next);
  setStoredLang(next);
  return next;
}

export default i18n;
