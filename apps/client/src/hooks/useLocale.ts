import '~i18n/config';
import { useTranslation } from 'react-i18next';
import { useStore } from '~store/store';
import shallow from 'zustand/shallow';
import { Locale } from '~util/enums';

export function useLocale() {
	const userPreferences = useStore((state) => state.userPreferences, shallow);
	const setLocale = useStore((state) => state.setLocale);

	function setLocaleFromStr(localeStr: string) {
		let locale = localeStr as Locale;

		if (locale) {
			setLocale(locale);
		}
	}

	const locale: string = userPreferences?.locale || 'en';

	const { t } = useTranslation(locale);

	const locales = Object.keys(Locale)
		.map((key) => ({ label: key, value: Locale[key as keyof typeof Locale] }))
		.filter((option) => typeof option.value === 'string');

	return { locale, setLocale, setLocaleFromStr, t, locales };
}
