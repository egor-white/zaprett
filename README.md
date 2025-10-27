# zaprett

![GitHub Downloads (all assets, all releases)](https://img.shields.io/github/downloads/egor-white/zaprett/total)
![GitHub Downloads (all assets, latest release)](https://img.shields.io/github/downloads/egor-white/zaprett/latest/total)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/egor-white/zaprett/workflow.yml)


### [Официальный Telegram-канал модуля](https://t.me/zaprett_module)

## О модуле
Представляет собой портированную версию [zapret](https://github.com/bol-van/zapret/) от [bol-van](https://github.com/bol-van/) для Android устройств.
Требования:
* Magisk 24.1+
* Прямые руки
* Termux или другой эмулятор терминала **И/ИЛИ**  [ремейк приложения zaprett от cherret](https://еithub.com/CherretGit/zaprett-app) ("оригинал" устарел и не обновляется, вместо этого мы вдвоём занимаемся версией на Kotlin!)

На данный момент модуль умеет:
+ Включать, выключать и перезапускать nfqws
+ Работать с листами, айписетами, стратегиями
+ Предлагать обновления через Magisk/KSU/KSU Next/APatch

## Какую версию модуля выбрать?

В актуальных релизах есть 3 версии модуля, а именно:
- zaprett.zip
- zaprett-hosts.zip
- zaprett-tv.zip

Основные их отличия представленны в таблице ниже.
Для устройств на Android TV **рекомендуется использовать именно TV версию** из-за некоторых особенностей работы этой ОС.
|Версия|Списки|/etc/hosts|
|------|------|----------|
|zaprett|list-youtube.txt, list-discord.txt|:x: Нет|
|zaprett-tv|list-youtube.txt|:x: Нет|
|zaprett-hosts|list-youtube.txt,list-discord.txt|:white_check_mark: Есть|

## Что такое /etc/hosts?
Говоря грубо, это файл, который влияет на работу нейросетей и других недоступных сервисов, перенаправляя ваш траффик на сторонние сервера.

Если вы используете модули, которые подменяют этот файл (например, всевозможные блокировщики рекламы и разблокировщики нейросетей), выбирайте версию <big>**без hosts**</big>, иначе модули будут конфликтовать друг с другом.

Сервера, используемые в качестве прокси и указанные в файле hosts нам неподконтрольны, мы не несём за них отвественность, используйте с осторожностью


