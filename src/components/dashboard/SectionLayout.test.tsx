import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { Card } from './Card';
import { SectionLayout } from './SectionLayout';
import { DEFAULT_SECTION_ORDER, SECTION_LABELS, saveSectionOrder } from '../../lib/sections';

const layout = (hidden: 'battery'[] = []) =>
  render(
    <div>
      <SectionLayout hidden={hidden}>
        {(id) => (
          <Card title={SECTION_LABELS[id]} section={id}>
            <input aria-label={`${id} input`} />
          </Card>
        )}
      </SectionLayout>
    </div>,
  );
const order = () =>
  [...document.querySelectorAll('section[data-section]')].map((card) =>
    card.getAttribute('data-section'),
  );

afterEach(() => {
  vi.restoreAllMocks();
});

describe('section drag layout', () => {
  it('restores saved order and omits absent cards without forgetting their place', () => {
    saveSectionOrder(['processes', 'network', 'battery', 'cpu-cores', 'gpu-power']);
    const view = layout(['battery']);
    expect(order()).toEqual(['processes', 'network', 'cpu-cores', 'gpu-power']);
    expect(screen.queryByRole('button', { name: 'Reorder Battery' })).not.toBeInTheDocument();
    view.unmount();
    layout();
    expect(order()).toEqual(['processes', 'network', 'battery', 'cpu-cores', 'gpu-power']);
  });

  it('reorders with the keyboard and preserves mounted card state', async () => {
    vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockImplementation(function (
      this: HTMLElement,
    ) {
      const card = this.closest('section[data-section]');
      const top = card ? order().indexOf(card.getAttribute('data-section')) * 100 : 0;
      return {
        x: 0,
        y: top,
        top,
        left: 0,
        width: 300,
        height: 80,
        right: 300,
        bottom: top + 80,
        toJSON: () => ({}),
      };
    });
    const view = layout();
    fireEvent.change(screen.getByRole('textbox', { name: 'gpu-power input' }), {
      target: { value: 'keep me' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'GPU & Power' }));
    const handle = screen.getByRole('button', { name: 'Reorder GPU & Power' });
    handle.focus();
    fireEvent.keyDown(handle, { key: ' ', code: 'Space' });
    await waitFor(() => expect(handle).toHaveAttribute('aria-pressed', 'true'));
    await act(async () => {
      await new Promise((resolve) => setTimeout(resolve, 10));
    });
    fireEvent.keyDown(document, { key: 'ArrowDown', code: 'ArrowDown' });
    fireEvent.keyDown(document, { key: ' ', code: 'Space' });
    await waitFor(() => expect(order()[0]).toBe('battery'));
    expect(screen.getByRole('button', { name: 'GPU & Power' })).toHaveAttribute(
      'aria-expanded',
      'false',
    );
    fireEvent.click(screen.getByRole('button', { name: 'GPU & Power' }));
    expect(screen.getByRole('textbox', { name: 'gpu-power input' })).toHaveValue('keep me');
    view.unmount();
    layout();
    expect(order()).toEqual(['battery', 'gpu-power', 'cpu-cores', 'network', 'processes']);
  });

  it('cancels a picked-up card without persisting a new order', async () => {
    layout();
    const handle = screen.getByRole('button', { name: 'Reorder Network' });
    handle.focus();
    fireEvent.keyDown(handle, { key: ' ', code: 'Space' });
    await waitFor(() => expect(handle).toHaveAttribute('aria-pressed', 'true'));
    await act(async () => {
      await new Promise((resolve) => setTimeout(resolve, 10));
    });
    fireEvent.keyDown(handle, { key: 'Escape', code: 'Escape' });
    await waitFor(() => expect(handle).not.toHaveAttribute('aria-pressed', 'true'));
    expect(order()).toEqual([...DEFAULT_SECTION_ORDER]);
    expect(localStorage.getItem('omniscience:section-order')).toBeNull();
  });
});
