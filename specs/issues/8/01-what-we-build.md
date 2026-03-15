# Issue 8: Что строим

## Problem

Сейчас `ai-teamlead` можно собрать и запустить только как development-инструмент
из репозитория.

Пробелы текущего состояния:

- нет канонического release flow, который публикует готовые бинарные артефакты;
- version в `Cargo.toml` не связана с semver tag и release lifecycle;
- пользователь не имеет стабильного install path через `brew` или `curl`;
- changelog не является обязательным и проверяемым release-входом;
- user-facing release notes и GitHub Release могут легко разойтись с реальным
  содержимым версии.

В результате проект остается пригодным для dogfooding, но не для нормального
публичного распространения и повторяемой установки.

## Who Is It For

- владелец репозитория, которому нужен предсказуемый способ выпускать версии;
- пользователь CLI, который хочет установить `ai-teamlead` через `brew` или
  одной `curl`-командой без локальной сборки;
- сопровождающий проекта, которому нужен единый контракт версионирования,
  changelog и release assets;
- будущая user-facing документация из issue `#9`, которая должна опираться на
  реальный install contract, а не на временные команды из development-режима.

## Outcome

Нужен минимальный публичный release contract, в котором:

- каждая публикуемая версия оформляется как semver release `vX.Y.Z`;
- `Cargo.toml`, Git tag, changelog и GitHub Release не противоречат друг другу;
- CI собирает и публикует бинарные артефакты для поддерживаемых платформ;
- install path через `brew` и `curl` использует те же опубликованные артефакты;
- release не требует ручной сборки, ручного пересчета checksums и ручного
  составления release package;
- changelog становится обязательной частью подготовки версии.

## Scope

В текущую задачу входит:

- tag-driven release flow в GitHub Actions;
- канонический versioning contract для `ai-teamlead`;
- публикация GitHub Release с бинарями и checksum-артефактами;
- install path через `brew`;
- install path через `curl`;
- changelog contract и связка changelog с release notes;
- минимальные user-facing install-инструкции, достаточные для release-пакета;
- verification contract для dry-run, smoke и реального release path.

## Non-Goals

В текущую задачу не входит:

- публикация в `crates.io`;
- поддержка `apt`, `yum`, `nix`, `winget` и других package manager;
- автоматический deploy или отдельный post-release operation flow;
- redesign полного user-facing `README.md` сверх минимально нужных install и
  release summary;
- code signing, notarization и другие advanced supply-chain меры первой версии,
  если они не требуются для базового install contract;
- несколько release channels (`stable`, `nightly`, `beta`) в первой версии.

## Constraints And Assumptions

- `ai-teamlead` остается Rust CLI, поэтому source of truth для продуктовой
  версии должен быть привязан к Rust package metadata, а не к отдельному
  произвольному файлу;
- release flow должен запускаться в CI и быть воспроизводимым без ручной сборки
  на машине владельца;
- install paths через `brew` и `curl` должны потреблять один и тот же
  опубликованный набор release assets;
- changelog должен быть version-aware и пригодным как для репозитория, так и
  для GitHub Release notes;
- full user-facing onboarding остается отдельной задачей `#9`, поэтому в этой
  задаче достаточно release-oriented install contract и минимальной документации;
- текущий проект уже использует GitHub Actions и GitHub Releases как допустимый
  operational baseline, поэтому новый flow должен ложиться на существующую
  GitHub-first модель.

## User Story

Как владелец `ai-teamlead`, я хочу выпускать версию по явному semver tag, чтобы
CI сам собрал бинарные артефакты, опубликовал GitHub Release, обновил install
каналы через `brew` и `curl` и связал это с changelog, не оставляя ручных
шагов, которые легко забыть или сделать по-разному.

## Use Cases

1. Разработчик поднимает version в `Cargo.toml`, обновляет `CHANGELOG.md`,
   создает tag `vX.Y.Z` и получает опубликованный GitHub Release с артефактами
   и checksums.
2. Пользователь на macOS или Linux выполняет install через `curl` и получает
   бинарь именно той версии, которая опубликована в GitHub Release.
3. Пользователь выполняет `brew install ...` и получает ту же версию по
   стабильному formula/tap contract.
4. Поддерживающий релиз проверяет changelog и release notes по конкретной
   версии без ручного сравнения между tag, binary assets и историей коммитов.

## Dependencies

- [../../../README.md](../../../README.md) задает текущую repo-level картину
  проекта и подтверждает, что release/user docs остаются отдельным кластером
  roadmap;
- [../../../ROADMAP.md](../../../ROADMAP.md) фиксирует issue `#8` как часть
  кластера `Release и user docs`, а issue `#9` как зависимую user-facing
  документацию;
- [../../../docs/features/0001-ai-teamlead-cli/README.md](../../../docs/features/0001-ai-teamlead-cli/README.md)
  задает базовый CLI-контракт и продуктовую рамку распространяемого бинаря;
- [../../../docs/features/0004-issue-implementation-flow/README.md](../../../docs/features/0004-issue-implementation-flow/README.md)
  и [../51/README.md](../51/README.md) явно оставляют release/deploy вне scope
  coding lifecycle, поэтому release contract нужно оформлять отдельно;
- [../../../docs/adr/0011-use-zellij-main-release-in-ci.md](../../../docs/adr/0011-use-zellij-main-release-in-ci.md)
  подтверждает, что проект уже использует GitHub Release как допустимый способ
  доставки бинарей в CI.
