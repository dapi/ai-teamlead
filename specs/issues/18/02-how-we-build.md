# Issue 18: Как строим

## Approach

Решение строится вокруг отдельного `analysis re-entry preflight`, который
выполняется внутри `run` до выбора launcher path.

Базовый порядок:

1. `run` определяет, что текущий project status относится к analysis lifecycle.
2. Orchestration собирает `observed analysis state` из нескольких источников:
   GitHub Project status, repo issue state, runtime-binding, analysis branch,
   worktree, artifacts dir и PR по analysis branch.
3. Domain-слой превращает observed state в одно из решений:
   `FreshEntry`, `ResumeExisting`, `RestartRequired`, `BlockedAmbiguity`.
4. Для `FreshEntry` текущий flow работает как раньше.
5. Для `ResumeExisting` система переиспользует existing analysis context и
   запускает новый live agent process.
6. Для `RestartRequired` в default `auto` режиме команда не гадает intent, а
   завершает run сообщением с summary и точным операторским next step.
7. Для явного operator intent `restart` система пересобирает analysis execution
   context поверх канонического analysis branch/worktree/artifacts contract.

Этот подход сохраняет главный инвариант: source of truth по статусу issue живет
в GitHub, а runtime/git/fs дают только observed execution context и диагностику.

## Affected Areas

- `docs/issue-analysis-flow.md`
  нужно обновить run/re-entry правила, allowed analysis entry statuses и edge
  case поведение;
- `docs/features/0001-ai-teamlead-cli/05-runtime-artifacts.md`
  нужно уточнить, что runtime-binding может быть reused, superseded или
  восстановлен из observed state, но не является обязательной semantic truth;
- `src/cli.rs`
  нужен явный operator-intent surface для ambiguous analysis re-entry;
- `src/app.rs`
  ручной path `run_manual_run` должен собирать observed analysis state и
  маршрутизировать `fresh/resume/restart/blocked` вместо текущей схемы
  "есть binding / нет binding";
- `src/domain.rs`
  нужен новый decision matrix для analysis re-entry и formatter итогового
  summary/diagnostics;
- `src/runtime.rs`
  нужен controlled path для reuse existing binding или выпуска нового
  `session_uuid`, если старый binding потерян или признан stale;
- `src/github.rs`
  понадобится lookup analysis PR по canonical analysis branch и, при
  необходимости, дополнительные данные для summary;
- `./.ai-teamlead/launch-agent.sh`
  должен уметь получать recovery summary и передавать его в новый agent prompt;
- integration/smoke tests
  должны покрыть same-status resume, missing runtime, deleted worktree и
  blocked ambiguity.

## Interfaces And Data

### Operator intent

Для manual `run` нужен явный, неинтерактивный CLI surface, который не ломает
`poll`/`loop` и headless automation:

```text
ai-teamlead run <issue> --analysis-reentry auto|resume|restart
```

Семантика:

- `auto`:
  - default режим;
  - автоматически выбирает `FreshEntry` или `ResumeExisting`, когда решение
    однозначно и безопасно;
  - на ambiguous состоянии не продолжает молча, а завершает run с summary и
    подсказкой, какой exact command запустить повторно;
- `resume`:
  - явный операторский сигнал продолжить существующую analysis attempt;
  - допускается только если найден согласованный analysis context;
- `restart`:
  - явный операторский сигнал начать analysis заново поверх канонического
    analysis workspace contract;
  - допускается только если previous state не merged и система может безопасно
    пересобрать branch/worktree context.

Для implementation lifecycle этот флаг не применяется и должен либо
игнорироваться с диагностикой, либо быть отвергнут как analysis-only override.

### Observed analysis state

Нужен один агрегированный объект, который формируется до launch:

- `project_status`;
- `issue_state`;
- `runtime_binding`:
  current analysis `session_uuid`, `session.json.status`, `stage_worktree_root`,
  `stage_artifacts_dir`;
- `analysis_artifacts`:
  существует ли `specs/issues/${N}/README.md`, какой там metadata-status,
  есть ли базовый комплект документов;
