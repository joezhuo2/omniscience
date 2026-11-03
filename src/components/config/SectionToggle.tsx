import { useState, type ReactNode } from 'react';
import { readSectionEnabled, saveSectionEnabled, type ConfigSectionId } from '../../lib/sections';
import { SectionToggleContext } from './sectionToggleContext';

export function SectionToggleProvider({
  section,
  active,
  children,
}: {
  section: ConfigSectionId;
  active: boolean;
  children: (active: boolean) => ReactNode;
}) {
  const [enabled, set] = useState(() => readSectionEnabled(section));
  const setEnabled = (next: boolean) => {
    set(next);
    saveSectionEnabled(section, next);
  };
  return (
    <SectionToggleContext.Provider value={{ enabled, setEnabled }}>
      {children(active && enabled)}
    </SectionToggleContext.Provider>
  );
}
