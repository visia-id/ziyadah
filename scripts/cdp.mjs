// Evaluasi ekspresi JS di halaman WebView2 Ziyadah lewat Chrome DevTools Protocol (untuk uji).
// node scripts/cdp.mjs <url-halaman-berisi> "<ekspresi>"   (port 9334, ubah lewat CDP_PORT)
const [, , match, expr] = process.argv;
const port = process.env.CDP_PORT || 9334;
const pages = await (await fetch(`http://127.0.0.1:${port}/json`)).json();
const page = pages.find((p) => (match.endsWith("/") ? p.url.endsWith(match) : p.url.includes(match)));
if (!page) throw new Error(`halaman ${match} tidak ada`);
const ws = new WebSocket(page.webSocketDebuggerUrl);
await new Promise((r) => ws.addEventListener("open", r));
ws.send(JSON.stringify({ id: 1, method: "Runtime.evaluate", params: { expression: expr, awaitPromise: true, returnByValue: true } }));
const msg = await new Promise((r) => ws.addEventListener("message", (e) => r(JSON.parse(e.data))));
console.log(JSON.stringify(msg.result?.result?.value ?? msg.result?.exceptionDetails ?? msg));
ws.close();
