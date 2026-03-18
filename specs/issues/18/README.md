# Issue 18: `run` и повторный запуск анализа уже проработанной issue

## Issue

- GitHub issue: https://github.com/dapi/ai-teamlead/issues/18
- Тип: `feature`
- Размер: `medium`
- Тип проекта: `infra/platform`

## Summary

Issue фиксирует re-entry контракт для `ai-teamlead run <issue>` в тех случаях,
когда analysis stage по этой issue уже запускался раньше и после этого остались
артефакты предыдущей попытки:

- `specs/issues/${ISSUE_NUMBER}/`;
- analysis branch/worktree;
- draft PR по analysis branch;
- runtime-binding в `.git/.ai-teamlead/`.

Ключевое решение:

- различать `live agent process` и `logical analysis binding`;
- не пытаться в рамках этой задачи восстанавливать старый диалог агента;
- при повторном `run` строить `observed re-entry state` из GitHub, git,
  runtime и versioned артефактов;
- для same-attempt recovery автоматически переиспользовать существующий analysis
  context;
- для неоднозначного повторного входа не угадывать intent, а требовать явный
  выбор оператора между `resume` и `restart`.

Это дает предсказуемый путь recovery без введения второго источника истины и без
скрытой магии вокруг потерянной `zellij` pane.

## Status

Черновик анализа готов к human review и переводу issue в
`Waiting for Plan Review`.

## Artifacts

- [01-what-we-build.md](./01-what-we-build.md)
- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)

## Related Context

- [../../../README.md](../../../README.md)
- [../../../docs/code-quality.md](../../../docs/code-quality.md)
- [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)
- [../../../docs/issue-implementation-flow.md](../../../docs/issue-implementation-flow.md)
- [../../../docs/features/0001-ai-teamlead-cli/05-runtime-artifacts.md](../../../docs/features/0001-ai-teamlead-cli/05-runtime-artifacts.md)
- [../../../docs/adr/0008-bind-issue-to-agent-session-uuid.md](../../../docs/adr/0008-bind-issue-to-agent-session-uuid.md)
- [../../../docs/adr/0013-agent-session-history-as-dialog-source.md](../../../docs/adr/0013-agent-session-history-as-dialog-source.md)
- [../../../docs/adr/0021-cli-contract-poll-run-loop.md](../../../docs/adr/0021-cli-contract-poll-run-loop.md)
- [../../../docs/adr/0024-stage-aware-run-dispatch.md](../../../docs/adr/0024-stage-aware-run-dispatch.md)
- [../../../docs/adr/0028-github-first-reconcile-and-runtime-cache-only.md](../../../docs/adr/0028-github-first-reconcile-and-runtime-cache-only.md)
- [../4/README.md](../4/README.md)
- [../15/README.md](../15/README.md)

## Open Questions

Блокирующих вопросов по текущему issue не выявлено.
