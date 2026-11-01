import { act, fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, expect, it } from 'vitest';
import { useSettingsStore, type Settings } from '../../settings';
import { Dashboard } from '../dashboard/Dashboard';

const withConfig = (config_enabled: boolean) =>
  useSettingsStore.setState({ settings: { config_enabled } as Settings });

beforeEach(() => withConfig(true));

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

it('hides the Info/Config switch and shows Info when config is disabled', () => {
  render(<Dashboard />);
  fireEvent.click(screen.getByRole('tab', { name: 'config' }));
  act(() => withConfig(false));
  expect(screen.queryByRole('tab', { name: 'config' })).not.toBeInTheDocument();
  expect(screen.queryByRole('tablist', { name: 'Dashboard' })).not.toBeInTheDocument();
  expect(screen.getByRole('searchbox')).toBeInTheDocument();
});
