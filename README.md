# sickle

a video trimmer made in iced that also compresses a video if its too large to send to most sites
videos above 10mb will be compressed to 10mb

---

## configuration

the configuration for it is in toml which needs to be placed in the `~/.config/sickle` folder and be named `config.toml`

configuration options:
```
main_color # hex color string
background_color # hex color string
timeline_color # hex color string
hover_background # hex color string
font # font name string
notification_audio # file path to audio, string
```

