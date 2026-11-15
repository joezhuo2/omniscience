import { create } from 'zustand';

const KEY = 'omniscience.islandImage';
export const MAX_IMAGE_BYTES = 3 * 1024 * 1024;

function read(): string | null {
  try {
    return localStorage.getItem(KEY);
  } catch {
    return null;
  }
}

export const useIslandImage = create<{
  image: string | null;
  setImage: (image: string | null) => boolean;
}>((set) => ({
  image: read(),
  setImage: (image) => {
    try {
      if (image) localStorage.setItem(KEY, image);
      else localStorage.removeItem(KEY);
    } catch {
      return false;
    }
    set({ image });
    return true;
  },
}));
