import { act, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';
import { useTelemetryStore } from '../../store';
import { sampleDetail, sampleSnapshot } from '../../test/fixtures';
import type { Battery } from '../../types/telemetry';
import { BatteryCard } from './BatteryCard';

const noHealth = {
  design_capacity_mwh: null,
  full_charge_capacity_mwh: null,
  wear_pct: null,
  cycle_count: null,
};

const show = (patch: Partial<Battery>) =>
  act(() =>
    useTelemetryStore.getState().setSnapshot({
      ...sampleSnapshot,
      detail: { ...sampleDetail, battery: { ...sampleDetail.battery!, ...noHealth, ...patch } },
    }),
  );

describe('BatteryCard health', () => {
  beforeEach(() => {
    useTelemetryStore.setState({ snapshot: null, lastTickAt: null });
  });

  it('shows wear, capacity and cycle count', () => {
    show({
      design_capacity_mwh: 60000,
      full_charge_capacity_mwh: 50000,
      wear_pct: 16.7,
      cycle_count: 212,
    });
    render(<BatteryCard />);
    const health = screen.getByTestId('battery-health');
    expect(health).toHaveTextContent('Wear 16.7%');
    expect(health).toHaveTextContent('50.0 Wh / 60.0 Wh');
    expect(health).toHaveTextContent('212 cycles');
  });

  it('shows only the readings that are available', () => {
    show({ cycle_count: 87 });
    render(<BatteryCard />);
    const health = screen.getByTestId('battery-health');
    expect(health).toHaveTextContent('87 cycles');
    expect(health).not.toHaveTextContent('Wear');
    expect(health).not.toHaveTextContent('Wh');
  });

  it('hides the health row when the driver reports nothing', () => {
    show({});
    render(<BatteryCard />);
    expect(screen.getByTestId('battery-percent')).toHaveTextContent('78%');
    expect(screen.queryByTestId('battery-health')).not.toBeInTheDocument();
  });

  it('keeps the charge and drain readings beside the health row', () => {
    show({ wear_pct: 4.2 });
    render(<BatteryCard />);
    expect(screen.getByTestId('battery-health')).toHaveTextContent('Wear 4.2%');
    expect(screen.getByTestId('battery-summary')).toHaveTextContent('2h 30m left');
  });
});
