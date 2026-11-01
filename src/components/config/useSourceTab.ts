import { useState } from 'react';

export function useSourceTab(activeAc: boolean | null | undefined) {
  const [chosen, setChosen] = useState<boolean | null>(null);
  return [chosen ?? activeAc ?? true, setChosen] as const;
}
