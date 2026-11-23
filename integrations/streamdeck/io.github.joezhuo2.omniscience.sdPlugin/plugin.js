const READINGS = {
  cpu: (c) => fmt(c.cpu && c.cpu.percent, '%'),
  cpu_temp: (c) => fmt(c.cpu_temp_c, '°C'),
  memory: (c) => fmt(c.ram && c.ram.percent, '%'),
  igpu: (c) => fmt(c.igpu && c.igpu.percent, '%'),
  dgpu: (c) => fmt(c.dgpu && c.dgpu.percent, '%'),
  disk: (c) => fmt(c.disk && c.disk.percent, '%'),
  down: (c) => speed(c.net && c.net.down_bps),
  up: (c) => speed(c.net && c.net.up_bps),
  power: (c) => fmt(c.package_watts, 'W'),
};

function fmt(value, unit) {
  return typeof value === 'number' ? Math.round(value) + unit : '--';
}

function speed(bps) {
  if (typeof bps !== 'number') return '--';
  const units = ['B/s', 'KB/s', 'MB/s', 'GB/s'];
  let i = 0;
  while (bps >= 1024 && i < units.length - 1) {
    bps /= 1024;
    i++;
  }
  return (bps < 10 && i > 0 ? bps.toFixed(1) : Math.round(bps)) + units[i];
}

const keys = new Map();
let ws = null;
let timer = null;

async function refresh(context) {
  const key = keys.get(context);
  if (!key) return;
  const s = key.settings;
  let title = '--';
  if (s.token) {
    try {
      const url =
        'http://127.0.0.1:' + (s.port || 9124) + '/api/status?token=' + encodeURIComponent(s.token);
      const res = await fetch(url);
      if (res.ok) {
        const json = await res.json();
        title = (READINGS[s.reading] || READINGS.cpu)(json.core || {});
      }
    } catch (e) {
      title = '--';
    }
  }
  if (ws && ws.readyState === 1) {
    ws.send(JSON.stringify({ event: 'setTitle', context, payload: { title, target: 0 } }));
  }
}

function tick() {
  keys.forEach((_, context) => refresh(context));
}

function connectElgatoStreamDeckSocket(inPort, inPluginUUID, inRegisterEvent) {
  ws = new WebSocket('ws://127.0.0.1:' + inPort);
  ws.onopen = () => ws.send(JSON.stringify({ event: inRegisterEvent, uuid: inPluginUUID }));
  ws.onmessage = (evt) => {
    const msg = JSON.parse(evt.data);
    const context = msg.context;
    if (msg.event === 'willAppear') {
      keys.set(context, { settings: (msg.payload && msg.payload.settings) || {} });
      refresh(context);
      if (!timer) timer = setInterval(tick, 2000);
    } else if (msg.event === 'willDisappear') {
      keys.delete(context);
      if (!keys.size && timer) {
        clearInterval(timer);
        timer = null;
      }
    } else if (msg.event === 'didReceiveSettings') {
      keys.set(context, { settings: (msg.payload && msg.payload.settings) || {} });
      refresh(context);
    } else if (msg.event === 'keyDown') {
      refresh(context);
    }
  };
}
