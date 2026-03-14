# Feature 0005: Как строим

## Архитектура

Платформа состоит из пяти слоев:

1. `host entrypoint`
   Единая CLI-команда, например `ai-teamlead test agent-flow`, которая
   загружает repo-local config, выбирает сценарий и orchestrate-ит запуск.
2. `sandbox builder`
   Подготавливает disposable Docker sandbox с pinned `zellij`, нужными CLI и
   временным workspace snapshot.
3. `agent bridge`
   Передает в sandbox только разрешенные host env vars, credentials и
   config-files для выбранного agent profile, используя те же host-level
   значения, с которыми запущен test suite.
4. `scenario runner`
   Запускает внутри sandbox `ai-teamlead`, `launch-agent.sh`, fake/stub
   adapters и assertion hooks.
5. `artifact collector`
   Выгружает наружу логи, runtime-state, stdout/stderr, metadata сценария и
   итоговый verdict.

Канонический path должен выглядеть так:

1. host CLI читает `./.ai-teamlead/settings.yml`
2. host CLI читает versioned scenario manifest
3. host CLI собирает sandbox и workspace snapshot
4. sandbox запускает `ai-teamlead` entrypoint
5. `ai-teamlead` проходит обычный launcher/orchestration path
6. выбранный agent profile (`stub`, `claude`, `codex`) отрабатывает внутри того
   же sandbox
7. runner выполняет assertions
8. artifact collector сохраняет результат вне sandbox

## Данные и состояния

### Сущности

- `test run`
  Отдельный запуск entrypoint с уникальным `run_id`.
- `scenario`
  Versioned описание одного integration path со входными данными,
  environment bridge и assertions.
- `sandbox`
  Disposable container runtime и его filesystem.
- `workspace snapshot`
  Изолированная копия текущего репозитория для прогона.
- `agent profile`
  Набор правил, как запускать `stub`, `claude` или `codex`.
- `artifact bundle`
  Экспортируемый результат теста.

### Жизненный цикл `test run`

- `created`
- `snapshot_prepared`
- `sandbox_ready`
- `runtime_started`
- `agent_running`
- `asserting`
- `passed | failed | errored`
- `artifacts_exported`

Переходы должны быть линейными и диагностируемыми. Повторный запуск создает
новый `run_id` и не переиспользует mutable state прошлого прогона.

### Workspace snapshot

В MVP snapshot должен включать:

- текущее содержимое репозитория
- versioned `.ai-teamlead/`, `.claude/`, `.codex/`, если они есть в repo
- достаточный git context для работы `ai-teamlead`, `git worktree` и launcher

Платформа должна поддержать локальную разработку, поэтому snapshot нельзя
жестко ограничивать только последним коммитом. Предпочтительный контракт:

- sandbox получает копию текущего working tree
- исходный host repo остается неизменным
- все побочные эффекты git/runtime пишутся только внутрь snapshot

## Интерфейсы

### CLI entrypoint

Черновой контракт:

```bash
ai-teamlead test agent-flow \
  --scenario run-happy-path \
  --agent claude \
  --mode live
```

Минимальные аргументы первой версии:

- `--scenario <name>`
- `--agent <stub|claude|codex>`
- `--mode <stub|live>`

Дополнительные аргументы:

- `--keep-sandbox`
- `--artifacts-dir <path>`
- `--timeout-seconds <n>`
- `--no-build`

Правила:

- `--mode stub` разрешает только `--agent stub`
- `--mode live` разрешает `claude` и `codex`
- если `--agent` не задан, default live profile = `claude`
- итоговый exit code отражает verdict сценария

### Scenario manifest

Scenario manifest должен быть versioned и лежать внутри репозитория. Черновой
формат:

