import { act, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';
import { useTelemetryStore } from '../../store';
import { sampleDetail, sampleSnapshot } from '../../test/fixtures';
import type { FanSpeeds } from '../../types/telemetry';
import { Dashboard } from './Dashboard';
import { FanCard } from './FanCard';

const show = (fans: FanSpeeds | null) =>
  act(() =>
    useTelemetryStore.getState().setSnapshot({
      ...sampleSnapshot,
      detail: { ...sampleDetail, fans },
    }),
  );

describe('FanCard', () => {
  beforeEach(() => {
    useTelemetryStore.setState({ snapshot: null, lastTickAt: null });
  });

  it('shows CPU and GPU fan speeds', () => {
    show({ cpu_rpm: 3200, gpu_rpm: 2800 });
    render(<FanCard />);
    expect(screen.getByRole('region', { name: 'Fans' })).toBeInTheDocument();
    expect(screen.getByTestId('fan-cpu')).toHaveTextContent('3200 rpm');
    expect(screen.getByTestId('fan-gpu')).toHaveTextContent('2800 rpm');
  });

  it('shows only the fans that report a reading', () => {
    show({ cpu_rpm: 4100, gpu_rpm: null });
    render(<FanCard />);
    expect(screen.getByTestId('fan-cpu')).toHaveTextContent('4100 rpm');
    expect(screen.queryByTestId('fan-gpu')).not.toBeInTheDocument();
  });

  it('labels a fan that reads zero as stopped', () => {
    show({ cpu_rpm: 0, gpu_rpm: 2800 });
    render(<FanCard />);
    expect(screen.getByTestId('fan-cpu')).toHaveTextContent('Stopped');
  });

  it('renders nothing without a reading or before detail arrives', () => {
    const { container } = render(<FanCard />);
    expect(container).toBeEmptyDOMElement();
    act(() => useTelemetryStore.getState().setSnapshot(sampleSnapshot));
    expect(container).toBeEmptyDOMElement();
    show(null);
    expect(container).toBeEmptyDOMElement();
  });

  it('is hidden from the dashboard on machines without a fan sensor', () => {
    show({ cpu_rpm: 3200, gpu_rpm: 2800 });
    render(<Dashboard />);
    expect(screen.getByRole('region', { name: 'Fans' })).toBeInTheDocument();
    show(null);
    expect(screen.queryByText('Fans')).not.toBeInTheDocument();
    expect(screen.getByRole('region', { name: 'Battery' })).toBeInTheDocument();
  });
});
