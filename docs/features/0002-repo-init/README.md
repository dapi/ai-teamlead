# Feature 0002: repo init

Статус: draft
Владелец: владелец репозитория
Последнее обновление: 2026-03-13

## Контекст

Эта feature описывает подключение `ai-teamlead` к конкретному репозиторию.
Ее задача не в запуске `poll`/`run` и не в анализе issue, а в создании versioned
project-local contract layer, который потом использует остальная система.

Документ оформлен как каталог, потому что feature затрагивает CLI-контракт,
структуру repo-local файлов, границы между versioned и runtime state и
идемпотентность.

## Что строим

- [01-what-we-build.md](./01-what-we-build.md)

## Как строим

- [02-how-we-build.md](./02-how-we-build.md)

## Как проверяем

- [03-how-we-verify.md](./03-how-we-verify.md)

## План реализации

- [04-implementation-plan.md](./04-implementation-plan.md)

## Связанные документы

- [README.md](../../../README.md)
- [docs/issue-analysis-flow.md](../../issue-analysis-flow.md)
- [docs/adr/0001-repo-local-ai-config.md](../../adr/0001-repo-local-ai-config.md)
- [docs/adr/0004-runtime-artifacts-in-git-dir.md](../../adr/0004-runtime-artifacts-in-git-dir.md)
- [docs/adr/0012-repo-init-command-and-project-contract-layer.md](../../adr/0012-repo-init-command-and-project-contract-layer.md)

## Открытые вопросы

- нужен ли в первой версии `init` интерактивный режим заполнения `project_id`

## Журнал изменений

### 2026-03-13

- создан каталог feature 0002 для `ai-teamlead init`
