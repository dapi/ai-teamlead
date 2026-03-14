# Issue 12: `zellij` layout при создании сессии

Статус: draft, implementation approved
Тип задачи: feature
Размер: medium
Последнее обновление: 2026-03-13

## Контекст

Issue: `zellij: опциональный layout при создании сессии`

- GitHub: https://github.com/dapi/ai-teamlead/issues/12
- Analysis branch: `analysis/issue-12`
- Session UUID: `e4c49c59-1bb8-4550-8e89-eb00515ea098`

Проблема состоит из двух связанных частей:

1. Сейчас новая `zellij` session создается через сгенерированный минимальный
   `launch-layout.kdl`, поэтому пользователь не может подключить свой именованный
   layout из `zellij`.
2. Когда `zellij.layout` не задан, launcher все равно стартует session в
   "bare" режиме и теряет привычный default UX `zellij`.

Цель анализа: зафиксировать минимальный дизайн, в котором новая session может
стартовать либо с пользовательским layout, либо с нормальным built-in default
UX, а analysis tab продолжает добавляться автоматически.

Дополнительное обязательное требование заказчика:

- analysis tab должна выглядеть как родной tab текущей session, а не как
  отдельная минимальная техническая вкладка;
- если в session/layout используются `compact bar`, плагины или другие
  versioned UX-элементы tab, analysis tab не должна молча терять этот
  контракт.

## Артефакты

## Что строим

- [01-what-we-build.md](./01-what-we-build.md)

## Как строим

- [02-how-we-build.md](./02-how-we-build.md)

## Как проверяем

- [03-how-we-verify.md](./03-how-we-verify.md)

## План имплементации

- [04-implementation-plan.md](./04-implementation-plan.md)

## Связанный контекст

- [../../../docs/features/0003-agent-launch-orchestration/README.md](../../../docs/features/0003-agent-launch-orchestration/README.md)
- [../../../docs/adr/0011-use-zellij-main-release-in-ci.md](../../../docs/adr/0011-use-zellij-main-release-in-ci.md)
- [../../../docs/adr/0022-zellij-layout-contract-for-new-sessions.md](../../../docs/adr/0022-zellij-layout-contract-for-new-sessions.md)

## Вывод анализа

Информации в issue достаточно, чтобы готовить план реализации без дополнительных
вопросов пользователю.

План согласован и может идти в реализацию при следующем контракте:

- `zellij.layout` принимает только строковое имя layout;
- если поле отсутствует, новая session создается без bare generated layout;
- analysis tab должна собираться из versioned tab-layout контракта, а не из
  голого минимального runtime layout, если нужно сохранить "родной" UX session;
- решение зафиксировано отдельным ADR, а не только issue-спекой.

Отдельно зафиксировано ограничение текущего минимального дизайна:

- hardcoded generated layout с одним tab и одним pane сам по себе не
  гарантирует, что analysis tab будет выглядеть как родной tab выбранной
  session;
- для этого нужен явный source of truth для tab-level UX, например
  project-local template или другой versioned layout contract.
