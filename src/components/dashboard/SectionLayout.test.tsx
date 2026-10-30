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
    expect(screen.queryByRole('region', { name: 'Battery' })).not.toBeInTheDocument();
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
    const handle = screen.getByRole('region', { name: 'GPU & Power' });
    handle.focus();
    fireEvent.keyDown(handle, { key: ' ', code: 'Space' });
    await waitFor(() => expect(handle).toHaveAttribute('data-dragging', 'true'));
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

  it.each(['surface', 'title', 'input'] as const)(
    'starts a mouse drag from the card %s',
    async (part) => {
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
      layout();
      const card = screen.getByRole('region', { name: 'GPU & Power' });
      const target =
        part === 'surface'
          ? card
          : part === 'title'
            ? screen.getByRole('button', { name: 'GPU & Power' })
            : screen.getByRole('textbox', { name: 'gpu-power input' });
      fireEvent.mouseDown(target, { button: 0, buttons: 1, clientX: 20, clientY: 20 });
      fireEvent.mouseMove(document, { buttons: 1, clientX: 20, clientY: 30 });
      await waitFor(() => expect(card).toHaveAttribute('data-dragging', 'true'));
      fireEvent.mouseMove(document, { buttons: 1, clientX: 20, clientY: 150 });
      fireEvent.mouseUp(document, { button: 0, clientX: 20, clientY: 150 });
      await waitFor(() => expect(order()[0]).toBe('battery'));
      expect(screen.queryByRole('button', { name: /Reorder/ })).not.toBeInTheDocument();
    },
  );

  it('keeps keyboard input in child controls from picking up a card', () => {
    layout();
    const input = screen.getByRole('textbox', { name: 'gpu-power input' });
    input.focus();
    fireEvent.keyDown(input, { key: ' ', code: 'Space' });
    expect(screen.getByRole('region', { name: 'GPU & Power' })).not.toHaveAttribute(
      'data-dragging',
      'true',
    );
    fireEvent.click(screen.getByRole('button', { name: 'GPU & Power' }));
    expect(screen.getByRole('button', { name: 'GPU & Power' })).toHaveAttribute(
      'aria-expanded',
      'false',
    );
  });

  it('cancels a picked-up card without persisting a new order', async () => {
    layout();
    const handle = screen.getByRole('region', { name: 'Network' });
    handle.focus();
    fireEvent.keyDown(handle, { key: ' ', code: 'Space' });
    await waitFor(() => expect(handle).toHaveAttribute('data-dragging', 'true'));
    await act(async () => {
      await new Promise((resolve) => setTimeout(resolve, 10));
    });
    fireEvent.keyDown(handle, { key: 'Escape', code: 'Escape' });
    await waitFor(() => expect(handle).not.toHaveAttribute('data-dragging', 'true'));
    expect(order()).toEqual([...DEFAULT_SECTION_ORDER]);
    expect(localStorage.getItem('omniscience:section-order')).toBeNull();
  });
});
