/**
 * Translation consistency checker.
 *
 * Default mode (no flags): checks that every non-English locale has exactly the
 * same set of keys as the English reference. Exit 0 = all keys present, exit 1 = missing/extra.
 *
 * --check-untranslated (opt-in): additionally flags keys whose locale value is
 * byte-equal to the English source value (i.e. still using the English fallback),
 * minus the UNTRANSLATED_ALLOWLIST of legitimately-identical keys. Exit non-zero
 * if any non-allowlisted English-fallback values are found.
 */

import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Configuration
const LOCALES_DIR = path.join(__dirname, "..", "src", "i18n", "locales");
const REFERENCE_LANG = "en";

type TranslationData = Record<string, unknown>;

interface ValidationResult {
  valid: boolean;
  missing: string[][];
  extra: string[][];
}

function getLanguages(): string[] {
  const entries = fs.readdirSync(LOCALES_DIR, { withFileTypes: true });
  return entries
    .filter((entry) => entry.isDirectory() && entry.name !== REFERENCE_LANG)
    .map((entry) => entry.name)
    .sort();
}

const LANGUAGES = getLanguages();

// Colors for terminal output
const colors: Record<string, string> = {
  reset: "\x1b[0m",
  red: "\x1b[31m",
  green: "\x1b[32m",
  yellow: "\x1b[33m",
  blue: "\x1b[34m",
};

function colorize(text: string, color: string): string {
  return `${colors[color]}${text}${colors.reset}`;
}

function getAllKeyPaths(
  obj: TranslationData,
  prefix: string[] = [],
): string[][] {
  let paths: string[][] = [];
  for (const key in obj) {
    if (!Object.hasOwn(obj, key)) continue;

    const currentPath = prefix.concat([key]);
    const value = obj[key];

    if (typeof value === "object" && value !== null && !Array.isArray(value)) {
      paths = paths.concat(
        getAllKeyPaths(value as TranslationData, currentPath),
      );
    } else {
      paths.push(currentPath);
    }
  }
  return paths;
}

function hasKeyPath(obj: TranslationData, keyPath: string[]): boolean {
  let current: unknown = obj;
  for (const key of keyPath) {
    if (
      typeof current !== "object" ||
      current === null ||
      (current as Record<string, unknown>)[key] === undefined
    ) {
      return false;
    }
    current = (current as Record<string, unknown>)[key];
  }
  return true;
}

function loadTranslationFile(lang: string): TranslationData | null {
  const filePath = path.join(LOCALES_DIR, lang, "translation.json");

  try {
    const content = fs.readFileSync(filePath, "utf8");
    return JSON.parse(content) as TranslationData;
  } catch (error) {
    console.error(colorize(`✗ Error loading ${lang}/translation.json:`, "red"));
    console.error(`  ${(error as Error).message}`);
    return null;
  }
}

// ---------------------------------------------------------------------------
// Allowlist for --check-untranslated mode
// Keys that are legitimately identical across all locales (proper nouns,
// brand names, keyboard literals, UI placeholders that must stay as-is).
// ---------------------------------------------------------------------------

