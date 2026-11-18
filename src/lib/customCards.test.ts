import { describe, expect, it } from 'vitest';
import { fillTemplate } from './customCards';

const sources = {
  core: { cpu: { percent: 12.345 }, name: 'pc' },
  detail: { gpus: [{ temp_c: 45 }, { temp_c: null }] },
  data: { items: [{ n: 3 }], ok: true, text: '7' },
};

describe('fillTemplate', () => {
  it('reads dotted paths from core, detail and data', () => {
    expect(fillTemplate('{core.name} / {detail.gpus.0.temp_c} / {data.items.0.n}', sources)).toBe(
      'pc / 45 / 3',
    );
  });

  it('formats numbers with the requested decimals', () => {
    expect(fillTemplate('{core.cpu.percent|1}%', sources)).toBe('12.3%');
    expect(fillTemplate('{core.cpu.percent|0}', sources)).toBe('12');
    expect(fillTemplate('{core.cpu.percent}', sources)).toBe('12.345');
  });

  it('leaves non-numbers as they are when decimals are given', () => {
    expect(fillTemplate('{data.text|2} {data.ok}', sources)).toBe('7 true');
  });

  it('renders anything missing as --', () => {
    expect(
      fillTemplate(
        '{nope} {core.cpu.x} {detail.gpus.5.temp_c} {detail.gpus.1.temp_c} {core.cpu}',
        sources,
      ),
    ).toBe('-- -- -- -- --');
    expect(fillTemplate('{data.a}', { core: null, detail: null, data: undefined })).toBe('--');
    expect(fillTemplate('{core.constructor}', sources)).toBe('--');
  });

  it('keeps text around placeholders and ignores text without them', () => {
    expect(fillTemplate('plain', sources)).toBe('plain');
    expect(fillTemplate('a {core.name} b {core.name}', sources)).toBe('a pc b pc');
  });
});
