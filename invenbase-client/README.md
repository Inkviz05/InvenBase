# InvenBase clients

В этом каталоге находится активный React web-клиент: [`web/`](web/README.md).

Native Android-клиент расположен отдельно в `../invenbase-android`. Старого Android WebView-клиента в этом каталоге нет.

Команды web выполняются из `invenbase-client/web`, а не из `invenbase-client`. Файл `invenbase-client/package-lock.json`, если он присутствует локально, не используется web-сборкой; рабочий lockfile находится в `invenbase-client/web/package-lock.json`.
