import { createContext, useContext } from 'react';

export type Toggle = { enabled: boolean; setEnabled: (enabled: boolean) => void };

export const SectionToggleContext = createContext<Toggle | null>(null);

export const useSectionToggle = () => useContext(SectionToggleContext);
