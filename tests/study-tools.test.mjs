import assert from 'node:assert/strict';
import test from 'node:test';

test('study renderer inputs use the book pins independently of website renderer pins', async () => {
  const { studyRendererInputs } = await import('../scripts/study-tools.mjs');
  const specimen = { renderers: { split: 'a'.repeat(40), sequent: 'b'.repeat(40) }, renderer_bundles: {
    split: { path: 'packages/core/src/index.js', sha256: 'c'.repeat(64) },
    sequent: { path: 'packages/core/src/index.js', sha256: 'd'.repeat(64) },
  } };
  const inputs = studyRendererInputs(specimen);
  assert.deepEqual(inputs.map(input => [input.name, input.repository, input.revision]), [
    ['split', 'https://github.com/dikini/split-md.git', 'a'.repeat(40)],
    ['sequent', 'https://github.com/dikini/sequent-md.git', 'b'.repeat(40)],
  ]);
  assert.throws(() => studyRendererInputs({ renderers: { unknown: 'a'.repeat(40) } }), /unsupported/i);
  assert.throws(() => studyRendererInputs({ ...specimen, renderers: { split: 'main' } }), /revision/i);
  assert.throws(() => studyRendererInputs({ ...specimen, renderer_bundles: { ...specimen.renderer_bundles, split: { path: '../escape', sha256: 'c'.repeat(64) } } }), /bundle path/i);
});
