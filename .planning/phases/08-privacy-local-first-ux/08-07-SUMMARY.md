---
phase: 08-privacy-local-first-ux
plan: "07"
subsystem: i18n
tags: [i18n, locale-propagation, privacy, local-first]
dependency_graph:
  requires: [08-06]
  provides: [i18n-parity-gate-green, all-locales-complete]
  affects: [bun-check-translations, frontend-build]
tech_stack:
  added: []
  patterns: [json-object-insertion, python-json-manipulation]
key_files:
  created: []
  modified:
    - src/i18n/locales/ar/translation.json
    - src/i18n/locales/bg/translation.json
    - src/i18n/locales/cs/translation.json
    - src/i18n/locales/de/translation.json
    - src/i18n/locales/es/translation.json
    - src/i18n/locales/fr/translation.json
    - src/i18n/locales/he/translation.json
    - src/i18n/locales/it/translation.json
    - src/i18n/locales/ja/translation.json
    - src/i18n/locales/ko/translation.json
    - src/i18n/locales/pl/translation.json
    - src/i18n/locales/pt/translation.json
    - src/i18n/locales/ru/translation.json
    - src/i18n/locales/sv/translation.json
    - src/i18n/locales/tr/translation.json
    - src/i18n/locales/uk/translation.json
    - src/i18n/locales/vi/translation.json
    - src/i18n/locales/zh/translation.json
    - src/i18n/locales/zh-TW/translation.json
decisions:
  - fr/translation.json carries verbatim French copy of cloudToggle + modelsAndLocalProcessing — source values are canonical French per mockup product intent
  - brand names preserved verbatim across all 19 locales (OpenAI, Anthropic, Groq, Cerebras, OpenRouter, Z.AI, Apple Intelligence, Ollama, gemma3:4b, GGUF, Custom (local))
  - {{count}} placeholders preserved verbatim in statusBadge.ready_one + ready_other
metrics:
  duration: ~5min
  completed: "2026-05-22T16:12:50Z"
  tasks_completed: 1
  files_modified: 19
requirements: [PRIV-01, PRIV-02, PRIV-03]
---

# Phase 8 Plan 07: i18n Locale Propagation (Phase 8 Cloud-Toggle + Local Models Keys) Summary

Propagated the 24 new scalar keys (under 25 logical key paths from Plan 08-06) to all 19 sibling locale files so `bun run check:translations` exits 0 and all UI surfaces render native-language strings in every supported locale.

## Tasks Completed

| Task | Name                                        | Commit  | Files                |
| ---- | ------------------------------------------- | ------- | -------------------- |
| 1    | Propagate 25 keys to all 19 sibling locales | 8a194e2 | 19 locale JSON files |

## What Was Done

Plan 08-06 added 25 new English i18n keys (24 scalar leaves) under:

- `settings.postProcessing.cloudToggle` (2 keys: label, description)
- `settings.postProcessing.modelsAndLocalProcessing` (22 keys: title, subtitle, learnMore, statusBadge.ready_one/other, selectedModel.title/recommendedBadge/tags.local/private/noDataSent/offlineCapable/cloudSelectedNotice, library.title/comingSoonBadge/comingSoonBody/currentBridge, pillars.privacy.title/body, pillars.control.title/body, pillars.simplicity.title/body)

This plan propagated those keys to all 19 sibling locales with appropriate translations, preserving brand names and `{{count}}` placeholders verbatim.

## Translation Table

