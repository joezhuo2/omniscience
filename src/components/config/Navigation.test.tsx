import { fireEvent, render, screen } from '@testing-library/react';
import { expect, it } from 'vitest';
import { Dashboard } from '../dashboard/Dashboard';

it('switches panels by keyboard and preserves Info controls', () => {
  render(<Dashboard />);
  const search = screen.getByRole('searchbox');
  fireEvent.change(search, { target: { value: 'saved query' } });
  fireEvent.keyDown(screen.getByRole('tab', { name: 'info' }), { key: 'ArrowRight' });
  expect(screen.getByRole('tab', { name: 'config' })).toHaveFocus();
  expect(screen.queryByRole('searchbox')).not.toBeInTheDocument();
  expect(screen.getByRole('tabpanel')).toHaveAccessibleName('config');
  fireEvent.keyDown(screen.getByRole('tab', { name: 'config' }), { key: 'Home' });
  expect(screen.getByRole('searchbox')).toHaveValue('saved query');
});
