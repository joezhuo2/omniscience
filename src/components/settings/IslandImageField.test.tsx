import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, expect, it } from 'vitest';
import { MAX_IMAGE_BYTES, useIslandImage } from '../../lib/islandImage';
import { IslandImageField } from './IslandImageField';

beforeEach(() => useIslandImage.setState({ image: null }));

const pick = (file: File) =>
  fireEvent.change(screen.getByTestId('island-image-input'), { target: { files: [file] } });

it('defaults to no image, saves a chosen image and removes it', async () => {
  render(<IslandImageField />);
  expect(screen.getByRole('button', { name: 'Choose island image' })).toBeInTheDocument();
  pick(new File(['png'], 'bg.png', { type: 'image/png' }));
  await waitFor(() => expect(useIslandImage.getState().image).toMatch(/^data:image\/png/));
  expect(localStorage.getItem('omniscience.islandImage')).toMatch(/^data:image\/png/);
  fireEvent.click(screen.getByRole('button', { name: 'Remove image' }));
  expect(useIslandImage.getState().image).toBeNull();
  expect(localStorage.getItem('omniscience.islandImage')).toBeNull();
});

it('rejects non-images and oversized files', () => {
  render(<IslandImageField />);
  pick(new File(['x'], 'notes.txt', { type: 'text/plain' }));
  expect(screen.getByRole('alert')).toHaveTextContent('Choose an image file.');
  pick(new File([new Uint8Array(MAX_IMAGE_BYTES + 1)], 'big.png', { type: 'image/png' }));
  expect(screen.getByRole('alert')).toHaveTextContent('3 MB or smaller');
  expect(useIslandImage.getState().image).toBeNull();
});
