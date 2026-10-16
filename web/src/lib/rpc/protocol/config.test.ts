import { describe, expect, test } from 'bun:test'
import { DAEMON_CONFIG_FIELDS, daemonConfigField, type DaemonConfigKind } from './config'

// tsconfig 未引入 Bun 类型，仅声明测试用到的最小接口。
declare const Bun: { file(path: URL): { text(): Promise<string> } }

// 镜像契约：native/protocol/src/daemon_config.rs 的 DAEMON_CONFIG_FIELDS。
//
// 漂移的代价是静默的：`normalizeDaemonValue` 对未知键返回 Error（该设置项在 Web 上
// 变成「填了也不生效」，桌面同项却正常），而默认值不一致会让未持久化的键在两端
// 显示不同。只有本测试能在提交前拦住它。
type RustField = { key: string; kind: string; default: string; min?: number; max?: number }

/** 解析 Rust 字面量整数（`86_400` 形式的下划线分隔）。非字面量（`i64::MAX`）返回 undefined。 */
function parseLiteralInt(text: string): number | undefined {
  return /^-?\d[\d_]*$/.test(text) ? Number(text.replaceAll('_', '')) : undefined
}

/** 从 Rust 目录取出每个键的类型、默认值与字面量边界。 */
function parseRustFields(block: string): RustField[] {
  const fields: RustField[] = []
  // 逐条切分：每条以 `field(` 开头，到下一条 `field(` 或块尾为止。
  const entries = block.split(/\n\s*field\(/).slice(1)
  for (const entry of entries) {
    const body = `field(${entry}`
    const quoted = [...body.matchAll(/"([^"]*)"/g)].map((m) => m[1] as string)
    if (quoted.length === 0) continue
    // 第一条字符串是键；随后数组字面量里的字符串（若有）属于枚举项而非默认值，
    // 因此默认值取最后一条字符串。
    const record: RustField = {
      key: quoted[0] as string,
      kind: body.match(/DaemonConfigKind::(\w+)/)?.[1] ?? 'Text',
      default: quoted[quoted.length - 1] as string,
    }
    const bounds = body.match(/min:\s*(-?[\d_]+)\s*,\s*max:\s*(-?[\d_]+)/)
    if (bounds) {
      record.min = parseLiteralInt(bounds[1] as string)
      record.max = parseLiteralInt(bounds[2] as string)
    }
    fields.push(record)
  }
  return fields
}

async function rustFields(): Promise<RustField[]> {
  const source = await Bun.file(new URL('../../../../../native/protocol/src/daemon_config.rs', import.meta.url)).text()
  const start = source.indexOf('pub const DAEMON_CONFIG_FIELDS')
  return parseRustFields(source.slice(start, source.indexOf('\n];', start)))
}

const KIND_BY_RUST: Record<string, DaemonConfigKind> = {
  Bool: 'bool',
  Integer: 'integer',
  Float: 'float',
  Enum: 'enum',
  Text: 'text',
  ReadOnly: 'readOnly',
}

describe('daemon 配置目录镜像', () => {
  test('键集与 Rust DAEMON_CONFIG_FIELDS 一致（无遗漏、无多余、无重复）', async () => {
    const rust = (await rustFields()).map((f) => f.key)
    const web = DAEMON_CONFIG_FIELDS.map((field) => field.key)
    expect(new Set(web).size).toBe(web.length)
    expect([...web].sort()).toEqual([...rust].sort())
  })

  test('每个键的类型与默认值与 Rust 一致', async () => {
    for (const rust of await rustFields()) {
      const web = daemonConfigField(rust.key)
      expect(web !== undefined).toBe(true)
      expect(web?.kind).toBe(KIND_BY_RUST[rust.kind] ?? 'text')
      expect(web?.default).toBe(rust.default)
    }
  })

  test('字面量整数边界与 Rust 一致', async () => {
    for (const rust of await rustFields()) {
      if (rust.kind !== 'Integer' || rust.min === undefined || rust.max === undefined) continue
      const web = daemonConfigField(rust.key)
      expect(web?.min).toBe(rust.min)
      expect(web?.max).toBe(rust.max)
    }
  })

  // BT 三个 Peer 预算键必须存在：漏掉任一个，`normalizeDaemonValue` 都会在 Web 上
  // 拒绝写入（桌面同项正常），且默认值/边界漂移会静默改变输入框范围。
  test('BT Peer 预算键的类型、默认值与边界与 Rust 逐条一致', async () => {
    const expected = [
      // 实时（已建立）任务 Peer 上限；半开拨号另由 bt_peer_connect_limit 计费。
      { key: 'bt_peer_limit', kind: 'integer', default: '256', min: 128, max: 512 },
      // 单任务半开连接上限。
      { key: 'bt_peer_connect_limit', kind: 'integer', default: '64', min: 32, max: 256 },
      // 会话 Peer 连接预算（含连接尝试与磁力 metadata）。
      { key: 'bt_session_peer_limit', kind: 'integer', default: '1024', min: 128, max: 4_096 },
    ] as const
    const rust = await rustFields()
    for (const want of expected) {
      const web = daemonConfigField(want.key)
      expect(web !== undefined).toBe(true)
      expect(web?.kind).toBe(want.kind)
      expect(web?.default).toBe(want.default)
      expect(web?.min).toBe(want.min)
      expect(web?.max).toBe(want.max)
      // 同一份期望值再与 Rust 目录对账，确保这里写死的数字不是自说自话。
      const canonical = rust.find((field) => field.key === want.key)
      expect(canonical?.kind).toBe('Integer')
      expect(canonical?.default).toBe(want.default)
      expect(canonical?.min).toBe(want.min)
      expect(canonical?.max).toBe(want.max)
    }
  })

  test('枚举键在 Web 侧带可选值且包含默认值', async () => {
    for (const rust of await rustFields()) {
      if (rust.kind !== 'Enum') continue
      const options = daemonConfigField(rust.key)?.options ?? []
      expect(options.length > 1).toBe(true)
      expect(options.includes(rust.default)).toBe(true)
    }
  })
})
