import assert from "node:assert/strict";
import test from "node:test";

import { requestChatCompletion } from "./deepseek-helper.mjs";

function jsonResponse(status, body) {
  return {
    ok: status >= 200 && status < 300,
    status,
    async text() {
      return JSON.stringify(body);
    },
    async json() {
      return body;
    },
  };
}

test("requestChatCompletion retries transient HTTP failures before succeeding", async () => {
  const statuses = [429, 500, 200];
  const calls = [];
  const delays = [];

  const payload = await requestChatCompletion({
    apiKey: "test-key",
    body: { model: "deepseek-chat", messages: [], temperature: 0.2 },
    fetchImpl: async (_url, init) => {
      calls.push(init);
      const status = statuses.shift();
      if (status === 200) {
        return jsonResponse(200, {
          model: "deepseek-chat",
          choices: [{ message: { content: "draft text" } }],
        });
      }
      return jsonResponse(status, { error: { message: `status ${status}` } });
    },
    sleep: async (ms) => {
      delays.push(ms);
    },
  });

  assert.equal(payload.choices[0].message.content, "draft text");
  assert.equal(calls.length, 3);
  assert.deepEqual(delays, [500, 1000]);
});
