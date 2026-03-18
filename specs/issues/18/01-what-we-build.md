# Issue 18: Что строим

## Problem

Сейчас `ai-teamlead run <issue>` умеет стартовать analysis flow и частично
умеет re-entry через существующий `session_uuid`, но не имеет целостного
контракта для повторного запуска issue, по которой analysis уже выполнялся и
после этого остались артефакты прошлой попытки.

Из-за этого возникают неоднозначные ситуации:

- issue все еще находится в `Waiting for Plan Review`,
  `Waiting for Clarification` или `Analysis Blocked`, но `zellij` pane уже
  потеряна;
- issue вернулась в `Backlog`, хотя предыдущий analysis branch, SDD и PR еще
  существуют;
- runtime-binding удален или устарел, но versioned артефакты и GitHub-состояние
  позволяют продолжить работу;
- analysis PR уже merged или существует больше одной предыдущей попытки, и
  система не может безопасно угадать, что оператор хочет сделать дальше.

Текущий пробел опасен по двум причинам:

- оператор не понимает, можно ли безопасно продолжать существующий analysis
  context или нужно начинать заново;
- core начинает зависеть от случайно сохранившегося runtime state вместо
  детерминированного observed state из GitHub, git и versioned артефактов.

## Who Is It For

- оператор `ai-teamlead`, который вручную повторно запускает `run <issue>` и
  ожидает recovery path без ручного поиска worktree, PR и SDD;
- владелец репозитория, которому нужен явный и документированный re-entry
  контракт для analysis stage;
- будущая реализация CLI и тестов, которой нужен детерминированный decision
  matrix вместо ad-hoc эвристик.

## Outcome

После изменения analysis re-entry должен работать по следующему контракту:

- `run` умеет обнаруживать предыдущий analysis context по issue через GitHub
  status, runtime-binding, analysis artifacts, analysis branch/worktree и PR;
- same-attempt recovery автоматически переиспользует существующий analysis
  context и запускает новый live agent process без попытки восстановить старый
  диалог;
- ambiguous re-entry не решается молча: оператор получает summary найденных
  артефактов и явный выбор между `resume` и `restart`;
- потеря runtime-binding сама по себе не делает issue невосстановимой, если
  context можно восстановить из наблюдаемого состояния;
- merged analysis PR или иная конфликтная картина не переиспользуются
  автоматически и дают понятную диагностику.

## Scope

В scope первой версии входят:

- re-entry logic для manual `run` в analysis lifecycle;
- observed-state summary по существующим analysis артефактам;
- различение safe auto-resume и ambiguous re-entry;
- явный operator intent для спорных случаев `resume` vs `restart`;
- переиспользование existing analysis branch/worktree/artifacts/PR там, где это
  безопасно;
- восстановление analysis запуска при потере `zellij` pane или локального
  runtime-binding;
- update `docs/issue-analysis-flow.md` и связанных analysis/runtime документов;
- тесты на repeated `run` и edge cases с worktree, PR и stale runtime.

## Non-Goals

В эту задачу не входят:

- восстановление истории диалога агента или старой `zellij` pane по
  `session_uuid`;
- новый общий multi-stage recovery flow для analysis и implementation сразу;
- автоматическое разрешение любой конфликтной истории попыток анализа;
- поддержка нескольких параллельных active analysis attempts для одной issue;
- скрытое auto-restart поведение без явного согласия оператора;
- изменение human gate или `complete-stage` vocabulary.

## Constraints And Assumptions

- источником истины по lifecycle issue остается GitHub Project status;
- повторный `run` не должен зависеть только от `.git/.ai-teamlead/`, потому что
  runtime может быть потерян или устареть;
- история агентской сессии остается источником истины по диалогу, но только
  пока сама сессия доступна; для этой задачи summary предыдущей попытки является
  лишь recovery-context, а не заменой полноценной истории;
- `poll` и `loop` не должны получать неявный interactive path из этой задачи;
- все проверки, которые затрагивают `zellij`, должны оставаться headless-safe;
- analysis branch, worktree root и artifacts dir продолжают рендериться из
  versioned config/template contract;
- merged analysis PR не должен silently превращаться в новый черновик анализа
  без отдельного явного решения по branch/PR lineage.

## User Story

Как оператор репозитория, я хочу повторно запускать `run <issue>` для задачи,
по которой analysis уже делался раньше, и получать детерминированный recovery
path с reuse существующих артефактов или явным выбором `restart`, чтобы не
терять SDD, не плодить случайные worktree и не гадать, можно ли продолжать
старую попытку.

## Use Cases

### Use Case 1. Повторный запуск в том же analysis status

- issue находится в `Waiting for Plan Review`;
- analysis SDD уже существует;
- `zellij` pane закрыта;
- оператор снова вызывает `run <issue>`;
- система находит existing analysis artifacts и draft PR;
- запускается новый live agent process в существующем analysis context без
  попытки восстановить старый диалог.

### Use Case 2. Повторный запуск после возврата статуса назад

- issue раньше уже проходила analysis;
- статус вручную вернули в `Backlog` или `Analysis Blocked`;
- analysis branch и SDD по-прежнему существуют;
- оператор вызывает `run <issue>`;
- система показывает summary предыдущей попытки и требует явный выбор:
  `resume` existing attempt или `restart` analysis.

### Use Case 3. Runtime потерян, но versioned артефакты остались

- `issues/<issue>.json` или `session.json` отсутствуют либо противоречат друг
  другу;
- analysis branch, artifacts dir и PR все еще наблюдаемы;
- оператор вызывает `run <issue>`;
- система строит re-entry решение из GitHub/git/fs observed state и не считает
  отсутствие runtime фатальной ошибкой по умолчанию.

## Dependencies

- [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)
  как SSOT analysis lifecycle и run/re-entry правил;
- [../../../docs/features/0001-ai-teamlead-cli/05-runtime-artifacts.md](../../../docs/features/0001-ai-teamlead-cli/05-runtime-artifacts.md)
  как канонический runtime layout;
- [../../../docs/adr/0008-bind-issue-to-agent-session-uuid.md](../../../docs/adr/0008-bind-issue-to-agent-session-uuid.md)
  как базовый контракт durable binding;
- [../../../docs/adr/0013-agent-session-history-as-dialog-source.md](../../../docs/adr/0013-agent-session-history-as-dialog-source.md)
  как ограничение на попытки "восстановить чат" внутри этой задачи;
- [../../../docs/adr/0021-cli-contract-poll-run-loop.md](../../../docs/adr/0021-cli-contract-poll-run-loop.md)
  как публичный CLI-контракт `run`;
- [../../../docs/adr/0024-stage-aware-run-dispatch.md](../../../docs/adr/0024-stage-aware-run-dispatch.md)
  как invariant единого issue-level entrypoint;
- issue [#4](https://github.com/dapi/ai-teamlead/issues/4) как отдельный
  follow-up по полноценному session recovery;
- issue [#15](https://github.com/dapi/ai-teamlead/issues/15) как контракт
  analysis finalization и появления артефактов, по которым теперь нужен re-entry.
