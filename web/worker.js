import init, { evaluate } from './pkg/slice_web.js';
try {
  await init({ module_or_path: new URL('./pkg/slice_web_bg.wasm', import.meta.url) });
  self.postMessage({ type: 'ready' });
} catch (error) {
  self.postMessage({ type: 'startup-error', message: String(error) });
}
self.onmessage = ({ data }) => {
  if (data.type !== 'evaluate') return;
  try {
    const canonical = evaluate(data.request);
    self.postMessage({ type: 'result', id: data.id, canonical });
  } catch (error) {
    self.postMessage({ type: 'compute-error', id: data.id, message: String(error) });
  }
};
