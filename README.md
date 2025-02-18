# Anki Webify

Convert Anki Decks to a web readable format.

## Installation

```sh
cargo install --git https://github.com/siriusmart/anki-webify
```

### Usage

Export your Anki deck with ***support for older Anki versions***.

```sh
anki-webify path/to/the/deck.apkg
```

You may include additional arguments
```sh
anki-webify [path to apkg] (output dir, default = ".") (media prepend, default = "./")
```
If using the default media prepend, the exported folder should be in the same level as the html file referencing it.

## Structuring

- The generated files are located in a folder of randomised ID, media files depend on this ID ***not being changed*** to work.
- The **index.json** contains a list of deck and the cards that are in the deck.
- Front and back stores the HTML files for the front and back of the deck respectively.
- The media folder stores all media files.