- `analysis_branch`:
  есть ли canonical branch локально и/или на remote;
- `analysis_worktree`:
  найден ли worktree для canonical branch, существует ли путь из runtime;
- `analysis_pr`:
  отсутствует, draft/open, closed, merged, ambiguous;
- `diagnostics`:
  список конфликтов observed state, который потом попадет в formatter.

Этот объект не пытается восстановить agent dialog. Он нужен только для
decision-making и для компактного recovery summary.

### Decision matrix

Минимальная матрица решений:

- `FreshEntry`
  когда analysis artifacts/branch/PR/binding не найдены и issue действительно
  выглядит как новый analysis start;
- `ResumeExisting`
  когда issue в waiting-status или `Analysis Blocked`, а найденный analysis
  context согласован и может быть безопасно переиспользован;
- `RestartRequired`
  когда существуют предыдущие analysis артефакты, но текущий project status
  вернулся к fresh-entry семантике (`Backlog`) или runtime/git state не дает
  безопасного default-choice;
- `BlockedAmbiguity`
  когда обнаружены конфликтные сигналы:
  merged analysis PR при analysis status, больше одного PR для canonical branch,
  противоречащие bindings или иная ситуация, где safe fallback неочевиден.

Правила:

- same-status recovery по `Waiting for Clarification`,
  `Waiting for Plan Review`, `Analysis Blocked` в режиме `auto` идет в
  `ResumeExisting`;
- `Analysis In Progress` не становится общим auto-entry status: recovery оттуда
  допускается только по явному `--analysis-reentry resume`, чтобы не плодить
  дубли live session, если старая pane еще жива;
- `Backlog` с найденными previous analysis артефактами в режиме `auto` дает
  `RestartRequired`, а не тихий `FreshEntry`;
- merged analysis PR никогда не считается resumable attempt для нового analysis
  run.

### Session-binding semantics

Нужно явно развести два понятия:

1. `logical analysis binding`
2. `live agent process`

Контракт:

- every repeated run launches a new live agent process/pane;
- если valid analysis binding существует, `ResumeExisting` переиспользует тот
  же `session_uuid`;
- если binding отсутствует или признан stale, но observed analysis state
  согласован, система выпускает новый `session_uuid` и помечает его как
  текущий analysis binding для issue;
- старый session manifest остается только диагностическим артефактом и не
  блокирует новый launch.

Это дает recovery без обязательного восстановления старого диалога и не ломает
pattern `issue <-> current session_uuid`.

### Restart path

`restart` должен быть осознанно более строгим, чем `resume`.

Минимальный контракт restart:

- используется только по явному operator intent;
- runtime-binding и live launcher context предыдущей attempt не переиспользуются;
- canonical analysis branch/worktree пересобираются из текущего config contract;
- previous unmerged PR по canonical branch можно reuse как carrier новой attempt
  либо создать заново, но merged PR не переиспользуется;
- previous analysis SDD остается доступным через git history/PR history, но
  новая attempt не обязана продолжать тот же `session_uuid`.

### Recovery summary для агента

Новый live agent process должен получать не старую историю диалога, а короткий
machine-produced summary предыдущей attempt:

- предыдущий project status и current status;
- найденный `session_uuid`, если он был;
- branch/worktree/PR state;
- состояние `specs/issues/${N}/`;
- выбранный re-entry mode: `resume` или `restart`.

Summary добавляется в launcher prompt как отдельный блок и явно помечается как
`re-entry context`, а не как канонический пересказ диалога.

## Configuration And Runtime Assumptions

- новые repo-local config поля не требуются: analysis branch, worktree и
  artifacts dir уже выводятся из существующих шаблонов;
- default CLI mode должен оставаться `auto`, чтобы не ломать текущий happy path
  для fresh issues;
- runtime schema может потребовать только небольшой extension для диагностики
  superseded binding, но не должна превращаться в source of truth по stage;
- если runtime отсутствует полностью, observed state должен по-прежнему
  собираться из GitHub, git и versioned artifacts;
