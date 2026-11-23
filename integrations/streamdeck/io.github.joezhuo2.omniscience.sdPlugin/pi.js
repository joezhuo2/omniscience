let ws = null;
let uuid = null;
const ids = ['reading', 'port', 'token'];

function save() {
  if (!ws || ws.readyState !== 1) return;
  const settings = {};
  ids.forEach((id) => {
    settings[id] = document.getElementById(id).value;
  });
  ws.send(JSON.stringify({ event: 'setSettings', context: uuid, payload: settings }));
}

function connectElgatoStreamDeckSocket(inPort, inPropertyInspectorUUID, inRegisterEvent) {
  uuid = inPropertyInspectorUUID;
  ws = new WebSocket('ws://127.0.0.1:' + inPort);
  ws.onopen = () => {
    ws.send(JSON.stringify({ event: inRegisterEvent, uuid }));
    ws.send(JSON.stringify({ event: 'getSettings', context: uuid }));
  };
  ws.onmessage = (evt) => {
    const msg = JSON.parse(evt.data);
    if (msg.event === 'didReceiveSettings') {
      const settings = (msg.payload && msg.payload.settings) || {};
      ids.forEach((id) => {
        if (settings[id] !== undefined) document.getElementById(id).value = settings[id];
      });
    }
  };
}

ids.forEach((id) => {
  const el = document.getElementById(id);
  el.addEventListener('input', save);
  el.addEventListener('change', save);
});
