# ttf-parser

### Personal project to learn ttf parsing in Rust

(Using JetBrainsMono-Regular to test)

1. Initial character rendering

    ![](img/initial_render.png)

2. Initial rendering multiple glyphs on canvas

    ![](img/multiglyph_render.png)
- Monospace font looks alright here, although still need to adjust heights..

3. Ascending height fixed, better spacing (for mono)

    ![](img/multi_asc_fixed.png)

## TODO
- Fix descending characters height rendering
- Fix some glyph data showing weird (space is showing as 'H')
- Render outline using bezier data
- Render glyphs totally (filling inside)