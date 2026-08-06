export const meta = {
  name: 'pre-commit',
  description: '提交前验证三件套: fmt + clippy + test。零失败才通过。',
  phases: [
    { title: 'Format', detail: 'cargo fmt --check' },
    { title: 'Clippy', detail: 'cargo clippy -- -D warnings' },
    { title: 'Test', detail: 'cargo test --lib --tests' },
  ],
}

// 每个 agent 用结构化结果报告成败, 聚合只读布尔值。
// (2026-08-01 修复: 旧版用 !includes('warn')/!includes('error') 对自由文本做子串判断,
// clippy 报告必含 "warnings"/"errors" 字样, 导致全绿也恒报失败。)
const RESULT_SCHEMA = {
  type: 'object',
  properties: {
    passed: { type: 'boolean', description: '该门验证是否通过' },
    detail: { type: 'string', description: '通过时的简短摘要; 失败时的完整错误输出' },
  },
  required: ['passed', 'detail'],
}

phase('Format')
const fmt = await agent(
  'Run `cargo fmt --check`. If it fails, list which files are not formatted in detail.',
  { label: 'fmt-check', schema: RESULT_SCHEMA }
)
phase('Clippy')
const clippy = await agent(
  'Run `cargo clippy -- -D warnings`. If it fails, list every warning/error in detail.',
  { label: 'clippy', schema: RESULT_SCHEMA }
)
phase('Test')
const test = await agent(
  'Run `cargo test --lib --tests`. If any test fails, include the failure details in detail.',
  { label: 'test', schema: RESULT_SCHEMA }
)

// agent 中途失败/被跳过返回 null, `r?.passed === true` 对 null 安全地判为不通过。
const allPassed = [fmt, clippy, test].every(r => r?.passed === true)

return {
  passed: allPassed,
  fmt: fmt?.detail ?? fmt,
  clippy: clippy?.detail ?? clippy,
  test: test?.detail ?? test,
}
