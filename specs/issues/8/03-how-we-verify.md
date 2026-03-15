# Issue 8: Как проверяем

## Acceptance Criteria

- в проекте есть отдельный release workflow, который запускается по semver tag;
- release workflow публикует GitHub Release с бинарными артефактами и
  checksum-файлами;
- version из `Cargo.toml`, tag `vX.Y.Z`, changelog и release notes обязаны
  совпадать по версии;
- install path через `brew` устанавливает опубликованную release-версию, а не
  development snapshot;
- install path через `curl` устанавливает опубликованную release-версию для
  поддерживаемой платформы;
- release contract не требует ручной сборки бинарей на машине владельца;
- минимальная release/install документация синхронизирована с реальным publish
  path;
- отсутствие changelog-секции или mismatch версии блокирует публикацию.

## Ready Criteria

- выбран и задокументирован release tooling approach;
- зафиксирован version/tag/changelog contract;
- зафиксирован минимальный release matrix первой версии;
- определен канонический Homebrew tap contract;
- определен формат `curl` installer path и поддержка latest/explicit version;
- определено, какие документы и summary-слои меняются вместе с release flow.

## Invariants

- source of truth для publishable version остается `Cargo.toml`;
- publish tag всегда имеет вид `vX.Y.Z` и совпадает с `Cargo.toml`;
- `brew` и `curl` используют один и тот же published asset contract;
- changelog является обязательным release input, а не post-factum заметкой;
- обычный PR CI и release CI остаются разными pipeline;
- release automation не зависит от host-окружения разработчика;
- release assets и checksums детерминированы по версии и platform target.

## Test Plan

### Unit tests

- проверка парсинга и сравнения версии между `Cargo.toml` и semver tag;
- проверка извлечения release notes из нужной секции `CHANGELOG.md`;
- проверка asset naming и platform mapping для installer path;
- проверка выбора latest vs explicit version для `curl` installer logic, если
  эта логика реализуется в versioned script/tooling.

### Integration tests

- dry-run release pipeline собирает ожидаемый набор assets без публикации;
- workflow/fake-release path валидирует mismatch:
  - tag != `Cargo.toml`;
  - отсутствует changelog-секция;
  - отсутствует checksum;
- generated Homebrew formula ссылается на asset и checksum нужной версии;
- `curl` installer smoke path скачивает корректный asset для Linux/macOS test
  target и раскладывает бинарь в ожидаемое место;
- повторный запуск release job для той же версии не приводит к silent
  расхождению артефактов.

### Smoke tests

- контролируемый выпуск первой тестовой версии в GitHub Releases;
- ручная проверка `brew install` из published formula/tap;
- ручная проверка `curl ... | sh` или эквивалентного documented install path на
  чистом окружении;
- проверка, что опубликованный бинарь печатает ожидаемую версию.

## Happy Path

1. Разработчик обновляет `Cargo.toml` и `CHANGELOG.md`.
2. Создается semver tag `vX.Y.Z`.
3. Release workflow валидирует совпадение tag, package version и changelog.
4. CI собирает matrix бинарей, публикует checksums и создает GitHub Release.
5. Homebrew formula и `curl` installer начинают указывать на новые assets.
6. Пользователь устанавливает новую версию через `brew` или `curl`.

## Edge Cases

- tag создан, но `Cargo.toml` не обновлен;
- версия есть в `Cargo.toml`, но отсутствует в `CHANGELOG.md`;
- release workflow уже публиковал часть артефактов и упал на tap update;
- release существует, но installer path не находит asset для конкретной
  платформы;
- пользователь хочет установить не latest, а конкретную версию.

## Failure Scenarios

- GitHub Release создан без полного набора assets;
- Homebrew formula обновилась на неверный checksum;
- `curl` installer скачал asset не той архитектуры;
- release notes и `CHANGELOG.md` указывают на разные версии;
- повторный publish той же версии silently заменил артефакт вместо явной ошибки.

## Observability

Нужны диагностические сигналы минимум по следующим точкам:

- какая версия публикуется и каким tag она была вызвана;
- какой release matrix реально собран;
- какие asset names и checksums опубликованы;
- какой changelog section была использована для release notes;
- какой URL formula/tap и какой installer endpoint относятся к этой версии;
- на каком шаге release flow остановился: build, publish, tap update,
  installer validation или changelog gate.

## Verification Checklist

- release workflow отделен от обычного CI и задокументирован;
- version/tag/changelog contract проверяется автоматически;
- GitHub Release содержит ожидаемые assets и checksums;
- `brew` path проверен на актуальную версию и checksum;
- `curl` path проверен для поддерживаемых платформ;
- документация не обещает install-команд, которых нет в реальном publish path;
- первая release-версия проходит controlled smoke validation;
- partial failure не оставляет систему без явной диагностики.
