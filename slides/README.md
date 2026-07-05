# Slides

Presentation decks for the project, written in [Marp](https://marp.app/) (plain markdown).

State-update talks happen every few weeks, so each one is a dated deck under `updates/`.

```
slides/
  .marprc.yml        registers the shared theme
  themes/project.css shared look for every deck
  template.md        copy this to start a new update
  updates/
    2026-07-05.md    one deck per presentation, ISO-dated
```

## Editing with live preview

Install the **Marp for VS Code** extension (`marp-team.marp-vscode`), open a deck,
and click the preview icon. It re-renders as you type.

## Exporting

Run marp from inside `slides/` so the theme load automatically:

```sh
cd slides

# PDF
npx @marp-team/marp-cli@latest updates/2026-07-05.md --pdf

# Presenter view + live reload for the whole folder
npx @marp-team/marp-cli@latest -s .
```

Speaker notes live in `<!-- ... -->` comments and show up in Marp's presenter view
and the exported PDF notes.