| Locale | title                           | subtitle                                                 | learnMore          | recommendedBadge | comingSoonBadge | pillars.privacy.title                | pillars.control.title          | pillars.simplicity.title  |
| ------ | ------------------------------- | -------------------------------------------------------- | ------------------ | ---------------- | --------------- | ------------------------------------ | ------------------------------ | ------------------------- |
| fr     | Modèles et traitement local     | Dictus privilégie les modèles locaux...                  | En savoir plus     | Recommandé       | Bientôt         | Confidentialité par défaut           | Contrôle utilisateur           | Expérience simplifiée     |
| es     | Modelos y procesamiento local   | Dictus prioriza los modelos locales...                   | Más información    | Recomendado      | Próximamente    | Privacidad por defecto               | Control del usuario            | Experiencia simplificada  |
| de     | Modelle und lokale Verarbeitung | Dictus bevorzugt lokale Modelle...                       | Mehr erfahren      | Empfohlen        | Demnächst       | Privatsphäre standardmäßig           | Nutzerkontrolle                | Vereinfachte Erfahrung    |
| it     | Modelli ed elaborazione locale  | Dictus privilegia i modelli locali...                    | Scopri di più      | Consigliato      | Prossimamente   | Privacy per impostazione predefinita | Controllo utente               | Esperienza semplificata   |
| pt     | Modelos e processamento local   | O Dictus prioriza modelos locais...                      | Saiba mais         | Recomendado      | Em breve        | Privacidade por padrão               | Controle do usuário            | Experiência simplificada  |
| ja     | モデルとローカル処理            | Dictusはあなたのプライバシーを保証するために...          | 詳細はこちら       | おすすめ         | 近日公開        | デフォルトでプライバシー             | ユーザーコントロール           | シンプルな体験            |
| ko     | 모델 및 로컬 처리               | Dictus는 개인 정보 보호를 위해 로컬 모델을 우선시합니다. | 자세히 알아보기    | 추천             | 곧 출시         | 기본 개인 정보 보호                  | 사용자 제어                    | 간소화된 경험             |
| zh     | 模型与本地处理                  | Dictus 优先使用本地模型以保障您的隐私。                  | 了解更多           | 推荐             | 即将推出        | 默认隐私保护                         | 用户控制                       | 简化体验                  |
| zh-TW  | 模型與本地處理                  | Dictus 優先使用本地模型以保障您的隱私。                  | 了解更多           | 推薦             | 即將推出        | 預設隱私保護                         | 使用者控制                     | 簡化體驗                  |
| ru     | Модели и локальная обработка    | Dictus отдаёт приоритет локальным моделям...             | Подробнее          | Рекомендуется    | Скоро           | Конфиденциальность по умолчанию      | Контроль пользователя          | Упрощённый опыт           |
| uk     | Моделі та локальна обробка      | Dictus надає перевагу локальним моделям...               | Дізнатися більше   | Рекомендовано    | Незабаром       | Конфіденційність за замовчуванням    | Контроль користувача           | Спрощений досвід          |
| pl     | Modele i przetwarzanie lokalne  | Dictus preferuje modele lokalne...                       | Dowiedz się więcej | Polecane         | Wkrótce         | Prywatność domyślnie                 | Kontrola użytkownika           | Uproszczone doświadczenie |
| cs     | Modely a lokální zpracování     | Dictus upřednostňuje lokální modely...                   | Zjistit více       | Doporučeno       | Již brzy        | Soukromí ve výchozím nastavení       | Uživatelská kontrola           | Zjednodušený zážitek      |
| bg     | Модели и локална обработка      | Dictus дава приоритет на локални модели...               | Научете повече     | Препоръчано      | Скоро           | Поверителност по подразбиране        | Контрол на потребителя         | Опростено изживяване      |
| sv     | Modeller och lokal bearbetning  | Dictus prioriterar lokala modeller...                    | Läs mer            | Rekommenderas    | Kommer snart    | Integritet som standard              | Användarkontroll               | Förenklad upplevelse      |
| tr     | Modeller ve yerel işleme        | Dictus, gizliliğinizi garanti altına almak için...       | Daha fazla bilgi   | Önerilen         | Yakında         | Varsayılan olarak gizlilik           | Kullanıcı kontrolü             | Sadeleştirilmiş deneyim   |
| vi     | Mô hình và xử lý cục bộ         | Dictus ưu tiên các mô hình cục bộ...                     | Tìm hiểu thêm      | Được khuyến nghị | Sắp ra mắt      | Quyền riêng tư mặc định              | Quyền kiểm soát của người dùng | Trải nghiệm đơn giản hóa  |
| ar     | النماذج والمعالجة المحلية       | يعطي Dictus الأولوية للنماذج المحلية...                  | اعرف المزيد        | موصى به          | قريبًا          | الخصوصية افتراضيًا                   | تحكم المستخدم                  | تجربة مبسطة               |
| he     | מודלים ועיבוד מקומי             | Dictus מעדיף מודלים מקומיים...                           | למידע נוסף         | מומלץ            | בקרוב           | פרטיות כברירת מחדל                   | שליטת משתמש                    | חוויה פשוטה               |

## Verification Results

```
bun run check:translations
✓ AR: All keys present
✓ BG: All keys present
✓ CS: All keys present
✓ DE: All keys present
✓ ES: All keys present
✓ FR: All keys present
✓ HE: All keys present
✓ IT: All keys present
✓ JA: All keys present
✓ KO: All keys present
✓ PL: All keys present
✓ PT: All keys present
✓ RU: All keys present
✓ SV: All keys present
✓ TR: All keys present
✓ UK: All keys present
✓ VI: All keys present
✓ ZH: All keys present
✓ ZH-TW: All keys present
✓ All 19 languages have complete translations!
```

`bun run build` exits 0 (Vite build green, 1930 modules transformed).

## Key Decisions Made

1. **fr/translation.json — verbatim copy**: Since the English source values are already in French (per mockup product intent for Plan 08-06), the French sibling carries the source values verbatim with no re-translation.

2. **Brand name discipline**: All brand names (OpenAI, Anthropic, Groq, Cerebras, OpenRouter, Z.AI, Apple Intelligence, Ollama, Dictus, gemma3:4b, GGUF, Custom (local)) preserved verbatim in all 19 locales. Brand name sweep confirmed zero failures.

3. **{{count}} placeholder discipline**: Both `statusBadge.ready_one` and `statusBadge.ready_other` preserve `{{count}}` verbatim. Each locale pluralizes the surrounding text naturally (e.g., de: "1 Modell bereit" / "N Modelle bereit").

4. **Insertion order**: `cloudToggle` → `modelsAndLocalProcessing` → `prompts` in each locale's `settings.postProcessing` object, matching the English source structure exactly.

## Deviations from Plan

None — plan executed exactly as written. The translation check script reported 24 missing scalar leaves per locale (not 25 as stated in the plan — 25 is the count of logical key paths, 24 is scalar leaves since `cloudToggle.label` and `cloudToggle.description` account for 2 of the 25, and `statusBadge` has `ready_one`+`ready_other` for 2 leaves). This is a count discrepancy in the plan narrative vs actual leaf count; both the script and acceptance criteria confirm all required keys are present.

## Phase 8 Gap Closure Status

Phase 8 i18n propagation is now complete:

- Plan 08-04: propagated 03 plans' keys to 19 locales
- Plan 08-07: propagated Plan 08-06 keys (cloud-toggle + local-models UI) to 19 locales
- `bun run check:translations` exits 0 with "All 19 languages have complete translations!"
- Ready for UAT re-test of test 9 (ProviderPicker local-first surface) and tests 10-14 (skipped pending the design pivot from Plan 08-05)

## Self-Check: PASSED

- 19 locale files modified: confirmed (git log shows 19 files changed, 836 insertions)
- Commit 8a194e2 exists: confirmed
- `bun run check:translations` exits 0: confirmed
- `bun run build` exits 0: confirmed
- Brand names preserved: confirmed (no failures in sweep)
