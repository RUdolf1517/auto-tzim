# Kickoff — AT-02 (готово к пересылке кодеру)

Скопируй блок ниже целиком и отправь кодеру. Он самодостаточен: всё остальное он найдёт по путям в репозитории.

---

```text
AT-02 стартует. Работай по брифу.

Язык — RUST. Не Python. Workspace: auto-tzim-core (библиотека) + auto-tzim (бинарь).
Тулчейн 1.89.0, как в rust-toolchain.toml (поднят с 1.82 ради файловой блокировки из std — D-53). Python в продукте запрещён; два CI-скрипта
(scripts/check-pins.py и его самотест) — единственное исключение.

ОБЯЗАТЕЛЬНО ПРОЧИТАЙ ПЕРЕД ПЕРВОЙ СТРОКОЙ:
1. docs/manager/CODER-QUEUE.md          — правила Rust, как сдаётся этап, вся очередь
2. docs/manager/HANDOFF-AT-02.md        — твой бриф: 12 пунктов приёмки + часть B (macOS)
3. docs/config.md                       — спецификация файла настроек, поле за полем
4. docs/cli.md §2.2–2.4                 — команды, которые ты делаешь в этом этапе
5. docs/manager/DECISIONS.md            — D-43, D-44, D-45 (имя папки, облачные папки, ссылки)

ЧТО СДЕЛАТЬ (кратко, детали в брифе):
- TOML-схема 1, строгая валидация: неизвестное поле — ошибка с подсказкой
  ("unknown field \"folder[0].recurse\": did you mean \"recursive\"?");
- config validate печатает ВСЕ ошибки, не первую;
- атомарная запись: temp в том же каталоге → fsync файла → rename → fsync каталога;
  права 0600/0700 явно, чтобы umask 000 ничего не менял; битый конфиг никогда не перезаписывается;
- папка идентифицируется ИМЕНЕМ (не id): по умолчанию basename корня, уникально без учёта регистра;
- отказы: "", ".", относительный путь, /, $HOME, файл, симлинк, папка внутри AUTO_TZIM_HOME,
  облачная папка (Dropbox/iCloud/OneDrive/Яндекс.Диск), follow_symlinks/allow_hard_links/jobs>1/
  [folder.schedule]/transforms — каждый с причиной из docs/config.md §5 и кодом 2;
- folder add|list|show|edit|enable|disable|remove, folder add принимает несколько путей
  (всё или ничего); folder add ничего не обрабатывает;
- ЧАСТЬ B: починить macOS-очередь в .github/workflows/ci.yml по D-51 — build/deny/
  pinned-dependencies только на Linux, test на Linux+macOS, concurrency с cancel-in-progress,
  timeout-minutes. Запрещено: continue-on-error, || true, объявлять отменённую джобу прошедшей,
  убирать macOS;
- F-18: добавить проверку метаданных манифестов (cargo metadata --no-deps, 7 полей, оба крейта).

ЧЕГО НЕ ДЕЛАТЬ В ЭТОМ ЭТАПЕ: discovery, план, адаптер tzim, runner, state/history, --json,
scan, run, --auto, --time, --force, doctor, update-check. Это AT-03…AT-06.

ЗАВИСИМОСТИ: максимум serde + toml, версии точно =X.Y.Z, каждый — два коммита (сама
зависимость, потом строка в deny.toml), плюс cargo tree -e features в PR.

КАК СДАВАТЬ:
- один этап = один PR, коммиты "AT-02: ...";
- в PR — блок handoff из docs/manager/HANDOFF-TEMPLATE.md, все строки заполнены;
- 5 негативных контролей из брифа §4 показать КРАСНЫМИ намеренно, с номерами прогонов CI,
  потом откатить;
- сам не мержишь ("Merge/Release performed by coder: no");
- гейт-документы (GATES.md, DECISIONS.md, кейсы корпуса, строки приёмки) не редактируешь:
  если строка кажется неверной — это finding в PR, а не правка.

ЕСЛИ НУЖНО РЕШЕНИЕ (новая зависимость, новый флаг, новое значение статуса, правка гейта) —
спроси в PR одной строкой. Не угадывай: в этом проекте догадка стоит переписывания.

СЛЕДУЮЩИЕ ЭТАПЫ УЖЕ В ОЧЕРЕДИ (docs/manager/CODER-QUEUE.md §2): AT-03 бриф готов
(docs/manager/HANDOFF-AT-03.md), AT-04…AT-06 описаны в docs/manager/ROADMAP-ISSUES.md.
Следующий этап стартует, когда ревью сказало PASS, а не когда влили.
```

---

## Если кодер спросит «а точно Rust?»

Ответ короткий: да. [D-33](DECISIONS.md), принято владельцем 2026-11-01; `SPEC.md` §4.1 всё ещё
рекомендует Python и **не редактируется**, приоритет у журнала решений и [CONFLICTS C-5](../spec/CONFLICTS.md).
