# Issue 18: Как проверяем

## Acceptance Criteria

1. `run` перед repeated analysis launch собирает observed state по issue из
   GitHub status, runtime-binding, analysis artifacts, branch/worktree и PR.
2. Повторный `run` для issue в `Waiting for Clarification`,
   `Waiting for Plan Review` или `Analysis Blocked` автоматически идет по пути
   `resume`, если найденный analysis context согласован.
3. Повторный `run` для issue в `Backlog` с уже существующими analysis
   артефактами не стартует молча как fresh analysis, а требует явный выбор
   между `resume` и `restart`.
4. Потерянный runtime-binding не делает issue невосстановимой, если analysis
   context можно восстановить из GitHub/git/versioned artifacts.
5. Повторный launch всегда создает новый live agent process; восстановление
   старой agent session не является частью этой задачи.
6. Merged analysis PR или иная конфликтная картина (`multiple PR`,
   contradictory bindings) не переиспользуются автоматически и дают понятный
   blocker/diagnostic.
7. Удаленный worktree при сохраненных branch/artifacts/PR допускает re-entry с
   пересозданием execution context.
8. `docs/issue-analysis-flow.md` обновлен так, чтобы run/re-entry contract и
   edge cases были зафиксированы как SSOT.
9. Есть хотя бы один automated test на repeated analysis run path и один smoke
   сценарий для recovery уже проработанной issue.

## Ready Criteria

- сформирован явный decision matrix `fresh/resume/restart/blocked`;
- определен operator-intent contract для ambiguous repeated run;
- re-entry summary отделен от agent dialog history и не объявлен новым
  источником истины;
- описано поведение для merged PR, missing runtime и deleted worktree;
- SSOT и runtime-docs перечислены как обязательные документы для синхронизации;
- тестовый план не требует опасного запуска `zellij` в host session.

## Invariants

- GitHub Project status остается единственным источником истины по lifecycle
  issue;
- history agent session не подменяется re-entry summary из versioned
  артефактов;
- repeated run never launches silently destructive `restart` в default режиме;
- один repeated run создает новый live agent process, даже если переиспользует
  существующий `session_uuid`;
- merged analysis PR не считается resumable attempt для нового analysis запуска;
- `poll` и `loop` не получают скрытый interactive contract из этой задачи;
- re-entry decision формируется из структурированных observed facts, а не из
  парсинга свободного текста.

## Test Plan

Unit tests:

- domain decision выбирает `ResumeExisting` для `Waiting for Plan Review` при
  согласованных binding/artifacts/PR;
- domain decision выбирает `RestartRequired` для `Backlog` с previous analysis
  artifacts;
- domain decision выбирает `BlockedAmbiguity` для merged analysis PR;
- formatter repeated-run diagnostics включает status, branch/worktree/PR summary
  и точный next step;
- missing runtime при согласованном git/artifacts state не приводит к
  безусловному отказу;
- `Analysis In Progress` без явного operator intent не проходит auto-reentry.

Integration tests:

- repeated `run` из `Waiting for Plan Review` переиспользует existing analysis
  branch и artifacts, запускает новый launcher path и не меняет SDD location;
- repeated `run` из `Backlog` с previous artifacts в режиме `auto` завершается
  diagnostic-path без launch и предлагает exact retry command;
- repeated `run` из `Backlog` с `--analysis-reentry restart` пересоздает
  analysis execution context и выдает новый live launch;
- repeated `run` при удаленном worktree, но существующих branch/artifacts,
  пересоздает worktree и продолжает analysis;
- repeated `run` при отсутствии runtime json, но наличии branch/artifacts/PR,
  создает новый binding и продолжает path;
- repeated `run` при merged analysis PR выдает blocker и не запускает нового
  агента.

Manual / smoke:

- в изолированном headless `zellij` окружении выполнить analysis run до
  `Waiting for Plan Review`, затем закрыть pane и убедиться, что повторный
  `run` идет по `resume` пути;
- повторить сценарий с ручным удалением worktree и убедиться, что execution
  context пересоздается;
- проверить ambiguous case со статусом `Backlog` и существующим analysis SDD:
  default `auto` не должен запускать silent restart.

## Verification Checklist

- `run` больше не опирается только на наличие runtime-binding для analysis
  re-entry;
- observed analysis state собирается до launcher path;
- same-status resume и changed-status ambiguity различаются явно;
- default repeated run не скрывает operator choice в restart-сценариях;
- old/live session restore не заявлен как часть результата issue;
- merged PR и multiple-attempt ambiguity дают явную диагностику;
- SSOT и runtime-doc синхронизированы с новым контрактом;
- automated tests покрывают repeated run и хотя бы один recovery edge case.

## Failure Scenarios

- система silently стартует fresh analysis поверх уже существующего approved
  или nearly-complete SDD;
- stale runtime-binding блокирует recoverable issue, хотя branch/artifacts/PR
  доступны;
- `restart` случайно выполняется в `auto` режиме и перетирает полезный context;
- repeated run в `Analysis In Progress` создает второго живого агента при еще
  существующей первой pane;
- recovery summary начинает подменять собой историю диалога и вводит агента в
  заблуждение;
- merged analysis PR mistakenly трактуется как resumable draft attempt.

## Observability

- user-facing repeated-run message должен включать минимум:
  current project status, выбранный re-entry mode или blocker, branch/worktree
  state, PR state и наличие SDD artifacts;
- launcher/logging path должен показывать, что запуск является `fresh`,
  `resume` или `restart`;
- diagnostics должны различать `missing runtime`, `deleted worktree`,
  `merged PR`, `ambiguous PR list` и `operator-intent required`;
- integration assertions лучше строить по устойчивым смысловым фрагментам, а
  не по полному byte-to-byte совпадению вывода.
