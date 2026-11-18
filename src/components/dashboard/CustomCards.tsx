import { useEffect, useState } from 'react';
import { fillTemplate } from '../../lib/customCards';
import { invoke, isEmbedded } from '../../lib/ipc';
import { useTelemetryStore } from '../../store';
import type { CustomCard } from '../../types/generated/CustomCard';
import { Card } from './Card';

const LIST_MS = 30_000;

function CustomCardView({ card }: { card: CustomCard }) {
  const core = useTelemetryStore((s) => s.snapshot?.core);
  const detail = useTelemetryStore((s) => s.snapshot?.detail);
  const [data, setData] = useState<unknown>(undefined);
  const [fetchError, setFetchError] = useState<string | null>(null);
  const { id, refresh_s: refresh } = card;

  useEffect(() => {
    if (!refresh) return;
    let live = true;
    const fetchData = () =>
      invoke<unknown>('get_custom_card_data', { id })
        .then((value) => {
          if (!live) return;
          setData(value);
          setFetchError(null);
        })
        .catch((e) => live && setFetchError(String(e)));
    void fetchData();
    const timer = setInterval(fetchData, refresh * 1000);
    return () => {
      live = false;
      clearInterval(timer);
    };
  }, [id, refresh]);

  return (
    <Card title={card.title} section={`custom:${id}`}>
      {card.error ? (
        <p className="text-[11px] text-island-crit">{card.error}</p>
      ) : (
        <div className="space-y-1 text-[11px]">
          {card.rows.map((row, i) => (
            <div key={i} className="flex items-baseline justify-between gap-2">
              <span className="text-island-text">{row.label}</span>
              <span className="text-island-muted tabular-nums">
                {fillTemplate(row.value, { core, detail, data })}
              </span>
            </div>
          ))}
          {fetchError && (
            <p role="status" className="text-[10px] text-island-muted">
              {fetchError}
            </p>
          )}
        </div>
      )}
    </Card>
  );
}

export function CustomCards() {
  const [cards, setCards] = useState<CustomCard[]>([]);

  useEffect(() => {
    if (isEmbedded()) return;
    let live = true;
    const load = () =>
      invoke<CustomCard[]>('get_custom_cards')
        .then((list) => live && Array.isArray(list) && setCards(list))
        .catch(() => {});
    void load();
    const timer = setInterval(load, LIST_MS);
    return () => {
      live = false;
      clearInterval(timer);
    };
  }, []);

  return (
    <>
      {cards.map((card) => (
        <CustomCardView key={card.id} card={card} />
      ))}
    </>
  );
}