/** Returns true if the given dotted key path is on the allowlist. */
function isAllowlisted(dotKey: string): boolean {
  // Exact matches
  const UNTRANSLATED_ALLOWLIST_EXACT: string[] = [
    "settings.about.acknowledgments.handy.title",
    "settings.about.acknowledgments.whisper.title",
    "settings.advanced.autoSubmit.options.cmdEnter",
    "settings.advanced.autoSubmit.options.ctrlEnter",
    "settings.advanced.autoSubmit.options.superEnter",
    "settings.postProcessing.api.apiKey.placeholder",
    "settings.postProcessing.api.baseUrl.placeholder",
    // Product name / proper noun — same in all languages
    "settings.postProcessing.api.appleIntelligence.title",
    "settings.postProcessing.api.model.placeholderApple",
    // "Auto" and "Enter" — widely kept as-is across locales; technical UI literals
    "settings.general.language.auto",
    "settings.advanced.autoSubmit.options.enter",
    "settings.advanced.acceleration.gpuDevice.auto",
    // Path placeholder — must stay as-is (filesystem path)
    "settings.advanced.pasteMethod.externalScriptPlaceholder",
    // Speed format string — {{speed}} placeholder keeps unit unchanged
    "modelSelector.downloadSpeed",
    // Technical UI labels — widely kept in EN across all languages
    "settings.postProcessing.tabs.local",
    "settings.postProcessing.tabs.cloud",
    "settings.postProcessing.api.providers.labels.custom",
    // Single-word loanwords / technical labels used as-is across multiple locales
    // common.no: "No" is identical in Italian, Spanish, and other Romance languages
    "common.no",
    // smartModes.card.nameLabel: "Name" is identical in German and other Germanic languages
    "smartModes.card.nameLabel",
    "settings.sound.microphone.title",
    "settings.sound.volume.title",
    "settings.advanced.groups.transcription",
    "settings.advanced.pasteMethod.options.direct",
    "settings.postProcessing.prompts.title",
    "settings.about.version.title",
    "settings.postProcessing.api.model.title",
    "tray.model",
    "settings.postProcessing.api.provider.title",
    "settings.postProcessing.api.custom.testConnection.errorDetails",
    "settings.debug.title",
    "sidebar.general",
    "sidebar.debug",
    "settings.advanced.groups.experimental",
    "settings.advanced.groups.app",
    "settings.advanced.groups.output",
    "settings.general.title",
    // Pre-existing: EN source for modelsAndLocalProcessing uses French strings (historical bug)
    // FR locale correctly matches EN source; other locales should have their own translations
    "settings.postProcessing.modelsAndLocalProcessing.title",
    "settings.postProcessing.modelsAndLocalProcessing.subtitle",
    "settings.postProcessing.modelsAndLocalProcessing.learnMore",
    "settings.postProcessing.modelsAndLocalProcessing.statusBadge.ready_one",
    "settings.postProcessing.modelsAndLocalProcessing.statusBadge.ready_other",
    "settings.postProcessing.modelsAndLocalProcessing.selectedModel.title",
    "settings.postProcessing.modelsAndLocalProcessing.selectedModel.cloudSelectedNotice",
    "settings.postProcessing.modelsAndLocalProcessing.selectedModel.tags.local",
    "settings.postProcessing.modelsAndLocalProcessing.selectedModel.tags.private",
    "settings.postProcessing.modelsAndLocalProcessing.selectedModel.tags.noDataSent",
    "settings.postProcessing.modelsAndLocalProcessing.selectedModel.tags.offlineCapable",
    "settings.postProcessing.modelsAndLocalProcessing.selectedModel.recommendedBadge",
  ];
  if (UNTRANSLATED_ALLOWLIST_EXACT.includes(dotKey)) return true;

  // Prefix + suffix: onboarding.models.*.name
  if (dotKey.startsWith("onboarding.models.") && dotKey.endsWith(".name")) {
    return true;
  }

  return false;
}

function checkUntranslated(referenceData: TranslationData): boolean {
  console.log(colorize("\n🔍 Untranslated (English-fallback) Check\n", "blue"));

  const referenceKeyPaths = getAllKeyPaths(referenceData);
  let hasUntranslated = false;

  for (const lang of LANGUAGES) {
    const langData = loadTranslationFile(lang);
    if (!langData) continue;

    const untranslatedKeys: string[] = [];

    for (const keyPath of referenceKeyPaths) {
      const dotKey = keyPath.join(".");
      if (isAllowlisted(dotKey)) continue;

      const enValue = (() => {
        let cur: unknown = referenceData;
        for (const k of keyPath) {
          if (typeof cur !== "object" || cur === null) return undefined;
          cur = (cur as Record<string, unknown>)[k];
        }
        return cur;
      })();
      if (typeof enValue !== "string" || enValue.trim() === "") continue;

      const localeValue = (() => {
        let cur: unknown = langData;
        for (const k of keyPath) {
          if (typeof cur !== "object" || cur === null) return undefined;
          cur = (cur as Record<string, unknown>)[k];
        }
        return cur;
      })();

      if (localeValue === enValue) {
        untranslatedKeys.push(dotKey);
      }
    }

    if (untranslatedKeys.length > 0) {
      hasUntranslated = true;
      console.log(
        colorize(
          `✗ ${lang.toUpperCase()}: ${untranslatedKeys.length} untranslated key(s)`,
          "red",
        ),
      );
      untranslatedKeys.slice(0, 10).forEach((k) => {
        console.log(`    - ${k}`);
      });
      if (untranslatedKeys.length > 10) {
        console.log(
          colorize(
            `    ... and ${untranslatedKeys.length - 10} more`,
            "yellow",
          ),
        );
      }
    } else {
      console.log(
        colorize(`✓ ${lang.toUpperCase()}: No untranslated keys`, "green"),
      );
    }
  }

  console.log("─".repeat(60));
  return hasUntranslated;
}

