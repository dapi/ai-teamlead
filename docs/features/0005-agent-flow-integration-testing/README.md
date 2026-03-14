# Feature 0005: платформа integration-тестирования agent flow

Статус: draft
Владелец: владелец репозитория
Последнее обновление: 2026-03-14

## Контекст

Проект уже фиксирует requirement на integration tests для `poll`, `run`,
launcher-контракта и headless `zellij`, но пока не имеет отдельной платформы,
которая локально поднимает полностью изолированный sandbox и прогоняет agent
flow end-to-end.

Новая feature описывает именно такую платформу:

- с единым локальным entrypoint
- с disposable sandbox, не использующим host `zellij`
- с поддержкой реальных агентов `codex` и `claude`
- с использованием project-local и user-local настроек подключения к LLM через
  явный allowlist

## Что строим

- [01-what-we-build.md](./01-what-we-build.md)

## Как строим

- [02-how-we-build.md](./02-how-we-build.md)

## Как проверяем

- [03-how-we-verify.md](./03-how-we-verify.md)

## Зависимости

- [Feature 0001](../0001-ai-teamlead-daemon/README.md) — основной CLI и runtime
  orchestration
- [Feature 0002](../0002-repo-init/README.md) — repo-local assets и
  `settings.yml`
- [Feature 0003](../0003-agent-launch-orchestration/README.md) — launcher path,
  `zellij` и запуск агента

## Связанные документы

- [../../code-quality.md](../../code-quality.md)
- [../../issue-analysis-flow.md](../../issue-analysis-flow.md)
- [../../adr/0011-use-zellij-main-release-in-ci.md](../../adr/0011-use-zellij-main-release-in-ci.md)

## Открытые вопросы

- нужен ли отдельный CLI namespace `test` или достаточно подкоманды в
  существующем verification path
- хотим ли в MVP поддерживать live-run одновременно для `codex` и `claude` или
  начать с одного real agent profile и одного stub profile
- нужно ли отдельно ограничивать budget, timeout и максимальное число LLM
  вызовов на сценарий

## Журнал изменений

### 2026-03-14

- создан draft feature-документ для платформы integration-тестирования agent
  flow