- headless test runner остается предпочтительным способом проверки repeated run
  path с `zellij`.

## Risks

- если `restart` будет слишком агрессивным, можно потерять полезный локальный
  analysis context; mitigation: default `auto` never restarts silently;
- если summary предыдущей attempt начнет трактоваться как замена agent dialog,
  это вступит в конфликт с ADR-0013;
- если merged analysis PR разрешить как resumable context, можно тихо
  переписать уже принятую историю анализа;
- если `Analysis In Progress` открыть для auto-reentry без live-session
  detection, можно запустить два параллельных агента на одну issue;
- если decision matrix размазать между `app`, `runtime` и `github`, тесты
  быстро станут хрупкими и появится drift поведения.

## External Interfaces

- GitHub Project status
  остается primary source of truth по stage;
- GitHub Pull Requests
  нужны для lookup analysis PR по canonical analysis branch;
- Git
  нужен для проверки branch/worktree presence и безопасного rebuild restart
  path;
- runtime artifacts в `.git/.ai-teamlead/`
  дают current binding и launcher metadata, но не заменяют GitHub;
- launcher/prompt interface
  получает новый re-entry summary block.

## Architecture Notes

### Почему не интерактивный prompt в CLI по умолчанию

`run` является публичным CLI entrypoint, который также используется внутри
`poll` и `loop`.

Следовательно:

- default path должен оставаться неинтерактивным;
- ambiguous cases должны разрешаться не через скрытый `stdin` диалог, а через
  явный operator intent surface;
- это лучше согласуется с headless tests и с уже принятым CLI-contract ADR-0021.

### Почему recovery и session restore разделяются

Issue [#4](https://github.com/dapi/ai-teamlead/issues/4) уже выделяет отдельную
задачу про восстановление agent session по `session_uuid`.

В рамках текущего issue достаточно более узкого контракта:

- восстановить execution context;
- запустить новый live agent process;
- передать ему structured re-entry summary.

Это осознанно проще и безопаснее, чем пытаться одновременно чинить и runtime,
и dialog history, и `zellij` resurrect.

## ADR Impact

Новый ADR не требуется.

Причина:

- source of truth по lifecycle issue не меняется;
- stage-aware `run` и разделение analysis/implementation flow сохраняются;
- изменение ограничивается уточнением re-entry контракта внутри уже принятого
  семейства CLI/runtime/analysis документов.

Нужно обновить SSOT и feature-docs, связанные с `run` и runtime artifacts.

## Alternatives Considered

### Всегда отказывать при повторном запуске уже проработанной issue

Отклонено.

Это сохраняет текущий UX-gap и не использует уже существующие versioned
артефакты, branch/worktree и PR как основу для recovery.

### Всегда автоматически переиспользовать все найденные артефакты

Отклонено.

Такой путь слишком рискован для случаев, когда статус вернулся в `Backlog`,
analysis PR уже merged или найден конфликтный набор попыток.

### По умолчанию спрашивать оператора через интерактивный CLI prompt

Отклонено для MVP.

Это ухудшает пригодность `run` для automation и потребует отдельного контракта
для `poll`/`loop` и non-TTY execution.

### Пытаться восстанавливать старую agent session в рамках этой же задачи

Отклонено.

Это отдельный problem space из issue #4 и слишком сильно смешивает re-entry
decision, runtime repair и dialog recovery.

## Migration Or Rollout Notes

- тексты ошибок и re-entry diagnostics для manual `run` изменятся;
- integration tests, которые сейчас ожидают отказ из-за missing binding, нужно
  заменить на новый decision matrix;
- `docs/issue-analysis-flow.md` и `docs/features/0001-ai-teamlead-cli/05-runtime-artifacts.md`
  должны синхронно отражать новую re-entry semantics;
- smoke path должен проверяться только в headless/isolated `zellij` окружении;
- existing issues в analysis waiting-status с уже потерянным runtime станут
  recoverable без ручного редактирования внутренних json.
