export const meta = {
  name: 'pre-commit',
  description: '提交前验证三件套: fmt + clippy + test。零失败才通过。',
  phases: [
    { title: 'Format', detail: 'cargo fmt --check' },
    { title: 'Clippy', detail: 'cargo clippy -- -D warnings' },
    { title: 'Test', detail: 'cargo test --lib --tests' },
  ],
}

phase('Format')
const fmtOk = await agent(
  'Run `cargo fmt --check` and report the result. If it fails, report which files are not formatted.',
  { label: 'fmt-check' }
)
log(`fmt: ${fmtOk}`)

phase('Clippy')
const clippyOk = await agent(
  'Run `cargo clippy -- -D warnings` and report the result. If it fails, extract and report every warning/error.',
  { label: 'clippy' }
)
log(`clippy: ${clippyOk}`)

phase('Test')
const testOk = await agent(
  'Run `cargo test --lib --tests` and report the result. If any test fails, extract and report the failure details.',
  { label: 'test' }
)
log(`test: ${testOk}`)

const allPassed = [fmtOk, clippyOk, testOk].every(r =>
  r && (typeof r === 'string' ? !r.includes('FAIL') && !r.includes('error') && !r.includes('warn') : true)
)

return {
  passed: allPassed,
  fmt: fmtOk,
  clippy: clippyOk,
  test: testOk,
}
