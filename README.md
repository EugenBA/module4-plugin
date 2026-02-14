# module4-plugin
Релизация приложения с поддержкой плагинов

## image-processor
Приложение для обработки изображений с поддержкой плагинов
### Запуск:
image-processor
--input - путь к входному изображению
--output - путь к выходному изображению
--plugin - имя плагина
--params - путь к параметрам плагина (файл в формате JSON)
--plugin-path - путь к директории с плагинами
--log-level - уровень логирования (опциональ, по умолчанию info)
                warn, error, debug, trace
--help - помощь

image-processor 
--input images.jpeg 
--output flip_blur.png 
--plugin libblur_plugin 
--params blur-plugin-config.json 
--plugin-path target/debug`

### Сборка
cargo build --bin image-processor
### 

## mirror-plugin
Плагин для отражения изображения по горизонтали и вертикали
### Сборка
cargo build --lib

### Конфигурация
```text
{
  "vertical_flip": true,
  "horizontal_flip": false,
  "log_level": "debug"
}
```
vertical-flip - отражение по вертикали (опционально)
horizontal_flip - отражение по горизонтали (опционально)
log_level - опционально (info, warn, error, debug, trace)

### Пример
Искодный файл

![images.jpeg](image/images.jpeg)

Горизонтальное отражение

![flip_h.png](image/flip_h.png)

Вертикальное отражение

![flip_v.png](image/flip_v.png)
###

## blur-plugin
Плагин для размытия изображения
### Сборка
cargo build --lib

### Конфигурация
```text
{
  "radius": 15,
  "step": 2,
  "log_level": "debug"
}
```
radius - радиус размытия
step - шаги прохода
log_level - опционально (info, warn, error, debug, trace)

### Пример
Искодный файл

![images.jpeg](image/images.jpeg)

Размытие

![flip_blur.png](image/flip_blur.png)
###


