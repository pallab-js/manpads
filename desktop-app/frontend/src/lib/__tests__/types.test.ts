import { describe, it, expect } from 'vitest';
import { FaultFlags } from '../types';

describe('FaultFlags', () => {
  it('all flags are distinct powers of two', () => {
    const values = Object.values(FaultFlags);
    const unique = new Set(values);
    expect(unique.size).toBe(values.length);
    values.forEach(v => expect(v & (v - 1)).toBe(0));
  });

  it('defines expected flags', () => {
    expect(FaultFlags.WATCHDOG_TIMEOUT).toBe(1);
    expect(FaultFlags.GPIO_INTERLOCK_ERR).toBe(2);
    expect(FaultFlags.THERMAL_CRITICAL).toBe(4);
    expect(FaultFlags.BATTERY_LOW).toBe(8);
    expect(FaultFlags.GPS_STALE).toBe(16);
  });
});
