const statusGlyphLabels = new Map<string, string>([
  ["💥", "Kills"],
  ["📦", "Scans"],
  ["⌚", "uptime"],
  ["⏱", "uptime"],
  ["🎯", "missions"],
])
const pictographicGlyphPattern = /\p{Extended_Pictographic}/u

export function sourceDetail(folder: string, selectedFile: string | null | undefined): string {
  const safeFolder = displaySafeText(folder)
  if (selectedFile === null || selectedFile === undefined) {
    return safeFolder
  }
  return joinJournalPath(safeFolder, displaySafeText(selectedFile))
}

export function assertNever(value: never): never {
  throw new Error(`Unhandled dashboard variant: ${String(value)}`)
}

export function lineSafeText(value: string): string {
  return displaySafeText(value).replaceAll(/Journal\.[^\s/\\]+\.log/g, "Selected Journal file")
}

function displaySafeCharacter(character: string): string {
  const statusLabel = statusGlyphLabels.get(character)
  if (statusLabel !== undefined) {
    return `${statusLabel} `
  }

  const codePoint = character.codePointAt(0)
  if (
    codePoint === undefined ||
    isLineUnsafeControl(codePoint) ||
    isEmojiSequenceMarker(codePoint) ||
    isStatusGlyph(codePoint, character)
  ) {
    return " "
  }
  return character
}

function isLineUnsafeControl(codePoint: number): boolean {
  return (
    codePoint === 9 || codePoint === 10 || codePoint === 13 || codePoint === 27 || codePoint === 155
  )
}

function isEmojiSequenceMarker(codePoint: number): boolean {
  return (
    codePoint === 0x200d ||
    codePoint === 0x20e3 ||
    codePoint === 0xfe0e ||
    codePoint === 0xfe0f ||
    (codePoint >= 0x1f3fb && codePoint <= 0x1f3ff) ||
    (codePoint >= 0x1f1e6 && codePoint <= 0x1f1ff) ||
    (codePoint >= 0xe0020 && codePoint <= 0xe007f)
  )
}

function isStatusGlyph(codePoint: number, character: string): boolean {
  return (
    pictographicGlyphPattern.test(character) ||
    (codePoint >= 0x2300 && codePoint <= 0x23ff) ||
    (codePoint >= 0x2600 && codePoint <= 0x27bf) ||
    (codePoint >= 0x1f000 && codePoint <= 0x1faff)
  )
}

function displaySafeText(value: string): string {
  return Array.from(value, displaySafeCharacter)
    .join("")
    .replaceAll(/\s{2,}/g, " ")
    .trim()
}

function joinJournalPath(folder: string, selectedFile: string): string {
  if (folder === "") {
    return selectedFile
  }
  if (folder === ".") {
    return `.${pathSeparator(selectedFile)}${selectedFile}`
  }
  const separator = pathSeparator(folder)
  if (folder.endsWith("/") || folder.endsWith("\\")) {
    return `${folder}${selectedFile}`
  }
  return `${folder}${separator}${selectedFile}`
}

function pathSeparator(path: string): "/" | "\\" {
  return /^[A-Za-z]:\\/.test(path) || path.includes("\\") ? "\\" : "/"
}
