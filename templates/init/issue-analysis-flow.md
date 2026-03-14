# issue-analysis-flow

Статус: project-local flow entrypoint

## Назначение

Этот файл является entrypoint prompt для анализа issue.

Он не должен содержать весь flow целиком. Вместо этого он маршрутизирует
анализ по staged prompts в каталоге:

- `./.ai-teamlead/flows/issue-analysis/`

## Порядок работы

Ты должен выполнять анализ последовательно по трем осям:

1. `./.ai-teamlead/flows/issue-analysis/01-what-we-build.md`
2. `./.ai-teamlead/flows/issue-analysis/02-how-we-build.md`
3. `./.ai-teamlead/flows/issue-analysis/03-how-we-verify.md`

Не перепрыгивай к следующей оси, пока предыдущая не собрана достаточно хорошо.

## Общие инварианты

- результат должен быть versioned SDD-комплектом в каталоге issue
- минимальный комплект документов:
  - `README.md`
  - `01-what-we-build.md`
  - `02-how-we-build.md`
  - `03-how-we-verify.md`
- минимум один документ на каждую из трех осей обязателен
- если issue маленькая, не создавай лишние документы сверх этого минимума
- вопросы пользователю задавай в агентской сессии
- если критичной информации не хватает, остановись и запроси уточнение

## Где искать project-local context

- `./.ai-teamlead/settings.yml`
- `./.ai-teamlead/README.md`
- staged prompts в `./.ai-teamlead/flows/issue-analysis/`
- project-local agent assets, если они есть:
  - `./.claude/`
  - `./.codex/`

## Связанные системные документы

- системный SSOT `docs/issue-analysis-flow.md` из репозитория `ai-teamlead`

## Ограничения источников

- launcher передает в prompt `Issue URL`, `Issue Title` и `Issue Body`; считай это
  каноническим task-level контекстом для текущей issue
- сначала используй локальные документы и файлы репозитория
- сначала читай обязательные governance-документы и явно связанные документы,
  перечисленные в issue body
- не ходи по несвязанным `specs/issues/*`, feature-спекам и ADR только ради
  поиска примеров, если текущая issue не ссылается на них напрямую
- не используй web search и не ходи в GitHub за текстом issue, если нужный
  контекст уже передан launcher'ом и доступен локально
- к внешнему поиску переходи только если локального контекста действительно не
  хватает для продолжения анализа
