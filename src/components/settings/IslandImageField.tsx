import { FolderOpen } from 'lucide-react';
import { useRef, useState, type DragEvent } from 'react';
import { MAX_IMAGE_BYTES, useIslandImage } from '../../lib/islandImage';

export function IslandImageField() {
  const { image, setImage } = useIslandImage();
  const input = useRef<HTMLInputElement>(null);
  const [over, setOver] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = (file: File | undefined) => {
    if (!file) return;
    if (!file.type.startsWith('image/')) return setError('Choose an image file.');
    if (file.size > MAX_IMAGE_BYTES) return setError('Image must be 3 MB or smaller.');
    const reader = new FileReader();
    reader.onload = () => {
      const ok = setImage(String(reader.result));
      setError(ok ? null : 'Could not save the image.');
    };
    reader.readAsDataURL(file);
  };

  const drop = (event: DragEvent) => {
    event.preventDefault();
    setOver(false);
    load(event.dataTransfer.files[0]);
  };

  return (
    <div className="flex w-36 shrink-0 flex-col items-end gap-1">
      <button
        type="button"
        aria-label={image ? 'Change island image' : 'Choose island image'}
        onClick={() => input.current?.click()}
        onDragOver={(event) => {
          event.preventDefault();
          setOver(true);
        }}
        onDragLeave={() => setOver(false)}
        onDrop={drop}
        className={`flex h-20 w-full items-center justify-center rounded-lg border border-dashed bg-cover bg-center text-island-muted transition-colors hover:border-island-muted hover:text-island-text ${over ? 'border-island-text text-island-text' : 'border-island-faint'}`}
        style={image ? { backgroundImage: `url(${image})` } : undefined}
      >
        {!image && <FolderOpen size={20} aria-hidden />}
      </button>
      <input
        ref={input}
        type="file"
        accept="image/*"
        hidden
        data-testid="island-image-input"
        onChange={(event) => {
          load(event.target.files?.[0]);
          event.target.value = '';
        }}
      />
      {image && (
        <button
          type="button"
          onClick={() => setImage(null)}
          className="text-[10px] text-island-muted hover:text-island-text"
        >
          Remove image
        </button>
      )}
      {error && (
        <p role="alert" className="text-[10px] text-island-crit">
          {error}
        </p>
      )}
    </div>
  );
}
