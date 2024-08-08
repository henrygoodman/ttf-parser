# ttf-parser

![](img/clean_bezier.png)


### Personal project to learn ttf parsing in Rust

(Using JetBrainsMono-Regular to test)

1. Initial rendering multiple glyphs on canvas

    ![](img/multiglyph_render.png)
- Monospace font looks alright here, although still need to adjust heights..

2. Ascending height fixed, better spacing (for mono)

    ![](img/multi_asc_fixed.png)

2. Descending heights fixed

    ![](img/multi_heights_fixed.png)

    ![](img/hello_world.png)

3. Using Bezier Curves

    ![](img/bezier_close.png)

    ![](img/bezier_far.png)



## TODO
- Fix some glyph data showing weird (space is showing as 'H')
- Handle compound glyphs
- Improve spacing
- Render glyphs totally (filling inside)
- Fix some strange aliasing issues when zoomed out (could be rasterization issue)