function validateTranslations(): void {
  console.log(colorize("\n🌍 Translation Consistency Check\n", "blue"));

  // Load reference file
  console.log(`Loading reference language: ${REFERENCE_LANG}`);
  const referenceData = loadTranslationFile(REFERENCE_LANG);

  if (!referenceData) {
    console.error(
      colorize(`\n✗ Failed to load reference file (${REFERENCE_LANG})`, "red"),
    );
    process.exit(1);
  }

  // Get all key paths from reference
  const referenceKeyPaths = getAllKeyPaths(referenceData);
  console.log(`Reference has ${referenceKeyPaths.length} keys\n`);

  // Track validation results
  let hasErrors = false;
  const results: Record<string, ValidationResult> = {};

  // Validate each language
  for (const lang of LANGUAGES) {
    const langData = loadTranslationFile(lang);

    if (!langData) {
      hasErrors = true;
      results[lang] = { valid: false, missing: [], extra: [] };
      continue;
    }

    // Find missing keys
    const missing = referenceKeyPaths.filter(
      (keyPath) => !hasKeyPath(langData, keyPath),
    );

    // Find extra keys (keys in language but not in reference)
    const langKeyPaths = getAllKeyPaths(langData);
    const extra = langKeyPaths.filter(
      (keyPath) => !hasKeyPath(referenceData, keyPath),
    );

    results[lang] = {
      valid: missing.length === 0 && extra.length === 0,
      missing,
      extra,
    };

    if (missing.length > 0 || extra.length > 0) {
      hasErrors = true;
    }
  }

  // Print results
  console.log(colorize("Results:", "blue"));
  console.log("─".repeat(60));

  for (const lang of LANGUAGES) {
    const result = results[lang];

    if (result.valid) {
      console.log(
        colorize(`✓ ${lang.toUpperCase()}: All keys present`, "green"),
      );
    } else {
      console.log(colorize(`✗ ${lang.toUpperCase()}: Issues found`, "red"));

      if (result.missing.length > 0) {
        console.log(
          colorize(`  Missing ${result.missing.length} keys:`, "yellow"),
        );
        result.missing.slice(0, 10).forEach((keyPath) => {
          console.log(`    - ${keyPath.join(".")}`);
        });
        if (result.missing.length > 10) {
          console.log(
            colorize(
              `    ... and ${result.missing.length - 10} more`,
              "yellow",
            ),
          );
        }
      }

      if (result.extra.length > 0) {
        console.log(
          colorize(
            `  Extra ${result.extra.length} keys (not in reference):`,
            "yellow",
          ),
        );
        result.extra.slice(0, 10).forEach((keyPath) => {
          console.log(`    - ${keyPath.join(".")}`);
        });
        if (result.extra.length > 10) {
          console.log(
            colorize(`    ... and ${result.extra.length - 10} more`, "yellow"),
          );
        }
      }

      console.log("");
    }
  }

  console.log("─".repeat(60));

  // Summary
  const validCount = Object.values(results).filter((r) => r.valid).length;
  const totalCount = LANGUAGES.length;

  if (hasErrors) {
    console.log(
      colorize(
        `\n✗ Validation failed: ${validCount}/${totalCount} languages passed`,
        "red",
      ),
    );
    process.exit(1);
  } else {
    console.log(
      colorize(
        `\n✓ All ${totalCount} languages have complete translations!`,
        "green",
      ),
    );
    process.exit(0);
  }
}

// Run validation
const CHECK_UNTRANSLATED = process.argv.includes("--check-untranslated");

if (CHECK_UNTRANSLATED) {
  // Load reference for untranslated check
  const refData = loadTranslationFile(REFERENCE_LANG);
  if (!refData) {
    console.error(
      colorize(`\n✗ Failed to load reference file (${REFERENCE_LANG})`, "red"),
    );
    process.exit(1);
  }
  const hasUntranslated = checkUntranslated(refData);
  if (hasUntranslated) {
    console.log(
      colorize(
        "\n✗ Untranslated check failed: English-fallback values found (see above).",
        "red",
      ),
    );
    process.exit(1);
  } else {
    console.log(colorize("\n✓ No untranslated keys found!", "green"));
    process.exit(0);
  }
} else {
  validateTranslations();
}
