import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { Card } from './Card';

describe('Card disclosures', () => {
  it('minimizes details, keeps its heading and aside, and restores input state', () => {
    const onSort = vi.fn();
    render(
      <Card
        section="processes"
        title="Top processes"
        aside={<button onClick={onSort}>Sort</button>}
      >
        <input aria-label="Query" defaultValue="" />
      </Card>,
    );
    const toggle = screen.getByRole('button', { name: 'Top processes' });
    const input = screen.getByRole('textbox', { name: 'Query' });
    fireEvent.change(input, { target: { value: 'chrome' } });
    expect(toggle).toHaveAttribute('aria-expanded', 'true');
    const body = document.getElementById(toggle.getAttribute('aria-controls')!)!;
    fireEvent.click(toggle);
    expect(toggle).toHaveAttribute('aria-expanded', 'false');
    expect(body).not.toBeVisible();
    expect(screen.queryByRole('textbox')).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Sort' }));
    expect(onSort).toHaveBeenCalledOnce();
    expect(toggle).toHaveAttribute('aria-expanded', 'false');
    fireEvent.click(toggle);
    expect(body).toBeVisible();
    expect(screen.getByRole('textbox')).toHaveValue('chrome');
  });

  it('remembers minimization on remount and replaces the aside with a compact summary', () => {
    const view = () =>
      render(
        <Card
          section="battery"
          title="Battery"
          aside={<span>Drain 12 W</span>}
          summary={<span>78%</span>}
        >
          <p>Battery detail</p>
        </Card>,
      );
    let rendered = view();
    fireEvent.click(screen.getByRole('button', { name: 'Battery' }));
    expect(screen.getByText('78%')).toBeVisible();
    expect(screen.queryByText('Drain 12 W')).not.toBeInTheDocument();
    rendered.unmount();
    rendered = view();
    expect(screen.getByRole('button', { name: 'Battery' })).toHaveAttribute(
      'aria-expanded',
      'false',
    );
    fireEvent.click(screen.getByRole('button', { name: 'Battery' }));
    expect(screen.getByText('Drain 12 W')).toBeVisible();
    expect(screen.queryByText('78%')).not.toBeInTheDocument();
    rendered.unmount();
    view();
    expect(screen.getByRole('button', { name: 'Battery' })).toHaveAttribute(
      'aria-expanded',
      'true',
    );
  });

  it('has independent controls and keeps ordinary cards expanded', () => {
    render(
      <>
        <Card section="network" title="Network">
          Network detail
        </Card>
        <Card section="cpu-cores" title="CPU cores">
          CPU detail
        </Card>
        <Card title="Static">Always visible</Card>
      </>,
    );
    const network = screen.getByRole('button', { name: 'Network' });
    const cpu = screen.getByRole('button', { name: 'CPU cores' });
    expect(network.getAttribute('aria-controls')).not.toBe(cpu.getAttribute('aria-controls'));
    fireEvent.click(network);
    expect(screen.getByText('Network detail')).not.toBeVisible();
    expect(screen.getByText('CPU detail')).toBeVisible();
    expect(screen.getByText('Always visible')).toBeVisible();
    expect(screen.queryByRole('button', { name: 'Static' })).not.toBeInTheDocument();
  });
});
