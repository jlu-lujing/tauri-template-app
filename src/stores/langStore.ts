import { useTranslation } from 'react-i18next';
import i18nModule, { toggleLang as _toggleLang, getLocaleName } from '../i18n';

export const i18n = i18nModule;

export function toggleLang() {
  return _toggleLang();
}

export function useLang() {
  const { i18n: ctxI18n } = useTranslation();
  return {
    lang: ctxI18n.language as 'zh' | 'en',
    localeName: getLocaleName(ctxI18n.language),
    toggle: () => _toggleLang(),
  };
}