```yaml
name: run-happy-path
description: Run issue-analysis flow in isolated sandbox
mode: stub
agent: stub
fixtures:
  github_snapshot: basic-backlog.json
  repo_state: clean
commands:
  - ai-teamlead run https://github.com/org/repo/issues/123
assertions:
  - type: exit_code
    equals: 0
  - type: issue_status
    equals: Waiting for Plan Review
  - type: file_exists
    path: specs/issues/123/README.md
```

Scenario не должен содержать secrets. Он описывает:

- какие fixtures нужны
- какой agent profile используется
- какие assertions обязательны
- какой cleanup и artifact export ожидаются

### Agent bridge

Bridge должен быть явным и profile-based.

Для каждого agent profile задаются:

- `env_allowlist`
- `file_mounts`
- `binary_resolution`
- `preflight_checks`

Примеры допустимых данных bridge:

- env vars вида `OPENAI_API_KEY`, `OPENAI_BASE_URL`, `ANTHROPIC_API_KEY`
- user-local config dirs или files для конкретного агента
- repo-local `.claude/` и `.codex/`

Bridge обязан брать значения из host environment и host config, с которыми
запущен test suite, а не из отдельного скрытого тестового профиля.

Недопустимо:

- монтировать весь `$HOME` целиком
- сохранять forwarded secrets в artifact bundle
- делать implicit fallback на host filesystem вне allowlist

### Stub agent

`stub`-agent нужен не как отдельный shortcut, а как controlled implementation
того же agent contract. Он должен:

- стартовать через тот же launcher path
- получать тот же prompt context
- уметь выполнить заранее заданный сценарный outcome:
  `plan-ready`, `needs-clarification`, `blocked`
- вызывать те же внутренние команды завершения стадии

## Технические решения

- Канонический sandbox для MVP: Docker-based headless runtime.
- Канонический `zellij` внутри sandbox: pinned version по ADR-0011.
- Live и stub режимы используют один и тот же sandbox entrypoint.
- Default live path для локального тестирования: `claude` / Claude Code с
  моделью класса Sonnet.
- Вердикт сценария считается по assertions, а не по одному exit code процесса.
- Артефакты должны собираться вне зависимости от `passed` или `failed`.
- Sandbox должен быть disposable по умолчанию; сохранение возможно только через
  явный флаг `--keep-sandbox`.

## Конфигурация

Глобальные repo-local defaults логично хранить в `./.ai-teamlead/settings.yml`
в новой секции `integration_tests.agent_flow`.

Черновая схема:

```yaml
integration_tests:
  agent_flow:
    sandbox_runtime: docker
    image: ai-teamlead-agent-flow-test:local
    default_timeout_seconds: 900
    artifacts_dir: ".git/.ai-teamlead/test-runs"
    scenario_root: ".ai-teamlead/tests/agent-flow"
    agent_profiles:
      claude:
        mode: live
        default: true
        model_family: sonnet
        env_allowlist:
          - ANTHROPIC_API_KEY
          - ANTHROPIC_BASE_URL
        file_mounts: []
      codex:
        mode: live
        env_allowlist:
          - OPENAI_API_KEY
          - OPENAI_BASE_URL
        file_mounts: []
      stub:
        mode: stub
        env_allowlist: []
        file_mounts: []
```

Правила:

- без `integration_tests.agent_flow` entrypoint использует встроенные safe
  defaults
- встроенный default live profile = `claude`
- secrets и значения токенов не хранятся в versioned YAML
- в config хранятся только имена env vars, пути mounts и runtime defaults

## Ограничения реализации

- В первой версии допускается только один sandbox backend: Docker.
- В первой версии допустим только Linux-oriented headless path.
- В первой версии live assertions должны проверять orchestration и артефакты, а
  не semantic quality generated текста.
- Если agent CLI отсутствует или credentials не проброшены, сценарий должен
  завершаться явным `preflight failed`, а не неявным timeout.

## Связанные документы

- [README.md](./README.md)
- [01-what-we-build.md](./01-what-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)
- [../0003-agent-launch-orchestration/02-how-we-build.md](../0003-agent-launch-orchestration/02-how-we-build.md)
