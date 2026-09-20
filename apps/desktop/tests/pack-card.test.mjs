import assert from "node:assert/strict";
import { after, before, test } from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { createServer } from "vite";
import react from "@vitejs/plugin-react";

let server;
let MechanicalSwitch;
let PackCard;
before(async () => {
  server = await createServer({
    configFile: false,
    plugins: [react()],
    server: { middlewareMode: true, hmr: false, watch: null },
  });
  ({ MechanicalSwitch } = await server.ssrLoadModule("/src/components/MechanicalSwitch.tsx"));
  ({ PackCard } = await server.ssrLoadModule("/src/components/PackCard.tsx"));
});
after(async () => { await server?.close(); });

const pack = {
  id: "kbsim-alpaca", name: "Alpaca Switch", author: "Test author",
  version: "1.0.0", description: "A linear switch.", license: "MIT",
  tags: ["smooth", "linear"], hasReleaseSamples: true,
};
const renderSwitch = (props) => renderToStaticMarkup(createElement(MechanicalSwitch, props));
const renderCard = (props = {}) => renderToStaticMarkup(createElement(PackCard, {
  pack, index: 0, active: false, onActivate() {}, onPreview() {}, ...props,
}));

test("Alpaca uses pink original vector artwork; other switches have distinct colors", () => {
  const alpaca = renderSwitch({ packId: "kbsim-alpaca" });
  assert.match(alpaca, /--switch-housing:#bb859a/);
  assert.match(alpaca, /--switch-stem:#efb2c8/);
  assert.match(alpaca, /aria-hidden="true"/);
  assert.doesNotMatch(alpaca, /<image|<img|https?:|color-mix/);
  assert.notEqual(alpaca, renderSwitch({ packId: "kbsim-mxblue" }));
});

test("unknown and empty pack IDs have deterministic, valid fallback artwork", () => {
  for (const packId of ["my-imported-pack", "", "🎹-custom", "__proto__", "constructor"]) {
    const markup = renderSwitch({ packId });
    assert.equal(markup, renderSwitch({ packId }));
    assert.match(markup, /--switch-housing:#[a-f0-9]{6}/);
    assert.doesNotMatch(markup, /NaN|undefined/);
  }
});

test("press animation is opt-in, so idle cards do not animate continuously", () => {
  assert.doesNotMatch(renderSwitch({ packId: pack.id }), /is-striking/);
  assert.match(renderSwitch({ packId: pack.id, strike: 1 }), /is-striking/);
});

test("both preview buttons are labelled and double-digit card numbers are correct", () => {
  const markup = renderCard({ index: 9 });
  assert.match(markup, /aria-label="Preview Alpaca Switch sound"/);
  assert.match(markup, /aria-label="Preview Alpaca Switch"/);
  assert.match(markup, /class="pack-index">10<\/span>/);
  assert.match(markup, /class="switch-type">linear<\/span>/);
  assert.equal((markup.match(/<button /g) ?? []).length, 3);
});

test("active cards keep previews enabled, disable activation, and retain attribution", () => {
  const markup = renderCard({ active: true });
  assert.match(markup, /class="activate-button" disabled=""/);
  assert.equal((markup.match(/ disabled=/g) ?? []).length, 1);
  assert.match(markup, /by Test author/);
  assert.match(markup, />MIT</);
  assert.match(markup, /Press \+ release/);
});
