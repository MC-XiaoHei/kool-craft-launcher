import { attachConsole, debug, error, info, trace, warn } from "@tauri-apps/plugin-log"

export async function initLogger() {
  await attachConsole()

  forwardConsole("trace", trace)
  forwardConsole("debug", debug)
  forwardConsole("info", info)
  forwardConsole("warn", warn)
  forwardConsole("error", error)

  console.log = console.info
}

const frontendMsgTag = "\u0001"

function forwardConsole(
  fnName: "log" | "debug" | "info" | "warn" | "error" | "trace",
  logger: (message: string) => Promise<void>,
) {
  const original = console[fnName]
  console[fnName] = function (...args: any[]) {
    const message = fastFormatArgs(args)
    if (isFrontendMsg(message)) return
    original(message)
    logger(`${frontendMsgTag}${message}`).then()
  }
}

function isFrontendMsg(message: string) {
  return message.includes(`${frontendMsgTag}`)
}

function fastFormatArgs(args: any[]): string {
  if (args.length === 0) return ""
  if (args.length === 1 && typeof args[0] === "string") return args[0]

  return args
    .map(arg => {
      const type = typeof arg

      if (type === "string") return arg
      if (type === "number" || type === "boolean" || arg === null) return String(arg)
      if (type === "undefined") return "undefined"
      if (type === "symbol") return arg.toString()
      if (type === "function") return `[Function: ${arg.name || "anonymous"}]`

      try {
        return JSON.stringify(arg, getCircularReplacer())
      } catch (e) {
        return "[Unserializable Object]"
      }
    })
    .join(" ")
}

const getCircularReplacer = () => {
  const seen = new WeakSet()
  return (_key: string, value: any) => {
    if (typeof value === "object" && value !== null) {
      if (seen.has(value)) {
        return "[Circular]"
      }
      seen.add(value)
    }
    return value
  }
}
